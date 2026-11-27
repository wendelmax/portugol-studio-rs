//! Validação semântica de um `.tela` contra o esquema.

use std::collections::{HashMap, HashSet};

use portugol_hir::{Diagnostic, Span};

use crate::ast::{Body, Control, Name, Property, Screen, Value};
use crate::schema::{self, ANCHORS, PropertySchema, ValueType};

/// Número máximo de controles numa janela.
pub const MAX_CONTROLS: usize = 1_024;
/// Profundidade máxima de painéis aninhados.
pub const MAX_DEPTH: usize = 8;

/// Palavras de PORS que não podem dar nome a janelas e controles, porque
/// viram campos e tipos no code-behind.
const RESERVED: &[&str] = &[
    "caso",
    "continue",
    "enquanto",
    "enum",
    "escolha",
    "escreva",
    "falso",
    "faca",
    "funcao",
    "janela",
    "leia",
    "mutavel",
    "para",
    "pare",
    "registro",
    "retorne",
    "se",
    "seja",
    "senao",
    "verdadeiro",
    "versao",
];

pub fn validate(screen: &Screen) -> Vec<Diagnostic> {
    let mut validator = Validator::default();
    validator.screen(screen);
    validator.diagnostics
}

#[derive(Default)]
struct Validator {
    diagnostics: Vec<Diagnostic>,
    names: HashMap<String, Span>,
    count: usize,
}

impl Validator {
    fn screen(&mut self, screen: &Screen) {
        if screen.version != crate::CURRENT_VERSION {
            self.error(
                "tela.unsupported-version",
                format!(
                    "Versão {} do formato .tela não é suportada; a versão atual é {}",
                    screen.version,
                    crate::CURRENT_VERSION
                ),
                screen.version_span.unwrap_or_default(),
            );
        }

        let window = &screen.window;
        self.declare(&window.name, "janela");
        self.properties(
            &window.body,
            schema::WINDOW_PROPERTIES
                .iter()
                .collect::<Vec<_>>()
                .as_slice(),
            |name| schema::WINDOW_PROPERTIES.iter().find(|p| p.name == name),
            &window.name,
            "a janela",
        );
        for control in &window.body.controls {
            self.control(control, 1);
        }
    }

    fn control(&mut self, control: &Control, depth: usize) {
        self.count += 1;
        if self.count == MAX_CONTROLS + 1 {
            self.error(
                "tela.too-many-controls",
                format!("Uma janela pode ter no máximo {MAX_CONTROLS} controles"),
                control.name.span,
            );
        }

        let Some(schema) = schema::control(&control.kind.text) else {
            let mut diagnostic = Diagnostic::error(
                "tela.unknown-control",
                format!("Controle desconhecido: `{}`", control.kind.text),
                control.kind.span,
            );
            if let Some(suggestion) =
                closest(&control.kind.text, schema::CONTROLS.iter().map(|c| c.name))
            {
                diagnostic = diagnostic.with_help(format!("Você quis dizer `{suggestion}`?"));
            }
            self.diagnostics.push(diagnostic);
            self.declare(&control.name, "controle");
            return;
        };

        self.declare(&control.name, "controle");

        if !schema.visual && depth > 1 {
            self.error(
                "tela.nonvisual-in-container",
                format!(
                    "`{}` não é visual e deve ficar direto na janela, fora de painéis",
                    schema.name
                ),
                control.kind.span,
            );
        }

        let subject = format!("`{}`", control.name.text);
        self.properties(
            &control.body,
            schema.all_properties().collect::<Vec<_>>().as_slice(),
            |name| schema.property(name),
            &control.name,
            &subject,
        );
        self.list_selection(&control.body);

        if !control.body.controls.is_empty() && !schema.container {
            self.error(
                "tela.children-not-allowed",
                format!(
                    "`{}` não pode conter outros controles; use um `painel`",
                    schema.name
                ),
                control.body.controls[0].kind.span,
            );
            return;
        }
        if depth >= MAX_DEPTH && !control.body.controls.is_empty() {
            self.error(
                "tela.too-deep",
                format!("Painéis podem ser aninhados em no máximo {MAX_DEPTH} níveis"),
                control.body.controls[0].kind.span,
            );
            return;
        }
        for child in &control.body.controls {
            self.control(child, depth + 1);
        }
    }

    fn properties<'s>(
        &mut self,
        body: &Body,
        accepted: &[&'s PropertySchema],
        lookup: impl Fn(&str) -> Option<&'s PropertySchema>,
        owner: &Name,
        subject: &str,
    ) {
        let mut seen: HashSet<&str> = HashSet::new();
        for property in &body.properties {
            if seen.contains(property.name.text.as_str()) {
                self.error(
                    "tela.duplicate-property",
                    format!(
                        "A propriedade `{}` aparece mais de uma vez",
                        property.name.text
                    ),
                    property.name.span,
                );
                continue;
            }
            seen.insert(&property.name.text);

            match lookup(&property.name.text) {
                Some(schema) => self.value(property, schema),
                None => {
                    let mut diagnostic = Diagnostic::error(
                        "tela.unknown-property",
                        format!(
                            "Propriedade desconhecida para {subject}: `{}`",
                            property.name.text
                        ),
                        property.name.span,
                    );
                    if let Some(suggestion) =
                        closest(&property.name.text, accepted.iter().map(|p| p.name))
                    {
                        diagnostic =
                            diagnostic.with_help(format!("Você quis dizer `{suggestion}`?"));
                    }
                    self.diagnostics.push(diagnostic);
                }
            }
        }

        for schema in accepted {
            if schema.required && !seen.contains(schema.name) {
                self.error(
                    "tela.missing-property",
                    format!(
                        "Falta a propriedade obrigatória `{}` em {subject}",
                        schema.name
                    ),
                    owner.span,
                );
            }
        }
    }

    fn value(&mut self, property: &Property, schema: &PropertySchema) {
        let span = property.value_span;
        let expected = |label: &str| {
            Diagnostic::error(
                "tela.invalid-value",
                format!(
                    "`{}` espera {label}, mas recebeu {}",
                    property.name.text,
                    property.value.kind_label()
                ),
                span,
            )
        };

        match (schema.value, &property.value) {
            (ValueType::Integer { min, max }, Value::Integer(value)) => {
                if !(min..=max).contains(value) {
                    self.error(
                        "tela.out-of-range",
                        format!(
                            "`{}` deve estar entre {min} e {max}, mas vale {value}",
                            property.name.text
                        ),
                        span,
                    );
                }
            }
            (ValueType::Integer { .. }, _) => self.diagnostics.push(expected("um inteiro")),
            (ValueType::Text, Value::Text(_)) => {}
            (ValueType::Text, _) => self.diagnostics.push(
                expected("uma cadeia entre aspas")
                    .with_help("Escreva o texto entre aspas: \"...\"."),
            ),
            (ValueType::Bool, Value::Bool(_)) => {}
            (ValueType::Bool, _) => self
                .diagnostics
                .push(expected("um lógico").with_help("Use `verdadeiro` ou `falso`.")),
            (ValueType::Event, Value::Name(name)) => {
                self.identifier(name, span, "o handler de evento");
            }
            (ValueType::Event, _) => self.diagnostics.push(
                expected("o nome de uma função")
                    .with_help("Escreva o nome da função sem aspas, como `btnSomar_clicar`."),
            ),
            (ValueType::TypeName, Value::Name(name)) => {
                self.identifier(name, span, "o registro");
            }
            (ValueType::TypeName, _) => self
                .diagnostics
                .push(expected("o nome de um registro, sem aspas")),
            (ValueType::TextList, Value::List(items)) => {
                for item in items {
                    if !matches!(item.value, Value::Text(_)) {
                        self.error(
                            "tela.invalid-value",
                            format!(
                                "Os itens de `{}` devem ser cadeias, mas há um {}",
                                property.name.text,
                                item.value.kind_label()
                            ),
                            item.span,
                        );
                    }
                }
            }
            (ValueType::TextList, _) => self
                .diagnostics
                .push(expected("uma lista de cadeias").with_help("Exemplo: [\"Um\", \"Dois\"].")),
            (ValueType::Anchors, Value::List(items)) => {
                let mut seen = Vec::new();
                for item in items {
                    match &item.value {
                        Value::Name(name) if ANCHORS.contains(&name.as_str()) => {
                            if seen.contains(name) {
                                self.error(
                                    "tela.duplicate-anchor",
                                    format!("A borda `{name}` aparece mais de uma vez"),
                                    item.span,
                                );
                            }
                            seen.push(name.clone());
                        }
                        _ => self.error(
                            "tela.invalid-value",
                            "Bordas válidas: cima, baixo, esquerda e direita",
                            item.span,
                        ),
                    }
                }
            }
            (ValueType::Anchors, _) => self
                .diagnostics
                .push(expected("uma lista de bordas").with_help("Exemplo: [cima, esquerda].")),
            (ValueType::Choice(options), Value::Name(name)) => {
                if !options.contains(&name.as_str()) {
                    self.error(
                        "tela.invalid-value",
                        format!(
                            "`{}` aceita {}, mas recebeu `{name}`",
                            property.name.text,
                            options.join(" ou ")
                        ),
                        span,
                    );
                }
            }
            (ValueType::Choice(options), _) => self.diagnostics.push(expected(&format!(
                "um destes nomes: {}",
                options.join(", ")
            ))),
            (ValueType::ResourcePath, Value::Text(path)) => self.resource_path(path, span),
            (ValueType::ResourcePath, _) => {
                self.diagnostics.push(expected("um caminho entre aspas"))
            }
        }
    }

    /// `selecionado` precisa apontar para um item existente.
    fn list_selection(&mut self, body: &Body) {
        let (Some(selected), items) = (body.property("selecionado"), body.property("itens")) else {
            return;
        };
        let Value::Integer(index) = selected.value else {
            return;
        };
        let count = match items.map(|p| &p.value) {
            Some(Value::List(items)) => items.len() as i64,
            Some(_) => return,
            None => 0,
        };
        if index >= count {
            self.error(
                "tela.out-of-range",
                format!(
                    "`selecionado` vale {index}, mas há {count} {}",
                    if count == 1 { "item" } else { "itens" }
                ),
                selected.value_span,
            );
        }
    }

    fn resource_path(&mut self, path: &str, span: Span) {
        let normalized = path.replace('\\', "/");
        let absolute = normalized.starts_with('/')
            || normalized.as_bytes().get(1) == Some(&b':')
            || normalized.is_empty();
        let escapes = normalized.split('/').any(|part| part == "..");
        if absolute || escapes {
            self.error(
                "tela.invalid-path",
                "O caminho deve ser relativo à pasta do projeto e não pode usar `..`",
                span,
            );
        }
    }

    fn declare(&mut self, name: &Name, what: &str) {
        if !self.identifier(&name.text, name.span, &format!("a {what}")) {
            return;
        }
        if self.names.contains_key(&name.text) {
            self.error(
                "tela.duplicate-name",
                format!("Já existe um controle ou janela chamado `{}`", name.text),
                name.span,
            );
        } else {
            self.names.insert(name.text.clone(), name.span);
        }
    }

    /// Nomes viram identificadores em PORS, Rust e Slint; por isso só aceitam
    /// letras ASCII, dígitos e `_`.
    fn identifier(&mut self, name: &str, span: Span, what: &str) -> bool {
        let mut chars = name.chars();
        let valid_start = chars
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_');
        if !valid_start || !chars.all(|c| c.is_ascii_alphanumeric() || c == '_') {
            self.diagnostics.push(
                Diagnostic::error(
                    "tela.invalid-name",
                    format!("Nome inválido para {what}: `{name}`"),
                    span,
                )
                .with_help("Use letras sem acento, dígitos e `_`, começando por uma letra."),
            );
            return false;
        }
        if RESERVED.contains(&name) {
            self.error(
                "tela.reserved-name",
                format!("`{name}` é uma palavra reservada e não pode nomear {what}"),
                span,
            );
            return false;
        }
        true
    }

    fn error(&mut self, code: &'static str, message: impl Into<String>, span: Span) {
        self.diagnostics
            .push(Diagnostic::error(code, message, span));
    }
}

/// Sugestão mais próxima por distância de edição, se for plausível.
fn closest<'a>(word: &str, candidates: impl Iterator<Item = &'a str>) -> Option<&'a str> {
    let limit = (word.chars().count() / 3).max(1);
    candidates
        .map(|candidate| (edit_distance(word, candidate), candidate))
        .filter(|(distance, _)| *distance <= limit)
        .min_by_key(|(distance, _)| *distance)
        .map(|(_, candidate)| candidate)
}

/// Distância de edição com transposição de letras vizinhas
/// (Damerau–Levenshtein restrita), para que `botoa` sugira `botao`.
fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut table = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for (i, row) in table.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in table[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            let mut best = (table[i - 1][j] + 1)
                .min(table[i][j - 1] + 1)
                .min(table[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                best = best.min(table[i - 2][j - 2] + 1);
            }
            table[i][j] = best;
        }
    }
    table[a.len()][b.len()]
}
