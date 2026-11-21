use std::collections::VecDeque;
use std::sync::LazyLock;

use portugol_bytecode::ValueType;
use portugol_syntax::Severity;
use portugol_vm::{Io, Status, Value, Vm};
use serde::Serialize;

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
}

struct Expected {
    kind: String,
    text: String,
}

impl Expected {
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
    inputs: Vec<String>,
    outputs: Vec<Expected>,
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
        let text = |node: roxmltree::Node<'_, '_>| node.text().unwrap_or_default().to_owned();
        let inputs = values("entradas", "entrada")
            .into_iter()
            .map(|entry| {
                if entry.attribute("tipodado") == Some("cadeia") {
                    text(entry)
                } else {
                    let text = text(entry);
                    text.trim().to_owned()
                }
            })
            .collect();
        let outputs = values("saidas", "saida")
            .into_iter()
            .map(|output| Expected {
                kind: output.attribute("tipodado").unwrap_or_default().to_owned(),
                text: text(output),
            })
            .collect();
        Self { inputs, outputs }
    }
}

static EXERCISES: LazyLock<Vec<(Exercise, Vec<Case>)>> = LazyLock::new(|| {
    CATALOG
        .iter()
        .map(|&(id, raw)| {
            let doc = roxmltree::Document::parse(raw).expect("exercício do catálogo inválido");
            let root = doc.root_element();
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
            (
                Exercise {
                    id,
                    statement,
                    cases: cases.len(),
                },
                cases,
            )
        })
        .collect()
});

pub fn exercises() -> Vec<Exercise> {
    EXERCISES
        .iter()
        .map(|(exercise, _)| exercise.clone())
        .collect()
}

struct GradingIo<'a> {
    inputs: VecDeque<String>,
    outputs: &'a [Expected],
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

    fn read_line(&mut self, _expected: ValueType) -> Option<String> {
        self.inputs.pop_front()
    }
}

impl GradingIo<'_> {
    fn verdict(self, runtime: Result<Status, portugol_vm::RuntimeError>) -> Option<String> {
        match runtime {
            Err(error) => Some(error.message),
            Ok(Status::Running) => {
                Some("O programa superou o limite de instruções do caso.".into())
            }
            Ok(_) => self.mismatch.or_else(|| {
                if self.written < self.outputs.len() {
                    Some("O programa produziu menos saídas do que o esperado.".into())
                } else if !self.inputs.is_empty() {
                    Some("O programa não leu todas as entradas do caso.".into())
                } else {
                    None
                }
            }),
        }
    }
}

pub fn check_exercise(id: u32, source: &str) -> Result<ExerciseResult, String> {
    let (_, cases) = EXERCISES
        .iter()
        .find(|(exercise, _)| exercise.id == id)
        .ok_or("Exercício não encontrado.")?;
    let (diagnostics, program) = crate::analyze(source);
    let program = program.ok_or_else(|| {
        diagnostics
            .into_iter()
            .find(|diagnostic| diagnostic.severity == Severity::Error)
            .map(|diagnostic| diagnostic.message)
            .unwrap_or_else(|| "Não foi possível analisar a solução.".into())
    })?;
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
            inputs: case.inputs.iter().cloned().collect(),
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
