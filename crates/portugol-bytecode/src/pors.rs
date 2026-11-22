//! Compilador HIR Portugol-RS -> bytecode da VM educacional.

use portugol_hir as hir;
use portugol_syntax::Span;

use crate::op::{Function, LocalInfo, Module, Op, ValueType};

#[derive(Debug, Clone, PartialEq)]
pub struct PorsCompileError {
    pub message: String,
    pub span: hir::Span,
}

pub fn compile_pors(program: &hir::Program) -> Result<Module, PorsCompileError> {
    Compiler::compile_program(program)
}

struct LoopFrame {
    breaks: Vec<usize>,
    continues: Vec<usize>,
    break_scope_depth: usize,
    continue_scope_depth: usize,
}

struct Compiler {
    code: Vec<Op>,
    spans: Vec<Span>,
    locals: Vec<LocalInfo>,
    strings: Vec<String>,
    loops: Vec<LoopFrame>,
    scope_locals: Vec<Vec<u32>>,
    scope_shadowed: Vec<Vec<u32>>,
}

impl Compiler {
    fn new(bindings: &[hir::Binding]) -> Self {
        let locals = bindings
            .iter()
            .map(|binding| LocalInfo {
                name: binding.source_name.clone(),
                ty: classic_type(binding.ty),
                dims: 0,
                by_ref: false,
            })
            .collect();
        Self {
            code: Vec::new(),
            spans: Vec::new(),
            locals,
            strings: Vec::new(),
            loops: Vec::new(),
            scope_locals: vec![Vec::new()],
            scope_shadowed: vec![Vec::new()],
        }
    }

    fn compile_program(program: &hir::Program) -> Result<Module, PorsCompileError> {
        let mut functions = Vec::with_capacity(program.functions.len() + 1);
        let mut strings = Vec::new();
        let mut ordered: Vec<_> = program.functions.iter().collect();
        ordered.sort_by_key(|function| function.id.0);
        for function in ordered {
            let (compiled, function_strings) = Self::compile_function(
                &function.source_name,
                &function.bindings,
                &function.body,
                function
                    .return_type
                    .map(value_type_hir)
                    .unwrap_or(ValueType::Vazio),
                function.parameters.len() as u32,
                function.span,
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
        let (compiled, function_strings) = Self::compile_function(
            "inicio",
            &program.bindings,
            &program.body,
            ValueType::Vazio,
            0,
            program.span,
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
            natives: vec![],
            native_consts: vec![],
        })
    }

    fn compile_function(
        name: &str,
        bindings: &[hir::Binding],
        body: &[hir::Stmt],
        return_type: ValueType,
        arity: u32,
        span: hir::Span,
    ) -> Result<(Function, Vec<String>), PorsCompileError> {
        let mut self_ = Self::new(bindings);
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
        self_.emit(Op::PushVoid, span);
        self_.emit(Op::Return, span);
        Ok((
            Function {
                name: name.into(),
                arity,
                return_type,
                locals: self_.locals,
                code: self_.code,
                spans: self_.spans,
                span: convert_span(span),
            },
            self_.strings,
        ))
    }

    fn emit(&mut self, op: Op, span: hir::Span) -> usize {
        self.code.push(op);
        self.spans.push(convert_span(span));
        self.code.len() - 1
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
                    self.expr(expr)?;
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
                self.expr(value)?;
                if let Some(op) = assign_op(*op) {
                    self.emit(op, stmt.span);
                }
                self.emit(Op::StoreLocal(binding.0), stmt.span);
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
                for arg in arguments {
                    self.expr(arg)?;
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
            S::Return { value } => {
                if let Some(value) = value {
                    self.expr(value)?;
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
            E::Call {
                function,
                arguments,
            } => {
                for arg in arguments {
                    self.expr(arg)?;
                }
                self.emit(
                    Op::Call {
                        function: function.0,
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
                let index = self
                    .strings
                    .iter()
                    .position(|item| item == value)
                    .unwrap_or_else(|| {
                        self.strings.push(value.clone());
                        self.strings.len() - 1
                    });
                self.emit(Op::PushStr(index as u32), expr.span);
            }
            E::Parenthesized(inner) => self.expr(inner)?,
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
}

fn classic_type(ty: hir::Type) -> portugol_syntax::ast::Type {
    match ty {
        hir::Type::Unit => portugol_syntax::ast::Type::Vazio,
        hir::Type::Integer => portugol_syntax::ast::Type::Inteiro,
        hir::Type::Real => portugol_syntax::ast::Type::Real,
        hir::Type::Boolean => portugol_syntax::ast::Type::Logico,
        hir::Type::String => portugol_syntax::ast::Type::Cadeia,
        hir::Type::Character => portugol_syntax::ast::Type::Caracter,
    }
}
fn value_type_hir(ty: hir::Type) -> ValueType {
    value_type(classic_type(ty))
}
fn value_type(ty: portugol_syntax::ast::Type) -> ValueType {
    ty.into()
}
fn convert_span(span: hir::Span) -> Span {
    Span {
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
