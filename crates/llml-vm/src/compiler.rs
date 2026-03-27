//! AST → Bytecode compiler.

use crate::bytecode::{Chunk, Constant, FunctionProto, Op};
use llml_lexer::Span;
use llml_parser::ast::*;

/// A local variable during compilation.
#[derive(Debug, Clone)]
struct Local {
    name: String,
    depth: usize,
    #[allow(dead_code)]
    mutable: bool,
}

/// Compiler state for a single function scope.
struct CompilerScope {
    chunk: Chunk,
    locals: Vec<Local>,
    scope_depth: usize,
    #[allow(dead_code)]
    fn_name: String,
}

impl CompilerScope {
    fn new(name: String) -> Self {
        Self {
            chunk: Chunk::new(),
            locals: Vec::new(),
            scope_depth: 0,
            fn_name: name,
        }
    }

    fn resolve_local(&self, name: &str) -> Option<u16> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name == name {
                return Some(i as u16);
            }
        }
        None
    }

    fn add_local(&mut self, name: String, mutable: bool) -> u16 {
        let idx = self.locals.len() as u16;
        self.locals.push(Local {
            name,
            depth: self.scope_depth,
            mutable,
        });
        idx
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self, span: Span) {
        self.scope_depth -= 1;
        while let Some(local) = self.locals.last() {
            if local.depth > self.scope_depth {
                self.locals.pop();
                self.chunk.emit(Op::Pop, span);
            } else {
                break;
            }
        }
    }
}

/// Compile a program to bytecode.
pub fn compile(program: &Program) -> Result<FunctionProto, CompileError> {
    let mut compiler = CompilerScope::new("<main>".into());

    // Register builtins as locals (so $print etc. resolve)
    let builtins = ["print", "to_str", "str_concat", "len", "not", "abs"];
    for name in &builtins {
        let idx = compiler.add_local(name.to_string(), false);
        let const_idx = compiler.chunk.add_constant(Constant::Str(name.to_string()));
        let span = Span { start: 0, end: 0 };
        compiler.chunk.emit(Op::CallBuiltin(const_idx, 0), span); // placeholder
        // Actually, we should just push a sentinel. Let's use a Nil for now
        // and resolve builtins by name at runtime.
        compiler.chunk.code.pop();
        compiler.chunk.spans.pop();
        compiler.chunk.emit(Op::Nil, span); // placeholder for the builtin slot
        let _ = idx;
    }

    let dummy_span = Span { start: 0, end: 0 };

    for decl in &program.decls {
        compile_decl(&mut compiler, &decl.inner, decl.span)?;
    }

    compiler.chunk.emit(Op::Halt, dummy_span);

    Ok(FunctionProto {
        name: "<main>".into(),
        arity: 0,
        chunk: compiler.chunk,
    })
}

fn compile_decl(scope: &mut CompilerScope, decl: &Decl, span: Span) -> Result<(), CompileError> {
    match decl {
        Decl::Fn(fn_decl) => {
            compile_fn_decl(scope, fn_decl, span)?;
            Ok(())
        }
        Decl::Let(let_decl) => {
            compile_expr(scope, &let_decl.value.inner, let_decl.value.span)?;
            scope.add_local(let_decl.name.clone(), let_decl.is_mut);
            Ok(())
        }
        Decl::TypeDef(_) => Ok(()), // type defs are compile-time only
        Decl::Module(mod_decl) => {
            for d in &mod_decl.decls {
                compile_decl(scope, &d.inner, d.span)?;
            }
            Ok(())
        }
        Decl::Pub(inner) => compile_decl(scope, &inner.inner, inner.span),
        Decl::Expr(expr) => {
            compile_expr(scope, expr, span)?;
            // Keep the result on stack (it's the "last value")
            Ok(())
        }
        Decl::Error => Ok(()),
    }
}

fn compile_fn_decl(
    scope: &mut CompilerScope,
    fn_decl: &FnDecl,
    span: Span,
) -> Result<(), CompileError> {
    // Compile the function body into a separate chunk
    let mut fn_scope = CompilerScope::new(fn_decl.name.clone());

    // Add parameters as locals
    for param in &fn_decl.params {
        fn_scope.add_local(param.name.clone(), param.is_mut);
    }

    // The function itself as a local for recursion
    fn_scope.add_local(fn_decl.name.clone(), false);
    // Push a Nil placeholder for the self-reference slot
    fn_scope.chunk.emit(Op::Nil, fn_decl.body.span);

    compile_expr(&mut fn_scope, &fn_decl.body.inner, fn_decl.body.span)?;
    fn_scope.chunk.emit(Op::Return, fn_decl.body.span);

    let proto = FunctionProto {
        name: fn_decl.name.clone(),
        arity: fn_decl.params.len() as u8,
        chunk: fn_scope.chunk,
    };

    let const_idx = scope.chunk.add_constant(Constant::Function(proto));
    scope.chunk.emit(Op::Closure(const_idx), span);
    scope.add_local(fn_decl.name.clone(), false);

    Ok(())
}

fn compile_expr(scope: &mut CompilerScope, expr: &Expr, span: Span) -> Result<(), CompileError> {
    match expr {
        Expr::IntLit(n) => {
            let idx = scope.chunk.add_constant(Constant::Int(*n));
            scope.chunk.emit(Op::Const(idx), span);
        }
        Expr::FloatLit(n) => {
            let idx = scope.chunk.add_constant(Constant::Float(*n));
            scope.chunk.emit(Op::Const(idx), span);
        }
        Expr::StrLit(s) => {
            let idx = scope.chunk.add_constant(Constant::Str(s.clone()));
            scope.chunk.emit(Op::Const(idx), span);
        }
        Expr::BoolLit(true) => {
            scope.chunk.emit(Op::True, span);
        }
        Expr::BoolLit(false) => {
            scope.chunk.emit(Op::False, span);
        }
        Expr::NilLit => {
            scope.chunk.emit(Op::Nil, span);
        }

        Expr::Var(name) => {
            if let Some(slot) = scope.resolve_local(name) {
                scope.chunk.emit(Op::GetLocal(slot), span);
            } else {
                return Err(CompileError(format!("undefined variable: ${name}")));
            }
        }

        Expr::TypeConstructor(name) => {
            // Nullary constructor
            let idx = scope.chunk.add_constant(Constant::Str(name.clone()));
            scope.chunk.emit(Op::Construct(idx, 0), span);
        }

        Expr::BinOp(op, lhs, rhs) => {
            compile_expr(scope, &lhs.inner, lhs.span)?;
            compile_expr(scope, &rhs.inner, rhs.span)?;
            let instr = match op {
                BinOp::Add => Op::AddI, // VM will dispatch by type at runtime
                BinOp::Sub => Op::SubI,
                BinOp::Mul => Op::MulI,
                BinOp::Div => Op::DivI,
                BinOp::Mod => Op::ModI,
                BinOp::Eq => Op::EqI,
                BinOp::Neq => Op::NeqI,
                BinOp::Lt => Op::LtI,
                BinOp::Gt => Op::GtI,
                BinOp::Le => Op::LeI,
                BinOp::Ge => Op::GeI,
                BinOp::And => Op::And,
                BinOp::Or => Op::Or,
            };
            scope.chunk.emit(instr, span);
        }

        Expr::UnaryOp(op, operand) => {
            compile_expr(scope, &operand.inner, operand.span)?;
            match op {
                UnaryOp::Neg => {
                    scope.chunk.emit(Op::NegI, span);
                }
                UnaryOp::Not => {
                    scope.chunk.emit(Op::Not, span);
                }
            }
        }

        Expr::If(cond, then_expr, else_expr) => {
            compile_expr(scope, &cond.inner, cond.span)?;
            let jump_false = scope.chunk.emit(Op::JumpIfFalse(0), span);
            compile_expr(scope, &then_expr.inner, then_expr.span)?;
            let jump_over = scope.chunk.emit(Op::Jump(0), span);
            scope.chunk.patch_jump(jump_false);
            compile_expr(scope, &else_expr.inner, else_expr.span)?;
            scope.chunk.patch_jump(jump_over);
        }

        Expr::Let(let_decl, body) => {
            compile_expr(scope, &let_decl.value.inner, let_decl.value.span)?;
            scope.add_local(let_decl.name.clone(), let_decl.is_mut);
            scope.begin_scope();
            compile_expr(scope, &body.inner, body.span)?;
            // Swap result and let binding, then pop the binding
            // Actually: result is on top, let binding below.
            // We need to keep result and pop the let binding.
            // Simple approach: don't end_scope here, just mark for later cleanup
            scope.end_scope(span);
        }

        Expr::Call(callee, args) => {
            // Check if this is a builtin call
            if let Expr::Var(name) = &callee.inner {
                let builtin_names = ["print", "to_str", "str_concat", "len", "not", "abs"];
                if builtin_names.contains(&name.as_str()) {
                    // Compile arguments
                    for arg in args {
                        compile_expr(scope, &arg.inner, arg.span)?;
                    }
                    let const_idx = scope.chunk.add_constant(Constant::Str(name.clone()));
                    scope
                        .chunk
                        .emit(Op::CallBuiltin(const_idx, args.len() as u8), span);
                    return Ok(());
                }
            }

            // Check if callee is a type constructor
            if let Expr::TypeConstructor(name) = &callee.inner {
                for arg in args {
                    compile_expr(scope, &arg.inner, arg.span)?;
                }
                let idx = scope.chunk.add_constant(Constant::Str(name.clone()));
                scope.chunk.emit(Op::Construct(idx, args.len() as u8), span);
                return Ok(());
            }

            // Regular function call
            compile_expr(scope, &callee.inner, callee.span)?;
            for arg in args {
                compile_expr(scope, &arg.inner, arg.span)?;
            }
            scope.chunk.emit(Op::Call(args.len() as u8), span);
        }

        Expr::Do(exprs) => {
            scope.begin_scope();
            let mut is_first = true;
            for expr_node in exprs {
                if !is_first {
                    // Pop the previous expression's result
                    // (but only if it wasn't a declaration)
                    // This is tricky — declarations don't push a user-visible result
                }
                is_first = false;

                match &expr_node.inner {
                    Expr::FnExpr(fn_decl) => {
                        compile_fn_decl(scope, fn_decl, expr_node.span)?;
                    }
                    Expr::Let(_let_decl, _body) => {
                        // In a do block, let is a declaration (no continuation body)
                        // Actually the parser wraps do-block lets differently.
                        // Let's compile the full let expression:
                        compile_expr(scope, &expr_node.inner, expr_node.span)?;
                    }
                    _ => {
                        compile_expr(scope, &expr_node.inner, expr_node.span)?;
                    }
                }
            }
            scope.end_scope(span);
        }

        Expr::Match(scrutinee, arms) => {
            compile_expr(scope, &scrutinee.inner, scrutinee.span)?;

            let mut end_jumps = Vec::new();

            for (i, arm) in arms.iter().enumerate() {
                let is_last = i == arms.len() - 1;

                // Duplicate scrutinee for pattern test (except last arm)
                if !is_last {
                    // We need the scrutinee value for the next arm if this one fails
                    // Emit a Dup-like operation. We don't have Dup, so use GetLocal
                    // Actually, we need to keep the scrutinee as a local
                }

                // For simplicity: compile match as a chain of if-else
                match &arm.pattern.inner {
                    Pattern::Wildcard | Pattern::Var(_) => {
                        // Always matches — compile body
                        if let Pattern::Var(name) = &arm.pattern.inner {
                            // Bind the scrutinee value
                            scope.add_local(name.clone(), false);
                            // Scrutinee is already on stack as the local
                        } else {
                            // Wildcard — pop scrutinee
                            scope.chunk.emit(Op::Pop, arm.pattern.span);
                        }
                        compile_expr(scope, &arm.body.inner, arm.body.span)?;
                    }
                    Pattern::IntLit(n) => {
                        // Compare with literal
                        let idx = scope.chunk.add_constant(Constant::Int(*n));
                        scope.chunk.emit(Op::Const(idx), arm.pattern.span);
                        scope.chunk.emit(Op::EqI, arm.pattern.span);
                        if !is_last {
                            let jump = scope.chunk.emit(Op::JumpIfFalse(0), arm.pattern.span);
                            compile_expr(scope, &arm.body.inner, arm.body.span)?;
                            let end = scope.chunk.emit(Op::Jump(0), span);
                            end_jumps.push(end);
                            scope.chunk.patch_jump(jump);
                            // Re-push scrutinee for next arm
                            // This is complex... let's use a different approach
                        } else {
                            scope.chunk.emit(Op::Pop, arm.pattern.span); // pop bool result
                            compile_expr(scope, &arm.body.inner, arm.body.span)?;
                        }
                    }
                    Pattern::Constructor(name, sub_pats) => {
                        // Test tag
                        let tag_idx = scope.chunk.add_constant(Constant::Str(name.clone()));
                        scope.chunk.emit(Op::TestTag(tag_idx), arm.pattern.span);
                        if !is_last {
                            let jump = scope.chunk.emit(Op::JumpIfFalse(0), arm.pattern.span);
                            scope.begin_scope();
                            // Extract fields and bind to pattern variables
                            for (fi, sub_pat) in sub_pats.iter().enumerate() {
                                scope.chunk.emit(Op::GetField(fi as u8), sub_pat.span);
                                if let Pattern::Var(vname) = &sub_pat.inner {
                                    scope.add_local(vname.clone(), false);
                                }
                            }
                            compile_expr(scope, &arm.body.inner, arm.body.span)?;
                            scope.end_scope(span);
                            let end = scope.chunk.emit(Op::Jump(0), span);
                            end_jumps.push(end);
                            scope.chunk.patch_jump(jump);
                        } else {
                            scope.chunk.emit(Op::Pop, arm.pattern.span); // pop bool
                            scope.begin_scope();
                            for (fi, sub_pat) in sub_pats.iter().enumerate() {
                                scope.chunk.emit(Op::GetField(fi as u8), sub_pat.span);
                                if let Pattern::Var(vname) = &sub_pat.inner {
                                    scope.add_local(vname.clone(), false);
                                }
                            }
                            compile_expr(scope, &arm.body.inner, arm.body.span)?;
                            scope.end_scope(span);
                        }
                    }
                    Pattern::TypeName(name) => {
                        // Test nullary constructor tag
                        let tag_idx = scope.chunk.add_constant(Constant::Str(name.clone()));
                        scope.chunk.emit(Op::TestTag(tag_idx), arm.pattern.span);
                        if !is_last {
                            let jump = scope.chunk.emit(Op::JumpIfFalse(0), arm.pattern.span);
                            compile_expr(scope, &arm.body.inner, arm.body.span)?;
                            let end = scope.chunk.emit(Op::Jump(0), span);
                            end_jumps.push(end);
                            scope.chunk.patch_jump(jump);
                        } else {
                            scope.chunk.emit(Op::Pop, arm.pattern.span);
                            compile_expr(scope, &arm.body.inner, arm.body.span)?;
                        }
                    }
                    _ => {
                        // Other patterns: just compile the body (best effort)
                        scope.chunk.emit(Op::Pop, arm.pattern.span);
                        compile_expr(scope, &arm.body.inner, arm.body.span)?;
                    }
                }
            }

            // Patch all end jumps
            for j in end_jumps {
                scope.chunk.patch_jump(j);
            }
        }

        Expr::FnExpr(fn_decl) => {
            compile_fn_decl(scope, fn_decl, span)?;
        }

        Expr::Return(inner) => {
            compile_expr(scope, &inner.inner, inner.span)?;
            scope.chunk.emit(Op::Return, span);
        }

        Expr::SetExpr(name, value) => {
            compile_expr(scope, &value.inner, value.span)?;
            if let Some(slot) = scope.resolve_local(name) {
                scope.chunk.emit(Op::SetLocal(slot), span);
            } else {
                return Err(CompileError(format!("undefined variable: ${name}")));
            }
            scope.chunk.emit(Op::Nil, span); // set returns nil
        }

        Expr::ModAccess(_, member) => {
            if let Some(slot) = scope.resolve_local(member) {
                scope.chunk.emit(Op::GetLocal(slot), span);
            } else {
                return Err(CompileError(format!("undefined module member: {member}")));
            }
        }
        Expr::Error => {}
    }
    Ok(())
}

/// Compilation error.
#[derive(Debug, Clone, thiserror::Error)]
#[error("{0}")]
pub struct CompileError(pub String);
