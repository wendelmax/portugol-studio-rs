//! Execução de uma tela desktop (ADR 0018, seções 4 e 5): a janela começa com
//! os valores do `.design`, `AoAbrir` ajusta o estado inicial e cada evento
//! chama a função ligada com a janela atual, guardando a janela devolvida.
//!
//! O estado fica em [`ScreenValue`], que pode atravessar threads (os valores
//! da VM usam `Rc`); ele é convertido para a VM só durante cada chamada. Cada
//! estado fica no histórico, para o depurador mostrar a janela antes e depois
//! de cada evento.

use std::cell::RefCell;
use std::rc::Rc;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value as Json, json};

use ferro_bytecode::Module;
use ferro_hir::{self as hir, Type};
use ferro_tela::codebehind::{FieldDef, FieldSource, FieldType, ScreenType, TypeDef, screen_type};
use ferro_tela::{Document, Element, schema};
use ferro_vm::{BufferIo, Host, Value, Vm};

/// Cria o host de cada chamada: quem mostra os diálogos (`Dialogo`) da tela.
pub type ScreenHostFactory = Box<dyn Fn() -> Box<dyn Host> + Send>;

/// Instruções por evento antes de interromper o handler (laço infinito).
pub const SCREEN_EVENT_STEP_BUDGET: usize = 5_000_000;
/// Estados guardados no histórico.
const MAX_HISTORY: usize = 256;

/// Valor do estado da janela, independente da VM.
#[derive(Debug, Clone, PartialEq)]
pub enum ScreenValue {
    Int(i32),
    Real(f64),
    Bool(bool),
    Char(char),
    Text(String),
    /// Vetor, registro (campos na ordem) ou enum (`[variante, dados...]`).
    List(Vec<ScreenValue>),
}

impl ScreenValue {
    fn to_vm(&self) -> Value {
        match self {
            ScreenValue::Int(n) => Value::Int(*n),
            ScreenValue::Real(n) => Value::Real(*n),
            ScreenValue::Bool(b) => Value::Bool(*b),
            ScreenValue::Char(c) => Value::Char(*c),
            ScreenValue::Text(text) => Value::Str(Rc::from(text.as_str())),
            ScreenValue::List(items) => Value::Array(Rc::new(RefCell::new(
                items.iter().map(Self::to_vm).collect(),
            ))),
        }
    }

    fn from_vm(value: &Value) -> Result<Self, String> {
        Ok(match value {
            Value::Int(n) => ScreenValue::Int(*n),
            Value::Real(n) => ScreenValue::Real(*n),
            Value::Bool(b) => ScreenValue::Bool(*b),
            Value::Char(c) => ScreenValue::Char(*c),
            Value::Str(text) => ScreenValue::Text(text.to_string()),
            Value::Array(items) => ScreenValue::List(
                items
                    .borrow()
                    .iter()
                    .map(Self::from_vm)
                    .collect::<Result<_, _>>()?,
            ),
            other => {
                return Err(format!(
                    "A janela devolvida tem um valor inesperado: {}",
                    other.display()
                ));
            }
        })
    }

    fn list_mut(&mut self) -> Option<&mut Vec<ScreenValue>> {
        match self {
            ScreenValue::List(items) => Some(items),
            _ => None,
        }
    }

    fn list(&self) -> Option<&[ScreenValue]> {
        match self {
            ScreenValue::List(items) => Some(items),
            _ => None,
        }
    }
}

/// Evento vindo da janela em execução.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenEvent {
    /// `Nome` do controle; vazio para a própria janela.
    pub control: String,
    /// Atributo do evento: `AoClicar`, `AoAlterar`, `AoSelecionar`...
    pub event: String,
    /// O que o usuário digitou (`texto`), marcou (`verdadeiro`/`falso`) ou
    /// escolheu (posição), aplicado à janela antes de chamar a função.
    #[serde(default)]
    pub value: Option<Json>,
}

/// Resultado de iniciar a tela ou tratar um evento.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenStep {
    /// Propriedades da janela, controles por nome e `dados`.
    pub snapshot: Json,
    /// Função chamada, quando o evento estava ligado.
    pub handler: Option<String>,
    /// O que o código escreveu com `escreva`.
    pub output: String,
    /// Posição do estado no histórico.
    pub step: usize,
}

/// Por que a tela não pôde iniciar.
#[derive(Debug, Clone, PartialEq)]
pub enum ScreenStartError {
    /// O `.design` ou o código têm erros; a mensagem resume o primeiro.
    Invalid(String),
    /// O código rodou e falhou (em `AoAbrir`).
    Runtime(String),
}

impl std::fmt::Display for ScreenStartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScreenStartError::Invalid(message) | ScreenStartError::Runtime(message) => {
                f.write_str(message)
            }
        }
    }
}

pub struct ScreenRun {
    module: Module,
    /// Propriedades com `{dados.campo}`, reavaliadas a cada evento.
    bound: Vec<crate::screen_bindings::BoundProperty>,
    document: Document,
    screen: ScreenType,
    record_types: Vec<hir::RecordDef>,
    enum_types: Vec<hir::EnumDef>,
    state: ScreenValue,
    history: Vec<ScreenValue>,
    step_budget: usize,
    host: Option<ScreenHostFactory>,
}

fn first_message(diagnostics: &[ferro_hir::Diagnostic], file: &str) -> String {
    diagnostics.first().map_or_else(
        || format!("{file} tem erros"),
        |d| format!("{file}: {}", d.message),
    )
}

fn element_at<'a>(document: &'a Document, path: &[usize]) -> Option<&'a Element> {
    let mut element = &document.root;
    for &index in path {
        element = element.elements().nth(index)?;
    }
    Some(element)
}

/// Valor inicial de uma propriedade a partir do atributo, ou o padrão dela.
/// Atributos vinculados (`{dados.campo}`) usam o padrão nesta etapa.
fn attribute_value(element: &Element, attribute: &str, ty: &FieldType) -> ScreenValue {
    let raw = element
        .attribute(attribute)
        .map(|a| a.value.as_str())
        .filter(|value| !value.trim_start().starts_with('{'));
    match ty {
        FieldType::Integer => {
            let default = match attribute {
                "Selecionado" => -1,
                "Largura" | "Altura" => schema::element(&element.name.text).map_or(0, |s| {
                    let (width, height) = s.default_size;
                    (if attribute == "Largura" {
                        width
                    } else {
                        height
                    }) as i32
                }),
                _ => 0,
            };
            ScreenValue::Int(raw.and_then(|v| v.trim().parse().ok()).unwrap_or(default))
        }
        FieldType::Bool => {
            let default = matches!(attribute, "Visivel" | "Habilitado");
            ScreenValue::Bool(raw.map_or(default, |v| v.trim() == "verdadeiro"))
        }
        _ => ScreenValue::Text(raw.unwrap_or_default().to_owned()),
    }
}

fn items(element: &Element) -> ScreenValue {
    ScreenValue::List(
        element
            .elements()
            .filter(|child| child.name.text == schema::ITEM)
            .map(|item| {
                ScreenValue::Text(
                    item.texts()
                        .map(|t| t.text.clone())
                        .collect::<Vec<_>>()
                        .join(" "),
                )
            })
            .collect(),
    )
}

impl ScreenRun {
    /// Compila o code-behind com os tipos da tela, monta a janela inicial e
    /// chama `AoAbrir`, se a janela tiver.
    pub fn start(
        design_source: &str,
        code_source: &str,
    ) -> Result<(Self, ScreenStep), ScreenStartError> {
        let analysis = ferro_tela::analyze(design_source);
        let document = match analysis.document {
            Some(document) if !analysis.has_errors() => document,
            _ => {
                return Err(ScreenStartError::Invalid(first_message(
                    &analysis.diagnostics,
                    "Desenho da tela",
                )));
            }
        };
        let program = ferro_frontend::lower_with(
            code_source,
            &crate::designer::code_behind_options(design_source),
        )
        .map_err(|d| ScreenStartError::Invalid(first_message(&d, "Código da tela")))?;
        let check = crate::designer::check_screen(design_source, Some(code_source));
        if let Some(problem) = check.design.first().or(check.code.first()) {
            return Err(ScreenStartError::Invalid(problem.message.clone()));
        }
        let module = ferro_bytecode::compile_ferro(&program).map_err(|error| {
            ScreenStartError::Invalid(format!("Código da tela: {}", error.message))
        })?;
        let (screen, _) = screen_type(&document);
        let mut run = Self {
            bound: crate::screen_bindings::bound_properties(&document),
            module,
            state: ScreenValue::List(Vec::new()),
            history: Vec::new(),
            record_types: program.record_types,
            enum_types: program.enum_types,
            screen,
            document,
            step_budget: SCREEN_EVENT_STEP_BUDGET,
            host: None,
        };
        run.state = run.initial_window();
        run.sync_bound();
        run.history.push(run.state.clone());
        let opening = run
            .document
            .root
            .attribute("AoAbrir")
            .map(|a| a.value.trim().to_owned())
            .filter(|h| !h.is_empty());
        let mut output = String::new();
        if let Some(handler) = &opening {
            output = run.call(handler).map_err(ScreenStartError::Runtime)?;
            run.history.push(run.state.clone());
        }
        let step = run.step(opening, output);
        Ok((run, step))
    }

    /// Host das chamadas dos eventos (a janela da tela, com os diálogos do
    /// sistema). Sem ele, `Dialogo` dá erro. `AoAbrir` roda antes de a janela
    /// existir, por isso usa o host só a partir dos eventos.
    pub fn set_host(&mut self, factory: ScreenHostFactory) {
        self.host = Some(factory);
    }

    /// Limite de instruções por evento (testes usam valores pequenos).
    pub fn with_step_budget(mut self, budget: usize) -> Self {
        self.step_budget = budget;
        self
    }

    fn record_def(&self, ty: Type) -> Option<&hir::RecordDef> {
        match ty {
            Type::Record(id) => self.record_types.iter().find(|record| record.id == id),
            _ => None,
        }
    }

    fn record_by_name(&self, name: &str) -> Option<&hir::RecordDef> {
        self.record_types.iter().find(|record| record.name == name)
    }

    /// Valor padrão de um tipo Ferro: `0`, `""`, `falso`, vetor vazio, e os
    /// campos de um registro com os padrões deles.
    fn default_value(&self, ty: Type, depth: usize) -> ScreenValue {
        match ty {
            Type::Integer | Type::Unit => ScreenValue::Int(0),
            Type::Real => ScreenValue::Real(0.0),
            Type::Boolean => ScreenValue::Bool(false),
            Type::String => ScreenValue::Text(String::new()),
            Type::Character => ScreenValue::Char('\0'),
            Type::Record(_) if depth < 32 => ScreenValue::List(
                self.record_def(ty)
                    .map(|record| {
                        record
                            .fields
                            .iter()
                            .map(|field| self.default_value(field.ty, depth + 1))
                            .collect()
                    })
                    .unwrap_or_default(),
            ),
            Type::Enum(id) if depth < 32 => {
                let payload = self
                    .enum_types
                    .iter()
                    .find(|definition| definition.id == id)
                    .and_then(|definition| definition.variants.first())
                    .map(|variant| variant.payload_types.clone())
                    .unwrap_or_default();
                let mut value = vec![ScreenValue::Int(0)];
                value.extend(
                    payload
                        .into_iter()
                        .map(|ty| self.default_value(ty, depth + 1)),
                );
                ScreenValue::List(value)
            }
            _ => ScreenValue::List(Vec::new()),
        }
    }

    fn control_value(&self, definition: &TypeDef, element: &Element) -> ScreenValue {
        ScreenValue::List(
            definition
                .fields
                .iter()
                .map(|field| match &field.source {
                    FieldSource::Attribute(attribute) => {
                        attribute_value(element, attribute, &field.ty)
                    }
                    FieldSource::Items => items(element),
                    _ => ScreenValue::Text(String::new()),
                })
                .collect(),
        )
    }

    fn initial_window(&self) -> ScreenValue {
        let root = &self.document.root;
        ScreenValue::List(
            self.screen
                .window
                .fields
                .iter()
                .map(|field| match (&field.source, &field.ty) {
                    (FieldSource::Attribute(attribute), ty) => attribute_value(root, attribute, ty),
                    (FieldSource::Control(path), FieldType::Named(tag)) => {
                        match (self.control_type(tag), element_at(&self.document, path)) {
                            (Some(definition), Some(element)) => {
                                self.control_value(definition, element)
                            }
                            _ => ScreenValue::List(Vec::new()),
                        }
                    }
                    (FieldSource::Data, FieldType::Named(name)) => self
                        .record_by_name(name)
                        .map(|record| self.default_value(Type::Record(record.id), 0))
                        .unwrap_or(ScreenValue::List(Vec::new())),
                    _ => ScreenValue::List(Vec::new()),
                })
                .collect(),
        )
    }

    fn control_type(&self, tag: &str) -> Option<&TypeDef> {
        self.screen
            .controls
            .iter()
            .find(|control| control.name == tag)
    }

    /// Chama um handler com a janela atual e guarda a janela devolvida. Em
    /// erro, o estado continua o de antes.
    fn call(&mut self, handler: &str) -> Result<String, String> {
        let mut io = BufferIo::default();
        let result = {
            let mut vm = Vm::new(&self.module, &mut io);
            if let Some(factory) = &self.host {
                vm = vm.with_host(factory());
            }
            vm.call_function(handler, vec![self.state.to_vm()], self.step_budget)
        };
        let value = result.map_err(|error| format!("Erro em `{handler}`: {}", error.message))?;
        let next = ScreenValue::from_vm(&value)?;
        if next.list().map(<[ScreenValue]>::len) != self.state.list().map(<[ScreenValue]>::len) {
            return Err(format!("`{handler}` não devolveu a janela"));
        }
        self.state = next;
        self.sync_bound();
        Ok(io.output)
    }

    /// Índices de `dados.a.b` dentro da janela: a posição de `dados` e, depois,
    /// a de cada campo.
    fn data_path(&self, path: &[String]) -> Option<Vec<usize>> {
        let slot = self
            .screen
            .window
            .fields
            .iter()
            .position(|f| f.source == FieldSource::Data)?;
        let FieldType::Named(name) = &self.screen.window.fields[slot].ty else {
            return None;
        };
        let mut ty = Type::Record(self.record_by_name(name)?.id);
        let mut indices = vec![slot];
        for field in path.iter().skip(1) {
            let record = self.record_def(ty)?;
            let index = record.fields.iter().position(|f| f.name == *field)?;
            ty = record.fields[index].ty;
            indices.push(index);
        }
        Some(indices)
    }

    fn value_at(&self, indices: &[usize]) -> Option<&ScreenValue> {
        let mut current = &self.state;
        for &index in indices {
            current = current.list()?.get(index)?;
        }
        Some(current)
    }

    fn set_at(&mut self, indices: &[usize], value: ScreenValue) {
        let mut current = &mut self.state;
        for &index in indices {
            let Some(next) = current.list_mut().and_then(|items| items.get_mut(index)) else {
                return;
            };
            current = next;
        }
        *current = value;
    }

    /// Texto de um valor simples, como `escreva` mostraria.
    fn display(value: &ScreenValue) -> String {
        match value {
            ScreenValue::Text(text) => text.clone(),
            other => other.to_vm().display(),
        }
    }

    /// Reavalia as propriedades vinculadas a partir de `dados`: texto com os
    /// valores interpolados, e o valor do campo nos outros atributos.
    fn sync_bound(&mut self) {
        for index in 0..self.bound.len() {
            let property = self.bound[index].clone();
            let value = if property.is_text() {
                let mut text = String::new();
                for part in &property.parts {
                    match part {
                        ferro_tela::value::TextPart::Literal(literal) => text.push_str(literal),
                        ferro_tela::value::TextPart::Binding { path, .. } => {
                            if let Some(value) =
                                self.data_path(path).and_then(|i| self.value_at(&i))
                            {
                                text.push_str(&Self::display(value));
                            }
                        }
                    }
                }
                ScreenValue::Text(text)
            } else {
                let Some(value) = property
                    .single_path()
                    .and_then(|path| self.data_path(path))
                    .and_then(|indices| self.value_at(&indices).cloned())
                else {
                    continue;
                };
                value
            };
            let field = ferro_tela::value::field_name(property.attribute);
            match &property.control {
                Some(control) => self.set_control_field(control, &field, value),
                None => {
                    if let Some(slot) = Self::field_index(&self.screen.window.fields, &field)
                        && let Some(target) = self.state.list_mut().and_then(|w| w.get_mut(slot))
                    {
                        *target = value;
                    }
                }
            }
        }
    }

    /// Muda uma propriedade de um controle por causa do usuário: se a
    /// marcação a vincula em mão dupla, o valor vai para `dados`.
    fn set_property(&mut self, control: &str, attribute: &str, value: ScreenValue) {
        let two_way = self
            .bound
            .iter()
            .find(|p| {
                p.control.as_deref() == Some(control) && p.attribute == attribute && p.is_two_way()
            })
            .and_then(|p| p.single_path().map(<[String]>::to_vec));
        match two_way.and_then(|path| self.data_path(&path)) {
            Some(indices) => self.set_at(&indices, value),
            None => {
                self.set_control_field(control, &ferro_tela::value::field_name(attribute), value)
            }
        }
    }

    fn field_index(fields: &[FieldDef], name: &str) -> Option<usize> {
        fields.iter().position(|field| field.name == name)
    }

    /// Posição do controle `name` entre os campos da janela.
    fn control_slot(&self, name: &str) -> Option<(usize, &TypeDef)> {
        let index = self.screen.window.fields.iter().position(|field| {
            field.name == name && matches!(field.source, FieldSource::Control(_))
        })?;
        let FieldType::Named(tag) = &self.screen.window.fields[index].ty else {
            return None;
        };
        Some((index, self.control_type(tag)?))
    }

    fn set_control_field(&mut self, control: &str, field: &str, value: ScreenValue) {
        let Some((slot, definition)) = self.control_slot(control) else {
            return;
        };
        let Some(field_index) = Self::field_index(&definition.fields, field) else {
            return;
        };
        if let Some(record) = self
            .state
            .list_mut()
            .and_then(|w| w.get_mut(slot))
            .and_then(ScreenValue::list_mut)
            && let Some(target) = record.get_mut(field_index)
        {
            *target = value;
        }
    }

    /// Aplica o que o usuário fez no controle antes de chamar a função: o
    /// texto digitado, a marcação (e desmarca os outros botões de opção do
    /// mesmo grupo) ou o item escolhido.
    fn apply_input(&mut self, element: &Element, control: &str, value: &Json) {
        match (element.name.text.as_str(), value) {
            ("CaixaTexto", Json::String(text)) => {
                self.set_property(control, "Texto", ScreenValue::Text(text.clone()))
            }
            ("CaixaSelecao", Json::Bool(checked)) => {
                self.set_property(control, "Marcado", ScreenValue::Bool(*checked))
            }
            ("BotaoOpcao", Json::Bool(checked)) => {
                if *checked {
                    let group = element
                        .attribute("Grupo")
                        .map(|a| a.value.clone())
                        .unwrap_or_default();
                    let others: Vec<String> = self
                        .document
                        .descendants()
                        .into_iter()
                        .filter(|other| other.name.text == "BotaoOpcao")
                        .filter(|other| {
                            other
                                .attribute("Grupo")
                                .map(|a| a.value.clone())
                                .unwrap_or_default()
                                == group
                        })
                        .filter_map(|other| other.control_name().map(|n| n.value.clone()))
                        .filter(|name| name != control)
                        .collect();
                    for other in others {
                        self.set_property(&other, "Marcado", ScreenValue::Bool(false));
                    }
                }
                self.set_property(control, "Marcado", ScreenValue::Bool(*checked));
            }
            ("Lista" | "Combo", Json::Number(index)) => {
                let index = index.as_i64().unwrap_or(-1).clamp(-1, i32::MAX as i64) as i32;
                self.set_property(control, "Selecionado", ScreenValue::Int(index));
            }
            _ => {}
        }
        self.sync_bound();
    }

    /// Trata um evento. Sem função ligada, só o valor digitado ou marcado
    /// muda; com erro na função, a janela continua como estava.
    pub fn dispatch(&mut self, event: &ScreenEvent) -> Result<ScreenStep, String> {
        let element = if event.control.is_empty() {
            Some(self.document.root.clone())
        } else {
            self.document.find(&event.control).cloned()
        };
        let Some(element) = element else {
            return Err(format!(
                "A tela não tem um controle chamado `{}`",
                event.control
            ));
        };
        let before = self.state.clone();
        if let Some(value) = &event.value {
            self.apply_input(&element, &event.control, value);
        }
        let handler = element
            .attribute(&event.event)
            .map(|a| a.value.trim().to_owned())
            .filter(|h| !h.is_empty());
        let output = match &handler {
            Some(name) => match self.call(name) {
                Ok(output) => output,
                Err(error) => {
                    self.state = before;
                    return Err(error);
                }
            },
            None => String::new(),
        };
        if self.state != before {
            self.history.push(self.state.clone());
            if self.history.len() > MAX_HISTORY {
                self.history.remove(0);
            }
        }
        Ok(self.step(handler, output))
    }

    /// O estado atual, sem chamar função (a janela pede ao abrir).
    pub fn current_step(&self) -> ScreenStep {
        self.step(None, String::new())
    }

    /// Estados guardados, do inicial ao atual.
    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    /// Retrato de um estado do histórico (para o depurador).
    pub fn snapshot_at(&self, index: usize) -> Option<Json> {
        self.history.get(index).map(|state| self.snapshot(state))
    }

    fn step(&self, handler: Option<String>, output: String) -> ScreenStep {
        ScreenStep {
            snapshot: self.snapshot(&self.state),
            handler,
            output,
            step: self.history.len().saturating_sub(1),
        }
    }

    fn json(&self, value: &ScreenValue, ty: Option<Type>) -> Json {
        match value {
            ScreenValue::Int(n) => json!(n),
            ScreenValue::Real(n) => json!(n),
            ScreenValue::Bool(b) => json!(b),
            ScreenValue::Char(c) => json!(c.to_string()),
            ScreenValue::Text(text) => json!(text),
            ScreenValue::List(items) => match ty.and_then(|ty| self.record_def(ty)) {
                Some(record) => Json::Object(
                    record
                        .fields
                        .iter()
                        .zip(items)
                        .map(|(field, item)| (field.name.clone(), self.json(item, Some(field.ty))))
                        .collect(),
                ),
                None => Json::Array(items.iter().map(|item| self.json(item, None)).collect()),
            },
        }
    }

    /// `{ janela: {...}, controles: { nome: {...} }, dados: {...} }`.
    fn snapshot(&self, state: &ScreenValue) -> Json {
        let values = state.list().unwrap_or_default();
        let mut window = Map::new();
        let mut controls = Map::new();
        let mut data = Json::Null;
        for (field, value) in self.screen.window.fields.iter().zip(values) {
            match (&field.source, &field.ty) {
                (FieldSource::Control(_), FieldType::Named(tag)) => {
                    let object = self.control_type(tag).map_or(Json::Null, |definition| {
                        Json::Object(
                            definition
                                .fields
                                .iter()
                                .zip(value.list().unwrap_or_default())
                                .map(|(field, value)| (field.name.clone(), self.json(value, None)))
                                .collect(),
                        )
                    });
                    controls.insert(field.name.clone(), object);
                }
                (FieldSource::Data, FieldType::Named(name)) => {
                    data = self.json(
                        value,
                        self.record_by_name(name)
                            .map(|record| Type::Record(record.id)),
                    );
                }
                _ => {
                    window.insert(field.name.clone(), self.json(value, None));
                }
            }
        }
        json!({ "janela": window, "controles": controls, "dados": data })
    }
}
