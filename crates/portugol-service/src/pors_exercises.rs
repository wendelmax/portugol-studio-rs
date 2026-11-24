//! Formato versionado (`schema_version = 1`) e avaliador executável para
//! exercícios em Portugol-RS (`.pors`), preservando o avaliador clássico `.pex`.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock};

use portugol_bytecode::ValueType;
use portugol_vm::{Io, RuntimeLimitReason, Status, Vm};
use serde::{Deserialize, Serialize};

pub const PORS_EXERCISE_SCHEMA_VERSION: u32 = 1;
const MAX_CASE_STEPS: usize = 500_000;
const MAX_CASE_STRING_BYTES: usize = 4 * 1024 * 1024;
const MAX_CASE_OUTPUT_BYTES: usize = 256 * 1024;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PorsExerciseCase {
    pub name: String,
    pub input: String,
    pub expected_output: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PorsExerciseDefinition {
    pub schema_version: u32,
    pub id: String,
    pub title: String,
    pub statement: String,
    pub starter_code: String,
    #[serde(default)]
    pub hints: Vec<String>,
    pub cases: Vec<PorsExerciseCase>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PorsExerciseSummary {
    pub schema_version: u32,
    pub id: String,
    pub title: String,
    pub statement: String,
    pub starter_code: String,
    pub hints: Vec<String>,
    pub cases: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PorsExerciseStatus {
    Passed,
    Failed,
    CompileError,
    RuntimeError,
    LimitExceeded,
}

#[derive(Debug, Clone, Serialize)]
pub struct PorsExerciseCaseOutcome {
    pub name: String,
    pub passed: bool,
    pub expected_output: String,
    pub actual_output: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PorsExerciseEvaluation {
    pub schema_version: u32,
    pub exercise_id: String,
    pub status: PorsExerciseStatus,
    pub total: usize,
    pub passed: usize,
    pub cases: Vec<PorsExerciseCaseOutcome>,
    pub diagnostics: Vec<String>,
    pub hints: Vec<String>,
}

const BUILTIN_RAW: &[&str] = &[
    include_str!("../assets/exercicios-pors/01-dobro.pors.json"),
    include_str!("../assets/exercicios-pors/02-maior-de-dois.pors.json"),
    include_str!("../assets/exercicios-pors/03-soma-ate-n.pors.json"),
    include_str!("../assets/exercicios-pors/04-funcao-fatorial.pors.json"),
    include_str!("../assets/exercicios-pors/05-divisao-segura.pors.json"),
    include_str!("../assets/exercicios-pors/06-filtrar-positivos.pors.json"),
    include_str!("../assets/exercicios-pors/07-normalizar-csv.pors.json"),
    include_str!("../assets/exercicios-pors/08-dias-do-mes.pors.json"),
    include_str!("../assets/exercicios-pors/09-resumo-json.pors.json"),
    include_str!("../assets/exercicios-pors/10-somar-lista-json.pors.json"),
];

static BUILTIN_CATALOG: LazyLock<Vec<PorsExerciseDefinition>> = LazyLock::new(|| {
    BUILTIN_RAW
        .iter()
        .map(|raw| parse_pors_exercise_json(raw).expect("exercício PORS embutido deve ser válido"))
        .collect()
});

pub fn parse_pors_exercise_json(raw: &str) -> Result<PorsExerciseDefinition, String> {
    let definition: PorsExerciseDefinition = serde_json::from_str(raw)
        .map_err(|error| format!("JSON de exercício .pors inválido: {error}"))?;
    if definition.schema_version != PORS_EXERCISE_SCHEMA_VERSION {
        return Err(format!(
            "Versão de esquema de exercício .pors não suportada ({}); esperado {}.",
            definition.schema_version, PORS_EXERCISE_SCHEMA_VERSION
        ));
    }
    if definition.id.trim().is_empty() {
        return Err("O exercício .pors precisa de um identificador `id`.".into());
    }
    if definition.title.trim().is_empty() || definition.statement.trim().is_empty() {
        return Err("O exercício .pors precisa de título e enunciado.".into());
    }
    if definition.cases.is_empty() {
        return Err("O exercício .pors precisa definir ao menos um caso de teste.".into());
    }
    Ok(definition)
}

fn summarize(definition: &PorsExerciseDefinition) -> PorsExerciseSummary {
    PorsExerciseSummary {
        schema_version: definition.schema_version,
        id: definition.id.clone(),
        title: definition.title.clone(),
        statement: definition.statement.clone(),
        starter_code: definition.starter_code.clone(),
        hints: definition.hints.clone(),
        cases: definition.cases.len(),
    }
}

pub fn pors_exercises() -> Vec<PorsExerciseSummary> {
    BUILTIN_CATALOG.iter().map(summarize).collect()
}

pub fn inspect_pors_exercise(raw_json: &str) -> Result<PorsExerciseSummary, String> {
    parse_pors_exercise_json(raw_json).map(|definition| summarize(&definition))
}

pub fn check_pors_exercise(id: &str, source: &str) -> Result<PorsExerciseEvaluation, String> {
    let definition = BUILTIN_CATALOG
        .iter()
        .find(|item| item.id == id)
        .ok_or_else(|| format!("Exercício PORS não encontrado: {id}"))?;
    Ok(evaluate_pors_exercise(definition, source))
}

pub fn check_imported_pors_exercise(
    raw_json: &str,
    source: &str,
) -> Result<PorsExerciseEvaluation, String> {
    let definition = parse_pors_exercise_json(raw_json)?;
    Ok(evaluate_pors_exercise(&definition, source))
}

pub fn evaluate_pors_exercise(
    definition: &PorsExerciseDefinition,
    source: &str,
) -> PorsExerciseEvaluation {
    let program = match portugol_rs_frontend::lower(source) {
        Ok(program) => program,
        Err(diagnostics) => {
            return PorsExerciseEvaluation {
                schema_version: definition.schema_version,
                exercise_id: definition.id.clone(),
                status: PorsExerciseStatus::CompileError,
                total: definition.cases.len(),
                passed: 0,
                cases: Vec::new(),
                diagnostics: diagnostics
                    .into_iter()
                    .map(|d| d.render(source, "exercicio.pors"))
                    .collect(),
                hints: Vec::new(),
            };
        }
    };
    let module = match portugol_bytecode::compile_pors(&program) {
        Ok(module) => module,
        Err(error) => {
            return PorsExerciseEvaluation {
                schema_version: definition.schema_version,
                exercise_id: definition.id.clone(),
                status: PorsExerciseStatus::CompileError,
                total: definition.cases.len(),
                passed: 0,
                cases: Vec::new(),
                diagnostics: vec![error.message],
                hints: Vec::new(),
            };
        }
    };

    let mut outcomes = Vec::with_capacity(definition.cases.len());
    let mut passed_count = 0usize;
    let mut saw_limit = false;
    let mut saw_runtime_error = false;

    for case in &definition.cases {
        let interrupt = Arc::new(AtomicBool::new(false));
        let mut io = BoundedCaseIo::new(&case.input, MAX_CASE_OUTPUT_BYTES, Arc::clone(&interrupt));
        let mut vm = Vm::new(&module, &mut io)
            .with_interrupt(interrupt)
            .with_string_memory_limit(MAX_CASE_STRING_BYTES);

        match vm.step(MAX_CASE_STEPS) {
            Ok(Status::Finished) => {
                let actual = normalize_output(&io.output);
                let expected = normalize_output(&case.expected_output);
                let passed = actual == expected;
                if passed {
                    passed_count += 1;
                }
                outcomes.push(PorsExerciseCaseOutcome {
                    name: case.name.clone(),
                    passed,
                    expected_output: expected,
                    actual_output: actual,
                    error: None,
                });
            }
            Ok(Status::Running) => {
                saw_limit = true;
                outcomes.push(PorsExerciseCaseOutcome {
                    name: case.name.clone(),
                    passed: false,
                    expected_output: normalize_output(&case.expected_output),
                    actual_output: normalize_output(&io.output),
                    error: Some(format!(
                        "Limite de {MAX_CASE_STEPS} passos excedido (possível laço infinito)."
                    )),
                });
            }
            Err(error) => {
                if io.truncated
                    || matches!(
                        error.limit,
                        Some(RuntimeLimitReason::StringMemory | RuntimeLimitReason::Time)
                    )
                {
                    saw_limit = true;
                } else {
                    saw_runtime_error = true;
                }
                let message = if io.truncated {
                    format!("Limite máximo de {MAX_CASE_OUTPUT_BYTES} bytes de saída excedido.")
                } else {
                    error.message
                };
                outcomes.push(PorsExerciseCaseOutcome {
                    name: case.name.clone(),
                    passed: false,
                    expected_output: normalize_output(&case.expected_output),
                    actual_output: normalize_output(&io.output),
                    error: Some(message),
                });
            }
        }
    }

    let status = if passed_count == definition.cases.len() {
        PorsExerciseStatus::Passed
    } else if saw_limit {
        PorsExerciseStatus::LimitExceeded
    } else if saw_runtime_error {
        PorsExerciseStatus::RuntimeError
    } else {
        PorsExerciseStatus::Failed
    };

    let hints = if status == PorsExerciseStatus::Passed {
        Vec::new()
    } else {
        definition.hints.clone()
    };

    PorsExerciseEvaluation {
        schema_version: definition.schema_version,
        exercise_id: definition.id.clone(),
        status,
        total: definition.cases.len(),
        passed: passed_count,
        cases: outcomes,
        diagnostics: Vec::new(),
        hints,
    }
}

fn normalize_output(text: &str) -> String {
    text.replace("\r\n", "\n").trim().to_owned()
}

struct BoundedCaseIo {
    input: VecDeque<String>,
    output: String,
    max_output_bytes: usize,
    truncated: bool,
    interrupt: Arc<AtomicBool>,
}

impl BoundedCaseIo {
    fn new(input: &str, max_output_bytes: usize, interrupt: Arc<AtomicBool>) -> Self {
        Self {
            input: input.lines().map(str::to_string).collect(),
            output: String::new(),
            max_output_bytes,
            truncated: false,
            interrupt,
        }
    }
}

impl Io for BoundedCaseIo {
    fn write(&mut self, text: &str) {
        if self.truncated {
            return;
        }
        if self.output.len().saturating_add(text.len()) > self.max_output_bytes {
            self.truncated = true;
            self.interrupt.store(true, Ordering::Release);
            return;
        }
        self.output.push_str(text);
    }

    fn read_line(&mut self, _expected: ValueType) -> Option<String> {
        self.input.pop_front()
    }

    fn clear(&mut self) {
        self.output.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluates_all_builtin_pors_exercises_with_passing_and_failing_solutions() {
        let catalog = pors_exercises();
        assert_eq!(catalog.len(), 10);
        for summary in &catalog {
            assert_eq!(summary.schema_version, PORS_EXERCISE_SCHEMA_VERSION);
            assert!(portugol_rs_frontend::lower(&summary.starter_code).is_ok());
        }

        let solutions = [
            (
                "pors-01-dobro",
                "funcao inicio() { seja mutavel n: inteiro = 0; leia(n); escreva(n * 2); }",
            ),
            (
                "pors-02-maior-de-dois",
                "funcao inicio() { seja mutavel a: inteiro = 0; seja mutavel b: inteiro = 0; leia(a); leia(b); se (a >= b) { escreva(a); } senao { escreva(b); } }",
            ),
            (
                "pors-03-soma-ate-n",
                "funcao inicio() { seja mutavel n: inteiro = 0; leia(n); seja mutavel s: inteiro = 0; para (seja mutavel i: inteiro = 1; i <= n; i += 1) { s += i; } escreva(s); }",
            ),
            (
                "pors-04-funcao-fatorial",
                "funcao fatorial(n: inteiro) -> inteiro { se (n <= 1) { retorne 1; } senao { retorne n * fatorial(n - 1); } } funcao inicio() { seja mutavel n: inteiro = 0; leia(n); escreva(fatorial(n)); }",
            ),
            (
                "pors-05-divisao-segura",
                "funcao dividir(a: inteiro, b: inteiro) -> Resultado<inteiro, cadeia> { se (b == 0) { retorne Resultado::Erro(\"erro\"); } retorne Resultado::Ok(a / b); } funcao inicio() { seja mutavel a: inteiro = 0; seja mutavel b: inteiro = 0; leia(a); leia(b); escolha dividir(a, b) { caso Resultado::Ok(q) => { escreva(q); } caso Resultado::Erro(e) => { escreva(e); } } }",
            ),
            (
                "pors-06-filtrar-positivos",
                "funcao inicio() { seja mutavel n: inteiro = 0; leia(n); seja mutavel positivos: vetor<inteiro> = []; para (seja mutavel i: inteiro = 0; i < n; i += 1) { seja mutavel v: inteiro = 0; leia(v); se (v > 0) { adicionar(positivos, v); } } escolha obter(positivos, 0) { caso Opcao::Algum(p) => { escreva(tamanho(positivos), \" \", p); } caso Opcao::Nenhum => { escreva(tamanho(positivos), \" 0\"); } } }",
            ),
            (
                "pors-07-normalizar-csv",
                "funcao inicio() { seja mutavel linha: cadeia = \"\"; leia(linha); seja partes: vetor<cadeia> = Texto.dividir(linha, \",\"); escreva(tamanho(partes), \"|\", Texto.caixa_alta(Texto.aparar(partes[0]))); }",
            ),
            (
                "pors-08-dias-do-mes",
                "funcao calcular_dias(ano_txt: cadeia, mes_txt: cadeia) -> Resultado<inteiro, cadeia> { seja ano: inteiro = Tipos.para_inteiro(ano_txt)?; seja mes: inteiro = Tipos.para_inteiro(mes_txt)?; retorne Calendario.dias_no_mes(ano, mes); } funcao inicio() { seja mutavel ano_txt: cadeia = \"\"; seja mutavel mes_txt: cadeia = \"\"; leia(ano_txt); leia(mes_txt); escolha calcular_dias(ano_txt, mes_txt) { caso Resultado::Ok(d) => { escreva(d); } caso Resultado::Erro(_) => { escreva(\"invalido\"); } } }",
            ),
            (
                "pors-09-resumo-json",
                "funcao processar(texto: cadeia) -> Resultado<cadeia, cadeia> { seja nome: cadeia = Json.obter_cadeia(texto, \"nome\")?; seja idade: inteiro = Json.obter_inteiro(texto, \"idade\")?; retorne Resultado::Ok(nome + \":\" + Tipos.inteiro_para_cadeia(idade + 1)); } funcao inicio() { seja mutavel linha: cadeia = \"\"; leia(linha); escolha processar(linha) { caso Resultado::Ok(s) => { escreva(s); } caso Resultado::Erro(_) => { escreva(\"invalido\"); } } }",
            ),
            (
                "pors-10-somar-lista-json",
                "funcao somar_json(texto: cadeia) -> Resultado<inteiro, cadeia> { seja lista: vetor<inteiro> = Json.obter_lista_inteiro(texto, \"valores\")?; seja mutavel total: inteiro = 0; para (seja mutavel i: inteiro = 0; i < tamanho(lista); i += 1) { total += lista[i]; } retorne Resultado::Ok(total); } funcao inicio() { seja mutavel linha: cadeia = \"\"; leia(linha); escolha somar_json(linha) { caso Resultado::Ok(s) => { escreva(s); } caso Resultado::Erro(_) => { escreva(\"invalido\"); } } }",
            ),
        ];

        for (id, source) in solutions {
            let result = check_pors_exercise(id, source).unwrap();
            assert_eq!(
                result.status,
                PorsExerciseStatus::Passed,
                "{id}: {result:?}"
            );
            assert_eq!(result.passed, result.total);
            assert!(result.hints.is_empty());
        }

        let wrong = check_pors_exercise(
            "pors-01-dobro",
            "funcao inicio() { seja mutavel n: inteiro = 0; leia(n); escreva(n); }",
        )
        .unwrap();
        assert_eq!(wrong.status, PorsExerciseStatus::Failed);
        assert_eq!(wrong.passed, 1); // apenas caso zero coincide
        assert!(!wrong.hints.is_empty());
    }

    #[test]
    fn distinguishes_compile_errors_limits_and_unsupported_schema_versions() {
        let compile_err =
            check_pors_exercise("pors-01-dobro", "funcao inicio() { escreva(inexistente); }")
                .unwrap();
        assert_eq!(compile_err.status, PorsExerciseStatus::CompileError);
        assert!(!compile_err.diagnostics.is_empty());

        let infinite = check_pors_exercise(
            "pors-01-dobro",
            "funcao inicio() { enquanto (verdadeiro) {} }",
        )
        .unwrap();
        assert_eq!(infinite.status, PorsExerciseStatus::LimitExceeded);

        let future_schema = r#"{"schema_version": 99, "id": "x", "title": "t", "statement": "s", "starter_code": "funcao inicio() {}", "cases": [{"name": "c", "input": "", "expected_output": ""}]}"#;
        let err = inspect_pors_exercise(future_schema).unwrap_err();
        assert!(err.contains("não suportada"));
    }
}
