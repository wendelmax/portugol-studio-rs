use std::sync::LazyLock;

use portugol_bytecode::ValueType;
use portugol_syntax::Severity;
use portugol_vm::{Io, Status, Value, Vm};
use serde::Serialize;

use crate::exercise_rules::{self, Model};

macro_rules! exercise_catalog {
    ($(($id:literal, $file:literal)),+ $(,)?) => {
        const CATALOG: &[(u32, &str)] = &[
            $(($id, include_str!(concat!("../assets/exercicios/", $file)))),+
        ];
    };
}
include!("exercise_catalog.rs");

#[derive(Clone, Serialize)]
pub struct Exercise {
    pub id: u32,
    pub statement: String,
    pub cases: usize,
}

#[derive(Serialize)]
pub struct ExerciseResult {
    pub total: usize,
    pub passed: usize,
    pub failures: Vec<String>,
    pub hints: Vec<String>,
}

struct Datum {
    kind: String,
    text: String,
}

impl Datum {
    fn matches(&self, value: &Value) -> bool {
        if value.type_name() != self.kind {
            return false;
        }
        match value {
            Value::Int(actual) => self
                .text
                .trim()
                .parse::<i32>()
                .is_ok_and(|expected| *actual == expected),
            Value::Real(actual) => self
                .text
                .trim()
                .replace(',', ".")
                .parse::<f64>()
                .is_ok_and(|expected| (*actual - expected).abs() <= 1e-9 * expected.abs().max(1.0)),
            _ => value.display() == self.text,
        }
    }
}

struct Case {
    inputs: Vec<Datum>,
    outputs: Vec<Datum>,
}

impl Case {
    fn parse(node: roxmltree::Node<'_, '_>) -> Self {
        let values = |wrapper: &str, item: &str| -> Vec<_> {
            node.children()
                .filter(|child| child.has_tag_name(wrapper))
                .flat_map(|child| child.children())
                .filter(|child| child.has_tag_name(item))
                .collect()
        };
        let read = |node: roxmltree::Node<'_, '_>| {
            let kind = node.attribute("tipodado").unwrap_or_default();
            let text = node.text().unwrap_or_default();
            Datum {
                kind: kind.to_owned(),
                text: if kind == "cadeia" { text } else { text.trim() }.to_owned(),
            }
        };
        let inputs = values("entradas", "entrada")
            .into_iter()
            .map(read)
            .collect();
        let outputs = values("saidas", "saida").into_iter().map(read).collect();
        Self { inputs, outputs }
    }
}

struct ExerciseDefinition {
    info: Exercise,
    cases: Vec<Case>,
    models: Vec<Model>,
}

fn parse_exercise(id: u32, raw: &str) -> Result<ExerciseDefinition, String> {
    let doc = roxmltree::Document::parse(raw).map_err(|_| "O arquivo .pex contém XML inválido.")?;
    let root = doc.root_element();
    if !root.has_tag_name("questao") {
        return Err("O arquivo .pex precisa conter uma questão.".into());
    }
    let statement = root
        .children()
        .find(|node| node.has_tag_name("enunciado"))
        .and_then(|node| node.text())
        .unwrap_or_default()
        .trim()
        .to_owned();
    let cases: Vec<_> = root
        .children()
        .filter(|node| node.has_tag_name("testes"))
        .flat_map(|node| node.children())
        .filter(|node| node.has_tag_name("caso"))
        .map(Case::parse)
        .collect();
    if statement.is_empty() {
        return Err("O exercício precisa de enunciado.".into());
    }
    if cases
        .iter()
        .flat_map(|case| case.inputs.iter().chain(&case.outputs))
        .any(|output| {
            !matches!(
                output.kind.as_str(),
                "inteiro" | "real" | "logico" | "caracter" | "cadeia"
            )
        })
    {
        return Err("Uma entrada ou saída do exercício tem tipo de dado inválido.".into());
    }
    let models = exercise_rules::models(root)?;
    if models.is_empty() && cases.is_empty() {
        return Err("O exercício está sem casos de teste e soluções modelo.".into());
    }
    Ok(ExerciseDefinition {
        info: Exercise {
            id,
            statement,
            cases: cases.len(),
        },
        cases,
        models,
    })
}

static EXERCISES: LazyLock<Vec<ExerciseDefinition>> = LazyLock::new(|| {
    CATALOG
        .iter()
        .map(|&(id, raw)| parse_exercise(id, raw).expect("exercício do catálogo inválido"))
        .collect()
});

pub fn inspect_exercise(raw: &str) -> Result<Exercise, String> {
    parse_exercise(0, raw).map(|exercise| exercise.info)
}

pub fn check_imported_exercise(raw: &str, source: &str) -> Result<ExerciseResult, String> {
    let exercise = parse_exercise(0, raw)?;
    check_definition(&exercise, source)
}

pub fn exercises() -> Vec<Exercise> {
    EXERCISES
        .iter()
        .map(|exercise| exercise.info.clone())
        .collect()
}

struct GradingIo<'a> {
    inputs: std::slice::Iter<'a, Datum>,
    outputs: &'a [Datum],
    read: usize,
    written: usize,
    mismatch: Option<String>,
}

impl Io for GradingIo<'_> {
    fn write(&mut self, text: &str) {
        self.write_value(&Value::str(text));
    }

    fn write_value(&mut self, value: &Value) {
        self.written += 1;
        if self.mismatch.is_some() {
            return;
        }
        let Some(expected) = self.outputs.get(self.written - 1) else {
            self.mismatch = Some("O programa produziu mais saídas do que o esperado.".into());
            return;
        };
        if !expected.matches(value) {
            let actual: String = value.display().chars().take(120).collect();
            self.mismatch = Some(format!(
                "Saída {}. Esperado {} ({:?}), recebido {} ({:?}).",
                self.written,
                expected.kind,
                expected.text,
                value.type_name(),
                actual
            ));
        }
    }

    fn read_line(&mut self, expected: ValueType) -> Option<String> {
        self.read += 1;
        let Some(input) = self.inputs.next() else {
            self.mismatch.get_or_insert_with(|| {
                "O programa solicitou mais entradas do que o caso fornece.".into()
            });
            return None;
        };
        if input.kind != expected.name() {
            self.mismatch.get_or_insert_with(|| {
                format!(
                    "Entrada {}. O caso fornece {}, mas a solução tenta ler {}.",
                    self.read,
                    input.kind,
                    expected.name()
                )
            });
            return None;
        }
        Some(input.text.clone())
    }
}

impl GradingIo<'_> {
    fn verdict(self, runtime: Result<Status, portugol_vm::RuntimeError>) -> Option<String> {
        match runtime {
            Err(error) => self.mismatch.or(Some(error.message)),
            Ok(Status::Running) => {
                Some("O programa superou o limite de instruções do caso.".into())
            }
            Ok(_) => self.mismatch.or_else(|| {
                if self.written < self.outputs.len() {
                    Some("O programa produziu menos saídas do que o esperado.".into())
                } else if self.inputs.len() != 0 {
                    Some("O programa não leu todas as entradas do caso.".into())
                } else {
                    None
                }
            }),
        }
    }
}

pub fn check_exercise(id: u32, source: &str) -> Result<ExerciseResult, String> {
    let exercise = EXERCISES
        .iter()
        .find(|exercise| exercise.info.id == id)
        .ok_or("Exercício não encontrado.")?;
    check_definition(exercise, source)
}

fn check_definition(exercise: &ExerciseDefinition, source: &str) -> Result<ExerciseResult, String> {
    let cases = &exercise.cases;
    let (diagnostics, program) = crate::analyze(source);
    let program = program.ok_or_else(|| {
        diagnostics
            .into_iter()
            .find(|diagnostic| diagnostic.severity == Severity::Error)
            .map(|diagnostic| diagnostic.message)
            .unwrap_or_else(|| "Não foi possível analisar a solução.".into())
    })?;
    let hints = exercise_rules::hints(&exercise.models, &program, source);
    if cases.is_empty() {
        return Ok(ExerciseResult {
            total: 0,
            passed: 0,
            failures: Vec::new(),
            hints,
        });
    }
    if program
        .includes
        .iter()
        .any(|include| include.library.name == "Arquivos")
    {
        return Err("O corretor executa cada caso sem acesso a arquivos. Remova a biblioteca Arquivos da solução.".into());
    }
    let module = portugol_bytecode::compile(&program).map_err(|error| error.message)?;
    let mut failures = Vec::new();
    for (index, case) in cases.iter().enumerate() {
        let mut io = GradingIo {
            inputs: case.inputs.iter(),
            read: 0,
            outputs: &case.outputs,
            written: 0,
            mismatch: None,
        };
        let status = Vm::new(&module, &mut io)
            .offline()
            .with_seed(1)
            .with_simulated_time(0)
            .step(250_000);
        if let Some(failure) = io.verdict(status) {
            failures.push(format!("Caso {}. {failure}", index + 1));
        }
    }
    Ok(ExerciseResult {
        total: cases.len(),
        passed: cases.len() - failures.len(),
        failures,
        hints,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_original_exercises_and_distinguishes_typed_outputs() {
        assert_eq!(exercises().len(), 54);
        assert!(
            exercises()
                .iter()
                .all(|exercise| exercise.cases > 0 && !exercise.statement.is_empty())
        );
        let correct = check_exercise(1, "programa { funcao inicio() { escreva(42) } }").unwrap();
        assert_eq!((correct.passed, correct.total), (1, 1));
        let wrong = check_exercise(1, "programa { funcao inicio() { escreva(\"42\") } }").unwrap();
        assert_eq!(wrong.passed, 0);
        assert!(wrong.failures[0].contains("cadeia"));
    }

    #[test]
    fn validates_and_corrects_an_imported_exercise() {
        let contents = r#"<questao><enunciado>Calcule o dobro.</enunciado><testes><caso><entradas><entrada tipodado="inteiro">12</entrada></entradas><saidas><saida tipodado="inteiro">24</saida></saidas></caso></testes></questao>"#;
        let exercise = inspect_exercise(contents).unwrap();
        assert_eq!(exercise.cases, 1);
        assert_eq!(exercise.statement, "Calcule o dobro.");
        let result = check_imported_exercise(
            contents,
            "programa { funcao inicio() { inteiro numero leia(numero) escreva(numero * 2) } }",
        )
        .unwrap();
        assert_eq!(result.passed, 1);
        let wrong_type = check_imported_exercise(
            contents,
            "programa { funcao inicio() { cadeia numero leia(numero) escreva(24) } }",
        )
        .unwrap();
        assert_eq!(wrong_type.passed, 0);
        assert!(wrong_type.failures[0].contains("tenta ler cadeia"));
        assert!(inspect_exercise("<questao/>").is_err());
    }

    #[test]
    fn checks_static_rules_without_executing_the_model() {
        let exercise = r#"<questao>
          <enunciado>Troque os valores.</enunciado>
          <solucoes>
            <solucao>
              <modelo><![CDATA[
                programa {
                    funcao inicio() {
                        inteiro primeiro, segundo, aux
                        leia(primeiro, segundo)
                        enquanto (falso) {}
                        aux = primeiro
                        primeiro = segundo
                        segundo = aux
                        escreva(primeiro, segundo)
                    }
                }
              ]]></modelo>
              <visitors>
                <visitor class-name="MandatoryInstructions"><parametro><valores><valor valor="br.univali.NoEnquanto"/></valores></parametro></visitor>
                <visitor class-name="ProhibitedInstructions"><parametro><valores><valor>NoPara</valor></valores></parametro></visitor>
                <visitor class-name="ReadWriteOrder"/>
                <visitor class-name="ReadAfterOperation"/>
                <visitor class-name="UsingAux"/>
                <visitor class-name="VarIsNotUsed"/>
              </visitors>
            </solucao>
          </solucoes>
        </questao>"#;
        let incorrect = "programa { funcao inicio() { inteiro primeiro, segundo, inutil primeiro = 1 leia(primeiro, segundo) para (inteiro i = 0; i < 1; i++) {} escreva(segundo, primeiro) } }";
        let verdict = check_imported_exercise(exercise, incorrect).unwrap();
        assert_eq!(verdict.total, 0);
        assert_eq!(verdict.hints.len(), 6, "{:?}", verdict.hints);
        for text in [
            "NoEnquanto",
            "NoPara",
            "ordem",
            "atribuições",
            "auxiliar",
            "inutil",
        ] {
            assert!(
                verdict.hints.iter().any(|hint| hint.contains(text)),
                "{text}"
            );
        }
        let valid = "programa { funcao inicio() { inteiro primeiro, segundo, aux leia(primeiro, segundo) enquanto (falso) {} aux = primeiro primeiro = segundo segundo = aux escreva(primeiro, segundo) } }";
        let verdict = check_imported_exercise(exercise, valid).unwrap();
        assert!(verdict.hints.is_empty(), "{:?}", verdict.hints);
    }

    #[test]
    fn supplies_inputs_and_bounds_loops() {
        let echo = check_exercise(
            2,
            "programa { funcao inicio() { cadeia resposta leia(resposta) escreva(resposta) } }",
        )
        .unwrap();
        let looping = check_exercise(
            1,
            "programa { funcao inicio() { enquanto (verdadeiro) {} } }",
        )
        .unwrap();
        assert_eq!((echo.passed, echo.total), (10, 10));
        assert!(looping.failures[0].contains("limite"));
    }
}
