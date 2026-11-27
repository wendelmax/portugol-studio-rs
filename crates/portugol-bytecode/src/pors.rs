//! Compilador HIR Portugol-RS -> bytecode da VM educacional.

use portugol_hir as hir;
use portugol_hir::Span;

use crate::op::{
    Function, InstructionMapping, LocalInfo, Module, NativeRef, Op, PorsDebugField, PorsDebugShape,
    PorsDebugType, PorsDebugVariant, ValueType,
};

#[derive(Debug, Clone, PartialEq)]
pub struct PorsCompileError {
    pub message: String,
    pub span: hir::Span,
}

pub fn compile_pors(program: &hir::Program) -> Result<Module, PorsCompileError> {
    compile_program(program)
}

struct LoopFrame {
    breaks: Vec<usize>,
    continues: Vec<usize>,
    break_scope_depth: usize,
    continue_scope_depth: usize,
}

struct Compiler<'a> {
    code: Vec<Op>,
    spans: Vec<Span>,
    nodes: Vec<Option<InstructionMapping>>,
    locals: Vec<LocalInfo>,
    strings: Vec<String>,
    natives: &'a mut Vec<NativeRef>,
    loops: Vec<LoopFrame>,
    scope_locals: Vec<Vec<u32>>,
    scope_shadowed: Vec<Vec<u32>>,
    function_vector_parameters: Vec<Vec<bool>>,
}

fn compile_program(program: &hir::Program) -> Result<Module, PorsCompileError> {
    let mut functions = Vec::with_capacity(program.functions.len() + 1);
    let mut strings = Vec::new();
    let mut natives = Vec::new();
    let mut ordered: Vec<_> = program.functions.iter().collect();
    ordered.sort_by_key(|function| function.id.0);
    let function_vector_parameters: Vec<Vec<bool>> = ordered
        .iter()
        .map(|function| {
            function
                .parameters
                .iter()
                .map(|id| {
                    let ty = function.bindings[id.0 as usize].ty;
                    ty.is_vector() || matches!(ty, hir::Type::Record(_) | hir::Type::Enum(_))
                })
                .collect()
        })
        .collect();
    for function in ordered {
        let (compiled, function_strings) = Compiler::compile_function(
            &function.source_name,
            &function.bindings,
            &function.body,
            function.tail_expression.as_ref(),
            function
                .return_type
                .map(value_type_hir)
                .unwrap_or(ValueType::Vazio),
            function.parameters.len() as u32,
            function.span,
            function_vector_parameters.clone(),
            program,
            &mut natives,
        )?;
        let base = strings.len() as u32;
        let mut compiled = compiled;
        for op in &mut compiled.code {
            if let Op::PushStr(index) = op {
                *index += base;
            }
        }
        strings.extend(function_strings);
        functions.push(compiled);
    }
    let entry = functions.len() as u32;
    let (compiled, function_strings) = Compiler::compile_function(
        "inicio",
        &program.bindings,
        &program.body,
        None,
        ValueType::Vazio,
        0,
        program.span,
        function_vector_parameters.clone(),
        program,
        &mut natives,
    )?;
    let base = strings.len() as u32;
    let mut compiled = compiled;
    for op in &mut compiled.code {
        if let Op::PushStr(index) = op {
            *index += base;
        }
    }
    strings.extend(function_strings);
    functions.push(compiled);
    Ok(Module {
        functions,
        entry,
        globals: vec![],
        strings,
        natives,
        native_consts: vec![],
    })
}

impl<'a> Compiler<'a> {
    fn new(
        bindings: &[hir::Binding],
        function_vector_parameters: Vec<Vec<bool>>,
        program: &hir::Program,
        natives: &'a mut Vec<NativeRef>,
    ) -> Self {
        let locals = bindings
            .iter()
            .map(|binding| LocalInfo {
                name: binding.source_name.clone(),
                ty: classic_type(binding.ty.vector_element().unwrap_or(binding.ty)),
                dims: u8::from(
                    binding.ty.is_vector()
                        || matches!(binding.ty, hir::Type::Record(_) | hir::Type::Enum(_)),
                ),
                by_ref: false,
                pors_debug: debug_type(binding.ty, program),
            })
            .collect();
        Self {
            code: Vec::new(),
            spans: Vec::new(),
            nodes: Vec::new(),
            locals,
            strings: Vec::new(),
            natives,
            loops: Vec::new(),
            scope_locals: vec![Vec::new()],
            scope_shadowed: vec![Vec::new()],
            function_vector_parameters,
        }
    }

    fn intern_native(&mut self, library: &str, name: &str) -> u32 {
        if let Some(index) = self
            .natives
            .iter()
            .position(|entry| entry.library == library && entry.name == name)
        {
            return index as u32;
        }
        let index = self.natives.len() as u32;
        self.natives.push(NativeRef {
            library: library.to_owned(),
            name: name.to_owned(),
        });
        index
    }

    #[allow(clippy::too_many_arguments)] // Metadata de depuração nominal e tabela de nativas do módulo.
    fn compile_function(
        name: &str,
        bindings: &[hir::Binding],
        body: &[hir::Stmt],
        tail_expression: Option<&hir::Expr>,
        return_type: ValueType,
        arity: u32,
        span: hir::Span,
        function_vector_parameters: Vec<Vec<bool>>,
        program: &hir::Program,
        natives: &'a mut Vec<NativeRef>,
    ) -> Result<(Function, Vec<String>), PorsCompileError> {
        let mut self_ = Self::new(bindings, function_vector_parameters, program, natives);
        for local in 0..arity {
            self_.emit(
                Op::SetLocalActive {
                    local,
                    active: true,
                },
                span,
            );
            self_.scope_locals[0].push(local);
        }
        for stmt in body {
            self_.stmt(stmt)?;
        }
        match tail_expression {
            Some(expression) => {
                self_.expr(expression)?;
                self_.emit_node(
                    Op::Return,
                    expression.span,
                    mapping("Return.tail", expression.span),
                );
            }
            None => {
                self_.emit(Op::PushVoid, span);
                self_.emit(Op::Return, span);
            }
        }
        Ok((
            Function {
                name: name.into(),
                arity,
                return_type,
                locals: self_.locals,
                code: self_.code,
                spans: self_.spans,
                nodes: self_.nodes,
                span: convert_span(span),
            },
            self_.strings,
        ))
    }

    fn emit(&mut self, op: Op, span: hir::Span) -> usize {
        self.code.push(op);
        self.spans.push(convert_span(span));
        self.nodes.push(None);
        self.code.len() - 1
    }

    fn emit_node(&mut self, op: Op, span: hir::Span, node: InstructionMapping) -> usize {
        let ip = self.emit(op, span);
        self.nodes[ip] = Some(node);
        ip
    }

    fn intern_string(&mut self, value: &str) -> u32 {
        if let Some(index) = self.strings.iter().position(|item| item == value) {
            return index as u32;
        }
        self.strings.push(value.to_owned());
        (self.strings.len() - 1) as u32
    }

    fn patch(&mut self, at: usize, target: u32) {
        match &mut self.code[at] {
            Op::Jump(t)
            | Op::JumpIfFalse(t)
            | Op::JumpIfTrue(t)
            | Op::JumpIfFalseKeep(t)
            | Op::JumpIfTrueKeep(t) => *t = target,
            _ => unreachable!("patch target must be a jump"),
        }
    }

    fn here(&self) -> u32 {
        self.code.len() as u32
    }

    fn error<T>(&self, message: impl Into<String>, span: hir::Span) -> Result<T, PorsCompileError> {
        Err(PorsCompileError {
            message: message.into(),
            span,
        })
    }

    fn stmt(&mut self, stmt: &hir::Stmt) -> Result<(), PorsCompileError> {
        use hir::StmtKind as S;
        match &stmt.kind {
            S::Declare {
                binding,
                initializer,
            } => {
                if let Some(previous) = self
                    .scope_locals
                    .iter()
                    .rev()
                    .flat_map(|scope| scope.iter().rev())
                    .copied()
                    .find(|id| {
                        self.locals[*id as usize].name == self.locals[binding.0 as usize].name
                    })
                {
                    self.emit(
                        Op::SetLocalActive {
                            local: previous,
                            active: false,
                        },
                        stmt.span,
                    );
                    self.scope_shadowed
                        .last_mut()
                        .expect("escopo ativo")
                        .push(previous);
                }
                if let Some(expr) = initializer {
                    self.expr_for_value(expr)?;
                    self.emit(Op::StoreLocal(binding.0), stmt.span);
                }
                self.emit(
                    Op::SetLocalActive {
                        local: binding.0,
                        active: true,
                    },
                    stmt.span,
                );
                self.scope_locals
                    .last_mut()
                    .expect("escopo de função")
                    .push(binding.0);
            }
            S::Assign { binding, op, value } => {
                if *op != hir::AssignOp::Set {
                    self.emit(Op::LoadLocal(binding.0), stmt.span);
                }
                if self.locals[binding.0 as usize].dims > 0 && *op == hir::AssignOp::Set {
                    self.expr_for_value(value)?;
                } else {
                    self.expr(value)?;
                }
                if let Some(op) = assign_op(*op) {
                    self.emit(op, stmt.span);
                }
                self.emit(Op::StoreLocal(binding.0), stmt.span);
            }
            S::FieldAssign {
                binding,
                fields,
                op,
                value,
            } => {
                // Registros são vetores na VM: desce até o registro que contém o
                // último campo (as cargas intermediárias são referências, então
                // a gravação altera o registro da variável).
                let (last, parents) = fields.split_last().expect("caminho de campos não vazio");
                if *op != hir::AssignOp::Set {
                    self.emit(Op::LoadLocal(binding.0), stmt.span);
                    for field in fields {
                        self.emit(Op::PushInt(*field as i32), stmt.span);
                        self.emit(Op::LoadElem, stmt.span);
                    }
                }
                self.expr_for_value(value)?;
                if let Some(opcode) = assign_op(*op) {
                    self.emit(opcode, stmt.span);
                }
                self.emit(Op::LoadLocal(binding.0), stmt.span);
                for field in parents {
                    self.emit(Op::PushInt(*field as i32), stmt.span);
                    self.emit(Op::LoadElem, stmt.span);
                }
                self.emit(Op::PushInt(*last as i32), stmt.span);
                self.emit(Op::StoreElem, stmt.span);
            }
            S::VectorPush { binding, value } => {
                self.emit(Op::LoadLocal(binding.0), stmt.span);
                self.expr_for_value(value)?;
                self.emit(Op::ArrayPush, stmt.span);
            }
            S::VectorPop { binding } => {
                self.emit(Op::LoadLocal(binding.0), stmt.span);
                self.emit(Op::ArrayPop, stmt.span);
                self.emit(Op::Pop, stmt.span);
            }
            S::Choose { value, arms } => {
                // Avalia o escrutínio exatamente uma vez (evita reexecutar chamadas
                // com efeito colateral como `remover(v)` ou `Util.sorteia(...)`).
                self.expr(value)?;
                let mut exits = Vec::new();
                for arm in arms {
                    if let Some(variant_index) = arm.variant_index {
                        self.emit(Op::Dup, arm.span);
                        self.emit(Op::PushInt(0), arm.span);
                        self.emit(Op::LoadElem, arm.span);
                        self.emit(Op::PushInt(variant_index as i32), arm.span);
                        self.emit(Op::Eq, arm.span);
                        let next_arm = self.emit(Op::JumpIfFalse(0), arm.span);
                        for (payload_index, binding) in arm.payload_bindings.iter().enumerate() {
                            self.emit(Op::Dup, arm.span);
                            self.emit(Op::PushInt((payload_index + 1) as i32), arm.span);
                            self.emit(Op::LoadElem, arm.span);
                            if self.locals[binding.0 as usize].dims > 0 {
                                self.emit(Op::CloneArray, arm.span);
                            } else if self.locals[binding.0 as usize].ty == ValueType::Cadeia {
                                self.emit(Op::CloneString, arm.span);
                            }
                            self.emit(Op::StoreLocal(binding.0), arm.span);
                        }
                        self.emit(Op::Pop, arm.span);
                        self.scope_locals
                            .push(arm.payload_bindings.iter().map(|id| id.0).collect());
                        self.scope_shadowed.push(Vec::new());
                        for binding in &arm.payload_bindings {
                            self.emit(
                                Op::SetLocalActive {
                                    local: binding.0,
                                    active: true,
                                },
                                arm.span,
                            );
                        }
                        self.block(&arm.body, arm.span)?;
                        self.finish_scope(arm.span);
                        exits.push(self.emit(Op::Jump(0), arm.span));
                        let next = self.here();
                        self.patch(next_arm, next);
                    } else {
                        // Ramo coringa (`caso _ =>`): descarta o escrutínio e executa o corpo.
                        self.emit(Op::Pop, arm.span);
                        self.scope_locals.push(Vec::new());
                        self.scope_shadowed.push(Vec::new());
                        self.block(&arm.body, arm.span)?;
                        self.finish_scope(arm.span);
                        exits.push(self.emit(Op::Jump(0), arm.span));
                    }
                }
                self.emit(Op::Pop, stmt.span);
                let end = self.here();
                for exit in exits {
                    self.patch(exit, end);
                }
            }
            S::IndexAssign {
                binding,
                index,
                value,
            } => {
                self.expr(value)?;
                self.emit(Op::LoadLocal(binding.0), stmt.span);
                self.expr(index)?;
                self.emit(Op::StoreElem, index.span);
            }
            S::Read { binding, .. } => {
                let Some(local) = self.locals.get(binding.0 as usize) else {
                    return self.error("binding de entrada inexistente", stmt.span);
                };
                self.emit(Op::Read(value_type(local.ty)), stmt.span);
                self.emit(Op::StoreLocal(binding.0), stmt.span);
            }
            S::Write(args) => {
                for arg in args {
                    self.expr(arg)?;
                }
                self.emit(Op::Write(args.len() as u32), stmt.span);
            }
            S::Call {
                function,
                arguments,
            } => {
                for (index, arg) in arguments.iter().enumerate() {
                    let is_vector_parameter = self
                        .function_vector_parameters
                        .get(function.0 as usize)
                        .and_then(|params| params.get(index))
                        .copied()
                        .unwrap_or(false);
                    if is_vector_parameter {
                        self.expr_for_value(arg)?;
                    } else {
                        self.expr(arg)?;
                    }
                }
                self.emit(
                    Op::Call {
                        function: function.0,
                        argc: arguments.len() as u32,
                    },
                    stmt.span,
                );
                self.emit(Op::Pop, stmt.span);
            }
            S::HostCall {
                module,
                name,
                arguments,
            } => {
                for arg in arguments {
                    self.expr(arg)?;
                }
                let vm_module = pors_vm_module(module);
                let native = self.intern_native(vm_module, name);
                self.emit(
                    Op::CallNative {
                        native,
                        argc: arguments.len() as u32,
                    },
                    stmt.span,
                );
                self.emit(Op::Pop, stmt.span);
            }
            S::Return { value } => {
                if let Some(value) = value {
                    if value.ty.is_vector()
                        || matches!(value.ty, hir::Type::Record(_) | hir::Type::Enum(_))
                    {
                        self.expr_for_value(value)?;
                    } else {
                        self.expr(value)?;
                    }
                } else {
                    self.emit(Op::PushVoid, stmt.span);
                }
                self.emit(Op::Return, stmt.span);
            }
            S::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.expr(condition)?;
                let branch = self.emit(Op::JumpIfFalse(0), stmt.span);
                self.block(then_branch, stmt.span)?;
                if let Some(other) = else_branch {
                    let end = self.emit(Op::Jump(0), stmt.span);
                    let else_target = self.here();
                    self.patch(branch, else_target);
                    self.block(other, stmt.span)?;
                    let end_target = self.here();
                    self.patch(end, end_target);
                } else {
                    let target = self.here();
                    self.patch(branch, target);
                }
            }
            S::While { condition, body } => {
                let start = self.here();
                self.expr(condition)?;
                let end = self.emit(Op::JumpIfFalse(0), stmt.span);
                let loop_depth = self.scope_locals.len();
                self.loops.push(LoopFrame {
                    breaks: Vec::new(),
                    continues: Vec::new(),
                    break_scope_depth: loop_depth,
                    continue_scope_depth: loop_depth,
                });
                self.block(body, stmt.span)?;
                self.emit(Op::Jump(start), stmt.span);
                let target = self.here();
                self.patch(end, target);
                self.finish_loop(target, start);
            }
            S::DoWhile { body, condition } => {
                let start = self.here();
                let loop_depth = self.scope_locals.len();
                self.loops.push(LoopFrame {
                    breaks: Vec::new(),
                    continues: Vec::new(),
                    break_scope_depth: loop_depth,
                    continue_scope_depth: loop_depth,
                });
                self.block(body, stmt.span)?;
                let cond_start = self.here();
                self.expr(condition)?;
                self.emit(Op::JumpIfTrue(start), stmt.span);
                let end = self.here();
                self.finish_loop(end, cond_start);
            }
            S::For {
                initializer,
                condition,
                step,
                body,
            } => {
                let outer_depth = self.scope_locals.len();
                self.scope_locals.push(Vec::new());
                self.scope_shadowed.push(Vec::new());
                self.stmt(initializer)?;
                let start = self.here();
                self.expr(condition)?;
                let end = self.emit(Op::JumpIfFalse(0), stmt.span);
                self.loops.push(LoopFrame {
                    breaks: Vec::new(),
                    continues: Vec::new(),
                    break_scope_depth: outer_depth,
                    continue_scope_depth: self.scope_locals.len(),
                });
                self.block(body, stmt.span)?;
                let continue_target = self.here();
                let continues =
                    std::mem::take(&mut self.loops.last_mut().expect("loop frame").continues);
                for at in continues {
                    self.patch(at, continue_target);
                }
                self.stmt(step)?;
                self.emit(Op::Jump(start), stmt.span);
                let target = self.here();
                self.patch(end, target);
                self.finish_loop(target, continue_target);
                self.finish_scope(stmt.span);
            }
            S::LoopControl { control } => {
                let Some(frame) = self.loops.last() else {
                    return self.error("controle de laço fora de laço", stmt.span);
                };
                let (break_depth, continue_depth) =
                    (frame.break_scope_depth, frame.continue_scope_depth);
                self.hide_scopes_from(
                    if matches!(control, hir::LoopControl::Break) {
                        break_depth
                    } else {
                        continue_depth
                    },
                    stmt.span,
                );
                let jump = self.emit(Op::Jump(0), stmt.span);
                let frame = self.loops.last_mut().expect("loop frame");
                match control {
                    hir::LoopControl::Break => frame.breaks.push(jump),
                    hir::LoopControl::Continue => frame.continues.push(jump),
                }
            }
        }
        Ok(())
    }

    fn finish_loop(&mut self, break_target: u32, continue_target: u32) {
        let frame = self.loops.pop().expect("loop frame");
        for at in frame.breaks {
            self.patch(at, break_target);
        }
        for at in frame.continues {
            self.patch(at, continue_target);
        }
    }

    fn block(&mut self, statements: &[hir::Stmt], span: hir::Span) -> Result<(), PorsCompileError> {
        self.scope_locals.push(Vec::new());
        self.scope_shadowed.push(Vec::new());
        for stmt in statements {
            self.stmt(stmt)?;
        }
        self.finish_scope(span);
        Ok(())
    }

    fn finish_scope(&mut self, span: hir::Span) {
        let locals = self.scope_locals.pop().expect("escopo ativo");
        let shadowed = self.scope_shadowed.pop().expect("escopo ativo");
        for local in locals.into_iter().rev() {
            self.emit(
                Op::SetLocalActive {
                    local,
                    active: false,
                },
                span,
            );
        }
        for local in shadowed.into_iter().rev() {
            self.emit(
                Op::SetLocalActive {
                    local,
                    active: true,
                },
                span,
            );
        }
    }

    fn hide_scopes_from(&mut self, depth: usize, span: hir::Span) {
        for scope_index in (depth..self.scope_locals.len()).rev() {
            let locals = self.scope_locals[scope_index].clone();
            for local in locals.into_iter().rev() {
                self.emit(
                    Op::SetLocalActive {
                        local,
                        active: false,
                    },
                    span,
                );
            }
            let shadowed = self.scope_shadowed[scope_index].clone();
            for local in shadowed.into_iter().rev() {
                self.emit(
                    Op::SetLocalActive {
                        local,
                        active: true,
                    },
                    span,
                );
            }
        }
    }

    fn expr(&mut self, expr: &hir::Expr) -> Result<(), PorsCompileError> {
        use hir::ExprKind as E;
        match &expr.kind {
            E::Binding(id) => {
                if id.0 as usize >= self.locals.len() {
                    return self.error("binding inexistente", expr.span);
                }
                self.emit(Op::LoadLocal(id.0), expr.span);
            }
            E::VectorLiteral(items) => {
                for item in items {
                    self.expr(item)?;
                }
                self.emit(Op::ArrayLit(items.len() as u32), expr.span);
            }
            E::RecordLiteral { fields, .. } => {
                for field in fields {
                    self.expr_for_value(field)?;
                }
                self.emit(Op::ArrayLit(fields.len() as u32), expr.span);
            }
            E::EnumConstructor {
                variant_index,
                payload,
                ..
            } => {
                self.emit(Op::PushInt(*variant_index as i32), expr.span);
                for item in payload {
                    self.expr_for_value(item)?;
                }
                self.emit(Op::ArrayLit(payload.len() as u32 + 1), expr.span);
            }
            E::Field { value, field_index } => {
                self.expr(value)?;
                self.emit(Op::PushInt(*field_index as i32), expr.span);
                self.emit(Op::LoadElem, expr.span);
            }
            E::Index { binding, index } => {
                self.emit(Op::LoadLocal(binding.0), expr.span);
                self.expr(index)?;
                self.emit(Op::LoadElem, expr.span);
            }
            E::Length { binding } => {
                self.emit(Op::LoadLocal(binding.0), expr.span);
                self.emit(Op::ArrayLen, expr.span);
            }
            E::VectorPop { binding } => {
                self.emit(Op::LoadLocal(binding.0), expr.span);
                self.emit(Op::ArrayPop, expr.span);
            }
            E::VectorGet { binding, index } => {
                self.emit(Op::LoadLocal(binding.0), expr.span);
                self.expr(index)?;
                self.emit(Op::ArrayGet, expr.span);
            }
            E::Call {
                function,
                arguments,
            } => {
                for (index, arg) in arguments.iter().enumerate() {
                    let is_vector_parameter = self
                        .function_vector_parameters
                        .get(function.0 as usize)
                        .and_then(|params| params.get(index))
                        .copied()
                        .unwrap_or(false);
                    if is_vector_parameter {
                        self.expr_for_value(arg)?;
                    } else {
                        self.expr(arg)?;
                    }
                }
                self.emit(
                    Op::Call {
                        function: function.0,
                        argc: arguments.len() as u32,
                    },
                    expr.span,
                );
            }
            E::HostCall {
                module,
                name,
                arguments,
            } => {
                for arg in arguments {
                    self.expr(arg)?;
                }
                let vm_module = pors_vm_module(module);
                let native = self.intern_native(vm_module, name);
                self.emit(
                    Op::CallNative {
                        native,
                        argc: arguments.len() as u32,
                    },
                    expr.span,
                );
            }
            E::Integer(value) => {
                self.emit(Op::PushInt(*value), expr.span);
            }
            E::Real(value) => {
                self.emit(Op::PushReal(*value), expr.span);
            }
            E::Boolean(value) => {
                self.emit(Op::PushBool(*value), expr.span);
            }
            E::Character(value) => {
                self.emit(Op::PushChar(*value), expr.span);
            }
            E::String(value) => {
                let index = self.intern_string(value);
                self.emit(Op::PushStr(index), expr.span);
            }
            E::Read { target } => {
                let ty = target.map(value_type_hir).unwrap_or(ValueType::Cadeia);
                self.emit_node(Op::Read(ty), expr.span, mapping("Read", expr.span));
            }
            E::InterpolatedString { parts } => {
                if parts.is_empty() {
                    let index = self.intern_string("");
                    self.emit(Op::PushStr(index), expr.span);
                }
                let mut started = false;
                for part in parts {
                    match part {
                        hir::StringPart::Literal(text, span) => {
                            let index = self.intern_string(text);
                            self.emit_node(
                                Op::PushStr(index),
                                *span,
                                mapping("InterpolatedString.Literal", *span),
                            );
                            if started {
                                self.emit_node(
                                    Op::Add,
                                    *span,
                                    mapping("InterpolatedString.Literal", *span),
                                );
                            }
                            started = true;
                        }
                        hir::StringPart::Expression(value, span) => {
                            if !started {
                                let index = self.intern_string("");
                                self.emit(Op::PushStr(index), *span);
                                started = true;
                            }
                            self.expr(value)?;
                            self.emit_node(
                                Op::Add,
                                *span,
                                mapping("InterpolatedString.Expression", *span),
                            );
                        }
                    }
                }
            }
            E::Parenthesized(inner) => self.expr(inner)?,
            E::Try { operand } => {
                self.expr(operand)?;
                self.emit(Op::Dup, expr.span);
                self.emit(Op::PushInt(0), expr.span);
                self.emit(Op::LoadElem, expr.span);
                self.emit(Op::PushInt(0), expr.span);
                self.emit(Op::Eq, expr.span);
                let is_ok = self.emit(Op::JumpIfTrue(0), expr.span);
                self.emit(Op::CloneArray, expr.span);
                self.emit(Op::Return, expr.span);
                let ok_target = self.here();
                self.patch(is_ok, ok_target);
                self.emit(Op::PushInt(1), expr.span);
                self.emit(Op::LoadElem, expr.span);
                if expr.ty == hir::Type::String {
                    self.emit(Op::CloneString, expr.span);
                } else if expr.ty.is_vector()
                    || matches!(expr.ty, hir::Type::Record(_) | hir::Type::Enum(_))
                {
                    self.emit(Op::CloneArray, expr.span);
                }
            }
            E::Unary { op, operand } => {
                self.expr(operand)?;
                let opcode = match op {
                    hir::UnaryOp::Negate => Some(Op::Neg),
                    hir::UnaryOp::Positive => None,
                    hir::UnaryOp::Not => Some(Op::Not),
                    hir::UnaryOp::BitNot => Some(Op::BitNot),
                };
                if let Some(opcode) = opcode {
                    self.emit(opcode, expr.span);
                }
            }
            E::Binary { op, left, right } => {
                if matches!(op, hir::BinaryOp::And | hir::BinaryOp::Or) {
                    self.expr(left)?;
                    let jump = self.emit(
                        if *op == hir::BinaryOp::And {
                            Op::JumpIfFalseKeep(0)
                        } else {
                            Op::JumpIfTrueKeep(0)
                        },
                        expr.span,
                    );
                    self.expr(right)?;
                    let target = self.here();
                    self.patch(jump, target);
                } else {
                    self.expr(left)?;
                    self.expr(right)?;
                    let Some(opcode) = binary_op(*op) else {
                        return self.error("operador binário não suportado", expr.span);
                    };
                    self.emit(opcode, expr.span);
                }
            }
        }
        Ok(())
    }

    fn expr_for_value(&mut self, expr: &hir::Expr) -> Result<(), PorsCompileError> {
        self.expr(expr)?;
        if (expr.ty.is_vector() || matches!(expr.ty, hir::Type::Record(_) | hir::Type::Enum(_)))
            && !matches!(
                expr.kind,
                hir::ExprKind::VectorLiteral(_)
                    | hir::ExprKind::RecordLiteral { .. }
                    | hir::ExprKind::EnumConstructor { .. }
                    | hir::ExprKind::Call { .. }
                    | hir::ExprKind::HostCall { .. }
                    | hir::ExprKind::VectorPop { .. }
                    | hir::ExprKind::VectorGet { .. }
                    | hir::ExprKind::Try { .. }
            )
        {
            self.emit(Op::CloneArray, expr.span);
        }
        Ok(())
    }
}

fn pors_vm_module(module: &str) -> &str {
    match module {
        "Matematica" => "PorsMatematica",
        "Texto" => "PorsTexto",
        "Tipos" => "PorsTipos",
        "Util" => "PorsUtil",
        "Calendario" => "PorsCalendario",
        "Arquivos" => "PorsArquivos",
        "Json" => "PorsJson",
        "Internet" => "PorsInternet",
        "Graficos" => "PorsGraficos",
        "Teclado" => "PorsTeclado",
        "Mouse" => "PorsMouse",
        "Sons" => "PorsSons",
        other => other,
    }
}

fn classic_type(ty: hir::Type) -> ValueType {
    match ty {
        hir::Type::Unit => ValueType::Vazio,
        hir::Type::Integer => ValueType::Inteiro,
        hir::Type::Real => ValueType::Real,
        hir::Type::Boolean => ValueType::Logico,
        hir::Type::String => ValueType::Cadeia,
        hir::Type::Character => ValueType::Caracter,
        hir::Type::VectorInteger
        | hir::Type::VectorReal
        | hir::Type::VectorBoolean
        | hir::Type::VectorString
        | hir::Type::VectorCharacter
        | hir::Type::VectorRecord(_)
        | hir::Type::VectorEnum(_) => {
            unreachable!("tipo vetor precisa ser reduzido ao tipo do elemento")
        }
        hir::Type::Record(_) | hir::Type::Enum(_) => ValueType::Vazio,
    }
}

fn debug_type(ty: hir::Type, program: &hir::Program) -> Option<PorsDebugType> {
    let (element, is_vector) = match ty {
        hir::Type::VectorRecord(id) => (hir::Type::Record(id), true),
        hir::Type::VectorEnum(id) => (hir::Type::Enum(id), true),
        hir::Type::Record(_) | hir::Type::Enum(_) => (ty, false),
        _ => return None,
    };
    match element {
        hir::Type::Record(id) => {
            let definition = program.record_types.iter().find(|item| item.id == id)?;
            Some(PorsDebugType {
                name: definition.name.clone(),
                is_vector,
                shape: PorsDebugShape::Record {
                    fields: definition
                        .fields
                        .iter()
                        .map(|field| PorsDebugField {
                            name: field.name.clone(),
                            ty: debug_type_label(field.ty, program),
                            nested: debug_type(field.ty, program).map(Box::new),
                        })
                        .collect(),
                },
            })
        }
        hir::Type::Enum(id) => {
            let definition = program.enum_types.iter().find(|item| item.id == id)?;
            Some(PorsDebugType {
                name: definition.name.clone(),
                is_vector,
                shape: PorsDebugShape::Enum {
                    variants: definition
                        .variants
                        .iter()
                        .map(|variant| PorsDebugVariant {
                            name: variant.name.clone(),
                            payload: variant
                                .payload_types
                                .iter()
                                .enumerate()
                                .map(|(index, ty)| PorsDebugField {
                                    name: index.to_string(),
                                    ty: debug_type_label(*ty, program),
                                    nested: debug_type(*ty, program).map(Box::new),
                                })
                                .collect(),
                        })
                        .collect(),
                },
            })
        }
        _ => None,
    }
}

fn debug_type_label(ty: hir::Type, program: &hir::Program) -> String {
    match ty {
        hir::Type::Unit => "vazio".into(),
        hir::Type::Integer => "inteiro".into(),
        hir::Type::Real => "real".into(),
        hir::Type::Boolean => "logico".into(),
        hir::Type::String => "cadeia".into(),
        hir::Type::Character => "caracter".into(),
        hir::Type::VectorInteger => "vetor<inteiro>".into(),
        hir::Type::VectorReal => "vetor<real>".into(),
        hir::Type::VectorBoolean => "vetor<logico>".into(),
        hir::Type::VectorString => "vetor<cadeia>".into(),
        hir::Type::VectorCharacter => "vetor<caracter>".into(),
        hir::Type::Record(id) => program
            .record_types
            .iter()
            .find(|item| item.id == id)
            .map(|item| item.name.clone())
            .unwrap_or_else(|| "registro".into()),
        hir::Type::VectorRecord(id) => program
            .record_types
            .iter()
            .find(|item| item.id == id)
            .map(|item| format!("vetor<{}>", item.name))
            .unwrap_or_else(|| "vetor<registro>".into()),
        hir::Type::Enum(id) => program
            .enum_types
            .iter()
            .find(|item| item.id == id)
            .map(|item| match item.builtin {
                Some(hir::BuiltinEnum::Option(inner)) => {
                    format!("Opcao<{}>", debug_type_label(inner, program))
                }
                Some(hir::BuiltinEnum::Result { ok, err }) => format!(
                    "Resultado<{}, {}>",
                    debug_type_label(ok, program),
                    debug_type_label(err, program)
                ),
                None => item.name.clone(),
            })
            .unwrap_or_else(|| "enum".into()),
        hir::Type::VectorEnum(id) => {
            format!("vetor<{}>", debug_type_label(hir::Type::Enum(id), program))
        }
    }
}

fn value_type_hir(ty: hir::Type) -> ValueType {
    classic_type(ty.vector_element().unwrap_or(ty))
}
fn value_type(ty: ValueType) -> ValueType {
    ty
}
fn convert_span(span: hir::Span) -> Span {
    span
}
fn mapping(node: &'static str, span: hir::Span) -> InstructionMapping {
    InstructionMapping {
        node,
        start: span.start,
        end: span.end,
    }
}
fn assign_op(op: hir::AssignOp) -> Option<Op> {
    match op {
        hir::AssignOp::Set => None,
        hir::AssignOp::Add => Some(Op::Add),
        hir::AssignOp::Subtract => Some(Op::Sub),
        hir::AssignOp::Multiply => Some(Op::Mul),
        hir::AssignOp::Divide => Some(Op::Div),
        hir::AssignOp::Remainder => Some(Op::Mod),
        hir::AssignOp::Xor => Some(Op::BitXor),
        hir::AssignOp::BitAnd => Some(Op::BitAnd),
        hir::AssignOp::BitOr => Some(Op::BitOr),
    }
}
fn binary_op(op: hir::BinaryOp) -> Option<Op> {
    Some(match op {
        hir::BinaryOp::Add => Op::Add,
        hir::BinaryOp::Subtract => Op::Sub,
        hir::BinaryOp::Multiply => Op::Mul,
        hir::BinaryOp::Divide => Op::Div,
        hir::BinaryOp::Remainder => Op::Mod,
        hir::BinaryOp::And | hir::BinaryOp::Or => return None,
        hir::BinaryOp::Xor => Op::BitXor,
        hir::BinaryOp::Equal => Op::Eq,
        hir::BinaryOp::NotEqual => Op::Ne,
        hir::BinaryOp::Greater => Op::Gt,
        hir::BinaryOp::Less => Op::Lt,
        hir::BinaryOp::LessEqual => Op::Le,
        hir::BinaryOp::GreaterEqual => Op::Ge,
        hir::BinaryOp::ShiftLeft => Op::Shl,
        hir::BinaryOp::ShiftRight => Op::Shr,
        hir::BinaryOp::BitAnd => Op::BitAnd,
        hir::BinaryOp::BitOr => Op::BitOr,
    })
}
