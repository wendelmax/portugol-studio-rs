//! Validação de um `.design` contra o esquema.
//!
//! Esta etapa confere a estrutura: elementos e atributos conhecidos, tipos dos
//! valores, onde cada atributo pode aparecer, nomes e a forma dos vínculos. Se
//! os campos de `{dados.campo}` existem no registro e se os handlers existem
//! no code-behind é verificado pelo frontend FERRO, que conhece o `.fer`.

use std::collections::{HashMap, HashSet};

use ferro_hir::{Diagnostic, Span};

use crate::ast::{Attribute, Document, Element};
use crate::schema::{
    self, ANCHORS, AttributeSchema, Binding, Category, ElementSchema, Layout, Placement, ValueType,
};
use crate::value::{self, TextPart};

/// Número máximo de elementos numa janela.
pub const MAX_ELEMENTS: usize = 1_024;
/// Profundidade máxima de contêineres aninhados.
pub const MAX_DEPTH: usize = 16;

/// Palavras que não podem dar nome a controles, porque viram campos e tipos
/// no code-behind.
const RESERVED: &[&str] = &[
    "caso",
    "continue",
    "dados",
    "enquanto",
    "enum",
    "escolha",
    "escreva",
    "escreval",
    "falso",
    "faca",
    "funcao",
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
];

pub fn validate(document: &Document) -> Vec<Diagnostic> {
    let mut validator = Validator::default();
    validator.document(document);
    validator.diagnostics
}

#[derive(Default)]
struct Validator {
    diagnostics: Vec<Diagnostic>,
    names: HashMap<String, Span>,
    count: usize,
    first_binding: Option<Span>,
    sample_data: Option<Span>,
}

/// Onde um elemento está, para as regras de posição.
#[derive(Clone, Copy)]
struct Context<'a> {
    parent: &'a Element,
    layout: Layout,
    in_window: bool,
    sole_child: bool,
    depth: usize,
}

/// Filhos que ocupam espaço na tela: exclui componentes, `<Item>` e
/// `<DadosExemplo>`. Decide se uma `Pilha`, `Grade` ou `Encaixe` preenche o
/// contêiner por ser o único filho.
fn visual_children(element: &Element) -> usize {
    element
        .elements()
        .filter(|child| schema::element(&child.name.text).is_some_and(|s| s.is_visual()))
        .count()
}

impl Validator {
    fn document(&mut self, document: &Document) {
        let root = &document.root;
        if root.name.text != schema::WINDOW.name {
            self.error(
                "design.expected-window",
                format!(
                    "O elemento raiz de um .design é `<Janela>`, mas encontrei `<{}>`",
                    root.name.text
                ),
                root.name.span,
            );
            return;
        }

        self.attributes(root, &schema::WINDOW, None);
        self.texts(root);
        let sole = visual_children(root) == 1;
        for child in root.elements() {
            self.element(
                child,
                Context {
                    parent: root,
                    layout: Layout::Free,
                    in_window: true,
                    sole_child: sole,
                    depth: 1,
                },
            );
        }

        if let Some(span) = self.first_binding
            && root.attribute("Dados").is_none()
        {
            self.push(
                Diagnostic::error(
                    "design.binding-without-data",
                    "A tela usa vínculos `{dados...}`, mas a `<Janela>` não declara `Dados`",
                    span,
                )
                .with_help(
                    "Acrescente `Dados=\"NomeDoRegistro\"` à `<Janela>`, com o registro do pseudocódigo em português que guarda o estado.",
                ),
            );
        }
        if let Some(span) = self.sample_data
            && root.attribute("Dados").is_none()
        {
            self.error(
                "design.sample-without-data",
                "`<DadosExemplo>` só faz sentido quando a `<Janela>` declara `Dados`",
                span,
            );
        }
    }

    fn element(&mut self, element: &Element, context: Context<'_>) {
        self.count += 1;
        if self.count == MAX_ELEMENTS + 1 {
            self.error(
                "design.too-many-elements",
                format!("Uma janela pode ter no máximo {MAX_ELEMENTS} elementos"),
                element.name.span,
            );
        }

        let tag = element.name.text.as_str();
        if tag == schema::ITEM {
            self.item(element, context);
            return;
        }
        if tag == schema::SAMPLE_DATA {
            self.sample_data(element, context);
            return;
        }
        if tag == schema::WINDOW.name {
            self.error(
                "design.nested-window",
                "Uma `<Janela>` não pode ficar dentro de outra; cada janela tem o próprio .design",
                element.name.span,
            );
            return;
        }

        let Some(schema) = schema::element(tag) else {
            let mut diagnostic = Diagnostic::error(
                "design.unknown-element",
                format!("Elemento desconhecido: `<{tag}>`"),
                element.name.span,
            );
            if let Some(suggestion) = closest(tag, schema::tag_names()) {
                diagnostic = diagnostic.with_help(format!("Você quis dizer `<{suggestion}>`?"));
            }
            self.push(diagnostic);
            return;
        };

        if context.layout == Layout::Items {
            self.error(
                "design.list-children",
                format!(
                    "`<{}>` só aceita elementos `<Item>`",
                    context.parent.name.text
                ),
                element.name.span,
            );
            return;
        }
        if schema.category == Category::Component && !context.in_window {
            self.error(
                "design.component-in-container",
                format!(
                    "`<{tag}>` não aparece na tela e deve ficar direto na `<Janela>`, fora de contêineres"
                ),
                element.name.span,
            );
        }

        self.attributes(element, schema, Some(context));
        self.position(element, schema, context);
        self.grid_cell(element, context);
        self.texts(element);

        let children: Vec<&Element> = element.elements().collect();
        match schema.layout {
            Layout::None => {
                if let Some(first) = children.first() {
                    self.error(
                        "design.children-not-allowed",
                        format!(
                            "`<{tag}>` não pode conter outros elementos; use uma `<Pilha>`, `<Grade>` ou `<Painel>` para agrupar"
                        ),
                        first.name.span,
                    );
                }
            }
            Layout::Items => {
                self.selection(element, children.len());
                for child in children {
                    self.element(
                        child,
                        Context {
                            parent: element,
                            layout: Layout::Items,
                            in_window: false,
                            sole_child: false,
                            depth: context.depth + 1,
                        },
                    );
                }
            }
            layout => {
                if context.depth >= MAX_DEPTH && !children.is_empty() {
                    self.error(
                        "design.too-deep",
                        format!("Contêineres podem ser aninhados em no máximo {MAX_DEPTH} níveis"),
                        children[0].name.span,
                    );
                    return;
                }
                let sole = visual_children(element) == 1;
                for child in children {
                    self.element(
                        child,
                        Context {
                            parent: element,
                            layout,
                            in_window: false,
                            sole_child: sole,
                            depth: context.depth + 1,
                        },
                    );
                }
            }
        }
    }

    fn item(&mut self, element: &Element, context: Context<'_>) {
        if context.layout != Layout::Items {
            self.error(
                "design.item-outside-list",
                "`<Item>` só pode ficar dentro de `<Lista>` ou `<Combo>`",
                element.name.span,
            );
            return;
        }
        if let Some(attribute) = element.attributes.first() {
            self.error(
                "design.item-attributes",
                "`<Item>` não tem atributos; o texto do item fica entre as tags",
                attribute.name.span,
            );
        }
        if let Some(child) = element.elements().next() {
            self.error(
                "design.children-not-allowed",
                "`<Item>` só contém texto",
                child.name.span,
            );
        }
    }

    fn sample_data(&mut self, element: &Element, context: Context<'_>) {
        if !context.in_window {
            self.error(
                "design.sample-data-placement",
                "`<DadosExemplo>` deve ficar direto na `<Janela>`",
                element.name.span,
            );
            return;
        }
        if self.sample_data.is_some() {
            self.error(
                "design.duplicate-sample-data",
                "A janela já tem um `<DadosExemplo>`",
                element.name.span,
            );
            return;
        }
        self.sample_data = Some(element.name.span);
        let mut seen = HashSet::new();
        for attribute in &element.attributes {
            let field = &attribute.name.text;
            if !value::is_identifier(field) {
                self.error(
                    "design.invalid-name",
                    format!("`{field}` não é um nome de campo válido"),
                    attribute.name.span,
                );
            } else if !seen.insert(field.as_str()) {
                self.error(
                    "design.duplicate-attribute",
                    format!("O campo `{field}` aparece mais de uma vez"),
                    attribute.name.span,
                );
            }
        }
        if let Some(child) = element.elements().next() {
            self.error(
                "design.children-not-allowed",
                "`<DadosExemplo>` não contém outros elementos",
                child.name.span,
            );
        }
    }

    fn attributes(
        &mut self,
        element: &Element,
        schema: &'static ElementSchema,
        context: Option<Context<'_>>,
    ) {
        let tag = element.name.text.as_str();
        let mut seen = HashSet::new();
        for attribute in &element.attributes {
            let name = attribute.name.text.as_str();
            if !seen.insert(name) {
                self.error(
                    "design.duplicate-attribute",
                    format!("O atributo `{name}` aparece mais de uma vez"),
                    attribute.name.span,
                );
                continue;
            }
            let Some(attribute_schema) = schema.attribute(name) else {
                self.unknown_attribute(attribute, schema);
                continue;
            };
            if let Some(context) = context
                && !self.placement(attribute, attribute_schema, context)
            {
                continue;
            }
            if schema.name == schema::WINDOW.name && name == "Formato" {
                if attribute.value.trim() != crate::CURRENT_VERSION.to_string() {
                    self.error(
                        "design.unsupported-version",
                        format!(
                            "`Formato=\"{}\"` não é suportado; a versão atual é {}",
                            attribute.value,
                            crate::CURRENT_VERSION
                        ),
                        attribute.value_span,
                    );
                }
                continue;
            }
            self.value(attribute, attribute_schema);
        }

        for attribute_schema in schema.all_attributes() {
            if attribute_schema.required && !seen.contains(attribute_schema.name) {
                self.error(
                    "design.missing-attribute",
                    format!(
                        "Falta o atributo obrigatório `{}` em `<{tag}>`",
                        attribute_schema.name
                    ),
                    element.name.span,
                );
            }
        }
    }

    fn unknown_attribute(&mut self, attribute: &Attribute, schema: &ElementSchema) {
        let name = &attribute.name.text;
        let mut diagnostic = Diagnostic::error(
            "design.unknown-attribute",
            format!("`<{}>` não tem o atributo `{name}`", schema.name),
            attribute.name.span,
        );
        let accepted = schema.all_attributes();
        if let Some(same) = accepted.iter().find(|a| a.name.eq_ignore_ascii_case(name)) {
            diagnostic = diagnostic.with_help(format!(
                "Os atributos começam com maiúscula: `{}`.",
                same.name
            ));
        } else if let Some(suggestion) = closest(name, accepted.iter().map(|a| a.name)) {
            diagnostic = diagnostic.with_help(format!("Você quis dizer `{suggestion}`?"));
        }
        self.push(diagnostic);
    }

    /// Confere se o atributo vale no contêiner do elemento. Devolve falso
    /// quando o atributo não deve ter o valor conferido.
    fn placement(
        &mut self,
        attribute: &Attribute,
        attribute_schema: &AttributeSchema,
        context: Context<'_>,
    ) -> bool {
        let parent = context.parent.name.text.as_str();
        let name = attribute.name.text.as_str();
        let layouts = matches!(context.layout, Layout::Stack | Layout::Grid | Layout::Dock);
        match attribute_schema.placement {
            Placement::Any => true,
            Placement::Free if context.layout == Layout::Free => true,
            Placement::Free => {
                self.push(
                    Diagnostic::error(
                        "design.position-in-layout",
                        format!("Dentro de `<{parent}>` a posição é automática; `{name}` não se aplica"),
                        attribute.name.span,
                    )
                    .with_help(format!(
                        "Remova `{name}`. Para posicionar à mão, coloque o controle num `<Painel>` ou direto na `<Janela>`."
                    )),
                );
                false
            }
            Placement::Grid if context.layout == Layout::Grid => true,
            Placement::Dock if context.layout == Layout::Dock => true,
            Placement::Grid | Placement::Dock => {
                let owner = if attribute_schema.placement == Placement::Grid {
                    "Grade"
                } else {
                    "Encaixe"
                };
                self.push(
                    Diagnostic::error(
                        "design.attached-outside-parent",
                        format!("`{name}` só vale para elementos dentro de `<{owner}>`"),
                        attribute.name.span,
                    )
                    .with_help(format!(
                        "`{name}` é uma propriedade que o `<{owner}>` usa para posicionar o filho."
                    )),
                );
                false
            }
            Placement::Layout if layouts => true,
            Placement::Layout => {
                self.push(Diagnostic::warning(
                    "design.alignment-ignored",
                    format!(
                        "`{name}` não tem efeito dentro de `<{parent}>`, que posiciona por X e Y"
                    ),
                    attribute.name.span,
                ));
                false
            }
        }
    }

    fn position(&mut self, element: &Element, schema: &ElementSchema, context: Context<'_>) {
        if context.layout != Layout::Free || !schema.is_visual() {
            return;
        }
        let fills = schema.category == Category::Container
            && matches!(schema.layout, Layout::Stack | Layout::Grid | Layout::Dock)
            && context.sole_child;
        let missing: Vec<&str> = ["X", "Y"]
            .into_iter()
            .filter(|a| element.attribute(a).is_none())
            .collect();
        if !fills && !missing.is_empty() {
            self.push(
                Diagnostic::warning(
                    "design.missing-position",
                    format!(
                        "`<{}>` sem {} fica encostado no canto superior esquerdo de `<{}>`",
                        schema.name,
                        missing.join(" e "),
                        context.parent.name.text
                    ),
                    element.name.span,
                )
                .with_help("Defina `X` e `Y`, ou coloque o controle numa `<Pilha>` ou `<Grade>`."),
            );
        }
    }

    /// `Linha` e `Coluna` precisam existir na `Grade` do pai.
    fn grid_cell(&mut self, element: &Element, context: Context<'_>) {
        if context.layout != Layout::Grid {
            return;
        }
        for (attribute, tracks) in [("Linha", "Linhas"), ("Coluna", "Colunas")] {
            let Some(index) = element
                .attribute(attribute)
                .and_then(|a| a.value.trim().parse::<usize>().ok().map(|i| (a, i)))
            else {
                continue;
            };
            let count = context
                .parent
                .attribute(tracks)
                .and_then(|t| value::tracks(&t.value))
                .map_or(1, |t| t.len());
            if index.1 >= count {
                self.error(
                    "design.out-of-range",
                    format!(
                        "`{attribute}=\"{}\"`, mas a `<Grade>` tem {count} {}",
                        index.1,
                        if attribute == "Linha" {
                            if count == 1 { "linha" } else { "linhas" }
                        } else if count == 1 {
                            "coluna"
                        } else {
                            "colunas"
                        }
                    ),
                    index.0.value_span,
                );
            }
        }
    }

    /// `Selecionado` precisa apontar para um `<Item>` existente.
    fn selection(&mut self, element: &Element, items: usize) {
        let Some(selected) = element.attribute("Selecionado") else {
            return;
        };
        let Ok(index) = selected.value.trim().parse::<i64>() else {
            return;
        };
        if index >= items as i64 {
            self.error(
                "design.out-of-range",
                format!(
                    "`Selecionado=\"{index}\"`, mas há {items} {}",
                    if items == 1 { "item" } else { "itens" }
                ),
                selected.value_span,
            );
        }
    }

    fn texts(&mut self, element: &Element) {
        for text in element.texts() {
            self.push(
                Diagnostic::error(
                    "design.unexpected-text",
                    format!(
                        "Texto solto não é permitido dentro de `<{}>`",
                        element.name.text
                    ),
                    text.span,
                )
                .with_help("O texto de um controle vai no atributo `Texto`; só `<Item>` tem texto entre as tags."),
            );
        }
    }

    fn value(&mut self, attribute: &Attribute, schema: &AttributeSchema) {
        let name = attribute.name.text.as_str();
        let span = attribute.value_span;

        if attribute.raw.contains(['{', '}']) {
            let parts = match value::text_parts(&attribute.raw, span.start) {
                Ok(parts) => parts,
                Err(diagnostic) => {
                    self.push(diagnostic);
                    return;
                }
            };
            if value::has_binding(&parts) {
                self.binding(attribute, schema, &parts);
                return;
            }
        }

        let text = attribute.value.as_str();
        let invalid = |expected: &str| {
            Diagnostic::error(
                "design.invalid-value",
                format!("`{name}` espera {expected}, mas recebeu \"{text}\""),
                span,
            )
        };
        match schema.value {
            ValueType::Text => {}
            ValueType::Integer { min, max } => match text.trim().parse::<i64>() {
                Ok(number) if (min..=max).contains(&number) => {}
                Ok(number) => self.error(
                    "design.out-of-range",
                    format!("`{name}` deve estar entre {min} e {max}, mas vale {number}"),
                    span,
                ),
                Err(_) => self.push(invalid("um número inteiro")),
            },
            ValueType::Bool => {
                if !matches!(text, "verdadeiro" | "falso") {
                    self.push(invalid("um lógico").with_help("Use \"verdadeiro\" ou \"falso\"."));
                }
            }
            ValueType::Event => {
                self.identifier(text, span, "o handler de evento");
            }
            ValueType::TypeName => {
                self.identifier(text, span, "o registro");
            }
            ValueType::ControlName => {
                if self.identifier(text, span, "o controle") {
                    if self.names.contains_key(text) {
                        self.error(
                            "design.duplicate-name",
                            format!("Já existe um elemento chamado `{text}`"),
                            span,
                        );
                    } else {
                        self.names.insert(text.to_owned(), span);
                    }
                }
            }
            ValueType::Choice(options) => {
                if !options.contains(&text) {
                    let mut diagnostic =
                        invalid(&format!("um destes valores: {}", options.join(", ")));
                    if let Some(option) = options.iter().find(|o| o.eq_ignore_ascii_case(text)) {
                        diagnostic = diagnostic.with_help(format!("Você quis dizer \"{option}\"?"));
                    }
                    self.push(diagnostic);
                }
            }
            ValueType::Anchors => match value::list(text) {
                Some(items) => {
                    let mut seen = HashSet::new();
                    for item in items {
                        if !ANCHORS.contains(&item) {
                            self.push(invalid("bordas separadas por vírgula").with_help(
                                "Bordas válidas: Cima, Baixo, Esquerda e Direita; por exemplo, \"Cima,Esquerda\".",
                            ));
                            break;
                        }
                        if !seen.insert(item) {
                            self.error(
                                "design.duplicate-anchor",
                                format!("A borda `{item}` aparece mais de uma vez"),
                                span,
                            );
                        }
                    }
                }
                None => self.push(invalid("bordas separadas por vírgula")),
            },
            ValueType::Thickness => {
                if value::thickness(text).is_none() {
                    self.push(invalid("uma espessura").with_help(
                        "Use um número (\"8\"), dois (\"8,4\": horizontal e vertical) ou quatro (\"1,2,3,4\": esquerda, cima, direita e baixo).",
                    ));
                }
            }
            ValueType::Tracks => {
                if value::tracks(text).is_none() {
                    self.push(invalid("tamanhos separados por vírgula").with_help(
                        "Cada tamanho é \"Auto\", \"*\" (proporcional, como \"2*\") ou um número; por exemplo, \"Auto,*\".",
                    ));
                }
            }
            ValueType::ResourcePath => self.resource_path(text, span),
        }
    }

    fn binding(&mut self, attribute: &Attribute, schema: &AttributeSchema, parts: &[TextPart]) {
        let name = attribute.name.text.as_str();
        if self.first_binding.is_none() {
            self.first_binding = Some(attribute.value_span);
        }
        match schema.binding {
            Binding::None => self.push(
                Diagnostic::error(
                    "design.binding-not-allowed",
                    format!("`{name}` não aceita vínculos `{{...}}`"),
                    attribute.value_span,
                )
                .with_help("Para escrever chaves literais, use `\\{` e `\\}`."),
            ),
            Binding::OneWay if schema.value == ValueType::Text => {}
            Binding::OneWay | Binding::TwoWay => {
                if value::single_binding(parts).is_none() {
                    self.push(
                        Diagnostic::error(
                            "design.binding-must-be-whole",
                            format!(
                                "O valor de `{name}` deve ser um único vínculo, sem texto em volta"
                            ),
                            attribute.value_span,
                        )
                        .with_help(format!(
                            "Escreva só o vínculo, como `{name}=\"{{dados.campo}}\"`."
                        )),
                    );
                }
            }
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
                "design.invalid-path",
                "O caminho deve ser relativo à pasta do projeto e não pode usar `..`",
                span,
            );
        }
    }

    /// Nomes viram identificadores em FERRO, Rust e Slint; por isso só aceitam
    /// letras sem acento, dígitos e `_`.
    fn identifier(&mut self, name: &str, span: Span, what: &str) -> bool {
        if !value::is_identifier(name) {
            self.push(
                Diagnostic::error(
                    "design.invalid-name",
                    format!("Nome inválido para {what}: \"{name}\""),
                    span,
                )
                .with_help("Use letras sem acento, dígitos e `_`, começando por uma letra."),
            );
            return false;
        }
        if RESERVED.contains(&name) {
            self.error(
                "design.reserved-name",
                format!("`{name}` é uma palavra reservada e não pode nomear {what}"),
                span,
            );
            return false;
        }
        true
    }

    fn error(&mut self, code: &'static str, message: impl Into<String>, span: Span) {
        self.push(Diagnostic::error(code, message, span));
    }

    fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }
}

/// Sugestão mais próxima por distância de edição, se for plausível.
fn closest<'a>(word: &str, candidates: impl Iterator<Item = &'a str>) -> Option<&'a str> {
    let limit = (word.chars().count() / 3).max(1);
    candidates
        .map(|candidate| {
            (
                edit_distance(&word.to_lowercase(), &candidate.to_lowercase()),
                candidate,
            )
        })
        .filter(|(distance, _)| *distance <= limit)
        .min_by_key(|(distance, _)| *distance)
        .map(|(_, candidate)| candidate)
}

/// Distância de edição com transposição de letras vizinhas
/// (Damerau–Levenshtein restrita), para que `Botoa` sugira `Botao`.
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
