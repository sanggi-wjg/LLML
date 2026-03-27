//! Main type checking logic — two-pass approach.
//!
//! Pass 1: Collect all type definitions and function signatures.
//! Pass 2: Type-check each declaration body.

use std::collections::HashMap;

use crate::context::{TypeContext, TypeDef as CtxTypeDef, VariantDef};
use crate::errors::TypeErrorKind;
use crate::types::{FloatSize, FnType, IntSize, Type};
use llml_lexer::Span;
use llml_parser::ast::*;

/// Type-check a program.
///
/// Returns `Ok(())` if no errors, or the collected errors.
pub fn check_program(ctx: &mut TypeContext, program: &Program) {
    // Register builtins in the type context
    register_builtins(ctx);

    // Pass 1: Collect type definitions and function signatures
    for decl in &program.decls {
        collect_decl(ctx, &decl.inner);
    }

    // Pass 2: Type-check each declaration body
    for decl in &program.decls {
        check_decl(ctx, &decl.inner, decl.span);
    }
}

fn register_builtins(ctx: &mut TypeContext) {
    // $print : (: ^T -> @Nil)
    let print_var = ctx.fresh_var();
    ctx.define_var(
        "print".into(),
        Type::Fn(FnType {
            params: vec![print_var],
            ret: Box::new(Type::Nil),
            effects: vec!["io".into()],
        }),
        false,
    );
    // $to_str : (: ^T -> @Str)
    let to_str_var = ctx.fresh_var();
    ctx.define_var(
        "to_str".into(),
        Type::Fn(FnType {
            params: vec![to_str_var],
            ret: Box::new(Type::Str),
            effects: vec![],
        }),
        false,
    );
    // $str_concat : (: @Str @Str -> @Str)
    ctx.define_var(
        "str_concat".into(),
        Type::Fn(FnType {
            params: vec![Type::Str, Type::Str],
            ret: Box::new(Type::Str),
            effects: vec![],
        }),
        false,
    );
    // $len : (: @Str -> @I64)
    ctx.define_var(
        "len".into(),
        Type::Fn(FnType {
            params: vec![Type::Str],
            ret: Box::new(Type::Int(IntSize::I64)),
            effects: vec![],
        }),
        false,
    );
    // $not : (: @Bool -> @Bool)
    ctx.define_var(
        "not".into(),
        Type::Fn(FnType {
            params: vec![Type::Bool],
            ret: Box::new(Type::Bool),
            effects: vec![],
        }),
        false,
    );
    // $abs : (: ^T -> ^T) where T is numeric
    let abs_var = ctx.fresh_var();
    ctx.define_var(
        "abs".into(),
        Type::Fn(FnType {
            params: vec![abs_var.clone()],
            ret: Box::new(abs_var),
            effects: vec![],
        }),
        false,
    );
}

// ── Pass 1: Collection ──

fn collect_decl(ctx: &mut TypeContext, decl: &Decl) {
    match decl {
        Decl::Fn(fn_decl) => collect_fn(ctx, fn_decl),
        Decl::TypeDef(type_def) => collect_type_def(ctx, type_def),
        Decl::Module(mod_decl) => {
            for d in &mod_decl.decls {
                collect_decl(ctx, &d.inner);
            }
        }
        Decl::Pub(inner) => collect_decl(ctx, &inner.inner),
        Decl::Let(_) | Decl::Expr(_) | Decl::Error => {}
    }
}

fn collect_fn(ctx: &mut TypeContext, fn_decl: &FnDecl) {
    let generics = HashMap::new();
    if let Some(sig) = &fn_decl.type_sig {
        let fn_type = resolve_fn_sig(ctx, sig, &generics);
        ctx.define_var(fn_decl.name.clone(), fn_type, false);
    } else {
        // No type signature — infer from param annotations
        let param_types: Vec<Type> = fn_decl
            .params
            .iter()
            .map(|p| ctx.resolve_type_expr(&p.ty.inner, &generics))
            .collect();
        let ret_type = ctx.fresh_var();
        ctx.define_var(
            fn_decl.name.clone(),
            Type::Fn(FnType {
                params: param_types,
                ret: Box::new(ret_type),
                effects: vec![],
            }),
            false,
        );
    }
}

fn resolve_fn_sig(ctx: &mut TypeContext, sig: &TypeSig, generics: &HashMap<String, Type>) -> Type {
    let param_types: Vec<Type> = sig
        .param_types
        .iter()
        .map(|p| ctx.resolve_type_expr(&p.inner, generics))
        .collect();
    let ret_type = ctx.resolve_type_expr(&sig.return_type.inner, generics);
    let effects: Vec<String> = sig.effects.clone();
    Type::Fn(FnType {
        params: param_types,
        ret: Box::new(ret_type),
        effects,
    })
}

fn collect_type_def(ctx: &mut TypeContext, td: &llml_parser::ast::TypeDef) {
    let generics: Vec<String> = td.params.clone();
    let mut generic_map = HashMap::new();
    for g in &generics {
        generic_map.insert(g.clone(), ctx.fresh_var());
    }

    let (variants, is_sum) = match &td.body {
        TypeDefBody::Sum(variants) => {
            let vs: Vec<VariantDef> = variants
                .iter()
                .map(|v| {
                    let fields: Vec<(String, Type)> = v
                        .fields
                        .iter()
                        .map(|f| {
                            let ty = ctx.resolve_type_expr(&f.ty.inner, &generic_map);
                            (f.name.clone(), ty)
                        })
                        .collect();
                    VariantDef {
                        name: v.name.clone(),
                        fields,
                    }
                })
                .collect();
            (vs, true)
        }
        TypeDefBody::Prod(fields) => {
            let fs: Vec<(String, Type)> = fields
                .iter()
                .map(|f| {
                    let ty = ctx.resolve_type_expr(&f.ty.inner, &generic_map);
                    (f.name.clone(), ty)
                })
                .collect();
            (
                vec![VariantDef {
                    name: td.name.clone(),
                    fields: fs,
                }],
                false,
            )
        }
        TypeDefBody::Linear => {
            // Linear marker — no variants
            (vec![], false)
        }
    };

    ctx.define_type(CtxTypeDef {
        name: td.name.clone(),
        generics,
        variants,
        is_sum,
    });
}

// ── Pass 2: Checking ──

fn check_decl(ctx: &mut TypeContext, decl: &Decl, span: Span) {
    match decl {
        Decl::Fn(fn_decl) => check_fn_body(ctx, fn_decl, span),
        Decl::Let(let_decl) => check_let(ctx, let_decl, span),
        Decl::TypeDef(_) => {} // Already collected
        Decl::Module(mod_decl) => {
            for d in &mod_decl.decls {
                check_decl(ctx, &d.inner, d.span);
            }
        }
        Decl::Pub(inner) => check_decl(ctx, &inner.inner, inner.span),
        Decl::Expr(expr) => {
            infer_expr(ctx, expr, span);
        }
        Decl::Error => {}
    }
}

fn check_fn_body(ctx: &mut TypeContext, fn_decl: &FnDecl, span: Span) {
    let generics = HashMap::new();

    // Get the expected return type from signature
    let expected_ret = fn_decl.type_sig.as_ref().map(|sig| {
        let sig_type = resolve_fn_sig(ctx, sig, &generics);
        if let Type::Fn(ft) = sig_type {
            *ft.ret
        } else {
            Type::Error
        }
    });

    ctx.push_scope();

    // Bind parameters
    for param in &fn_decl.params {
        let param_type = ctx.resolve_type_expr(&param.ty.inner, &generics);
        ctx.define_var(param.name.clone(), param_type, param.is_mut);
    }

    // Check body
    let body_type = infer_expr(ctx, &fn_decl.body.inner, fn_decl.body.span);

    // Verify return type matches
    if let Some(expected) = expected_ret {
        ctx.unify(&expected, &body_type, span);
    }

    ctx.pop_scope();
}

fn check_let(ctx: &mut TypeContext, let_decl: &LetDecl, span: Span) {
    let generics = HashMap::new();
    let value_type = infer_expr(ctx, &let_decl.value.inner, let_decl.value.span);

    if let Some(ref te) = let_decl.ty {
        let declared_type = ctx.resolve_type_expr(&te.inner, &generics);
        ctx.unify(&declared_type, &value_type, span);
        ctx.define_var(let_decl.name.clone(), declared_type, let_decl.is_mut);
    } else {
        ctx.define_var(let_decl.name.clone(), value_type, let_decl.is_mut);
    }
}

/// Infer the type of an expression.
pub fn infer_expr(ctx: &mut TypeContext, expr: &Expr, span: Span) -> Type {
    match expr {
        Expr::IntLit(_) => Type::Int(IntSize::I32),
        Expr::FloatLit(_) => Type::Float(FloatSize::F64),
        Expr::StrLit(_) => Type::Str,
        Expr::BoolLit(_) => Type::Bool,
        Expr::NilLit => Type::Nil,

        Expr::Var(name) => {
            if let Some(binding) = ctx.get_var(name) {
                binding.ty.clone()
            } else {
                ctx.error(TypeErrorKind::UndefinedVar(name.clone()), span);
                Type::Error
            }
        }

        Expr::TypeConstructor(name) => {
            // Nullary constructor
            if let Some((type_name, fields)) = ctx.get_constructor(name) {
                if fields.is_empty() {
                    Type::Adt(type_name.clone(), vec![])
                } else {
                    // Constructor needs arguments — it's a function
                    let type_name = type_name.clone();
                    let fields = fields.clone();
                    Type::Fn(FnType {
                        params: fields,
                        ret: Box::new(Type::Adt(type_name, vec![])),
                        effects: vec![],
                    })
                }
            } else {
                // Unknown constructor — might be used without type def
                Type::Adt(name.clone(), vec![])
            }
        }

        Expr::BinOp(op, lhs, rhs) => {
            let lhs_ty = infer_expr(ctx, &lhs.inner, lhs.span);
            let rhs_ty = infer_expr(ctx, &rhs.inner, rhs.span);
            infer_binop(ctx, *op, &lhs_ty, &rhs_ty, span)
        }

        Expr::UnaryOp(op, operand) => {
            let operand_ty = infer_expr(ctx, &operand.inner, operand.span);
            infer_unaryop(ctx, *op, &operand_ty, span)
        }

        Expr::If(cond, then_expr, else_expr) => {
            let cond_ty = infer_expr(ctx, &cond.inner, cond.span);
            ctx.unify(&Type::Bool, &cond_ty, cond.span);

            let then_ty = infer_expr(ctx, &then_expr.inner, then_expr.span);
            let else_ty = infer_expr(ctx, &else_expr.inner, else_expr.span);
            ctx.unify(&then_ty, &else_ty, span);
            then_ty
        }

        Expr::Let(let_decl, body) => {
            check_let(ctx, let_decl, span);
            infer_expr(ctx, &body.inner, body.span)
        }

        Expr::Call(callee, args) => {
            let callee_ty = infer_expr(ctx, &callee.inner, callee.span);
            let callee_ty = ctx.resolve(&callee_ty);

            match callee_ty {
                Type::Fn(ft) => {
                    if ft.params.len() != args.len() {
                        let name = if let Expr::Var(n) = &callee.inner {
                            n.clone()
                        } else {
                            "<anonymous>".into()
                        };
                        ctx.error(
                            TypeErrorKind::ArityMismatch {
                                name,
                                expected: ft.params.len(),
                                got: args.len(),
                            },
                            span,
                        );
                        return Type::Error;
                    }
                    for (param_ty, arg) in ft.params.iter().zip(args.iter()) {
                        let arg_ty = infer_expr(ctx, &arg.inner, arg.span);
                        ctx.unify(param_ty, &arg_ty, arg.span);
                    }
                    *ft.ret
                }
                Type::Adt(name, _) => {
                    // Constructor call — produces a constructed value
                    let arg_types: Vec<Type> = args
                        .iter()
                        .map(|a| infer_expr(ctx, &a.inner, a.span))
                        .collect();
                    // Validate against constructor definition if known
                    if let Some((_, field_types)) = ctx.get_constructor(&name) {
                        let field_types = field_types.clone();
                        if field_types.len() != arg_types.len() {
                            ctx.error(
                                TypeErrorKind::ArityMismatch {
                                    name: name.clone(),
                                    expected: field_types.len(),
                                    got: arg_types.len(),
                                },
                                span,
                            );
                        } else {
                            for (expected, actual) in field_types.iter().zip(arg_types.iter()) {
                                ctx.unify(expected, actual, span);
                            }
                        }
                    }
                    Type::Adt(name, vec![])
                }
                Type::Error => Type::Error,
                Type::Var(_) => {
                    // Unknown callee type — could be generic, infer args
                    for arg in args {
                        infer_expr(ctx, &arg.inner, arg.span);
                    }
                    ctx.fresh_var()
                }
                other => {
                    ctx.error(TypeErrorKind::NotCallable(other), span);
                    Type::Error
                }
            }
        }

        Expr::Do(exprs) => {
            ctx.push_scope();
            let mut last_ty = Type::Nil;
            for expr_node in exprs {
                // Handle declarations inside do blocks
                match &expr_node.inner {
                    Expr::FnExpr(fn_decl) => {
                        collect_fn(ctx, fn_decl);
                        check_fn_body(ctx, fn_decl, expr_node.span);
                        last_ty = Type::Nil;
                    }
                    Expr::Let(let_decl, body) => {
                        check_let(ctx, let_decl, expr_node.span);
                        last_ty = infer_expr(ctx, &body.inner, body.span);
                    }
                    _ => {
                        last_ty = infer_expr(ctx, &expr_node.inner, expr_node.span);
                    }
                }
            }
            ctx.pop_scope();
            last_ty
        }

        Expr::Match(scrutinee, arms) => {
            let scrutinee_ty = infer_expr(ctx, &scrutinee.inner, scrutinee.span);
            let mut result_ty: Option<Type> = None;

            for arm in arms {
                ctx.push_scope();
                check_pattern(ctx, &arm.pattern.inner, &scrutinee_ty, arm.pattern.span);
                let body_ty = infer_expr(ctx, &arm.body.inner, arm.body.span);
                if let Some(ref prev) = result_ty {
                    ctx.unify(prev, &body_ty, arm.body.span);
                } else {
                    result_ty = Some(body_ty);
                }
                ctx.pop_scope();
            }

            result_ty.unwrap_or(Type::Nil)
        }

        Expr::FnExpr(fn_decl) => {
            collect_fn(ctx, fn_decl);
            check_fn_body(ctx, fn_decl, span);
            if let Some(binding) = ctx.get_var(&fn_decl.name) {
                binding.ty.clone()
            } else {
                Type::Error
            }
        }

        Expr::Return(inner) => infer_expr(ctx, &inner.inner, inner.span),

        Expr::SetExpr(name, value) => {
            let val_ty = infer_expr(ctx, &value.inner, value.span);
            let var_info = ctx.get_var(name).map(|b| (b.ty.clone(), b.mutable));
            match var_info {
                Some((var_ty, mutable)) => {
                    if !mutable {
                        ctx.error(TypeErrorKind::ImmutableBinding(name.clone()), span);
                    }
                    ctx.unify(&var_ty, &val_ty, span);
                }
                None => {
                    ctx.error(TypeErrorKind::UndefinedVar(name.clone()), span);
                }
            }
            Type::Nil
        }

        Expr::ModAccess(_, member) => {
            // Module access — look up the member directly for now
            if let Some(binding) = ctx.get_var(member) {
                binding.ty.clone()
            } else {
                ctx.fresh_var()
            }
        }
        Expr::Error => Type::Error,
    }
}

fn check_pattern(ctx: &mut TypeContext, pattern: &Pattern, expected_ty: &Type, span: Span) {
    match pattern {
        Pattern::Wildcard => {}
        Pattern::Var(name) => {
            ctx.define_var(name.clone(), expected_ty.clone(), false);
        }
        Pattern::IntLit(_) => {
            if !matches!(expected_ty, Type::Int(_) | Type::Var(_) | Type::Error) {
                ctx.error(
                    TypeErrorKind::PatternMismatch {
                        expected: expected_ty.clone(),
                        found: "integer literal".into(),
                    },
                    span,
                );
            }
        }
        Pattern::FloatLit(_) => {
            if !matches!(expected_ty, Type::Float(_) | Type::Var(_) | Type::Error) {
                ctx.error(
                    TypeErrorKind::PatternMismatch {
                        expected: expected_ty.clone(),
                        found: "float literal".into(),
                    },
                    span,
                );
            }
        }
        Pattern::StrLit(_) => {
            if !matches!(expected_ty, Type::Str | Type::Var(_) | Type::Error) {
                ctx.error(
                    TypeErrorKind::PatternMismatch {
                        expected: expected_ty.clone(),
                        found: "string literal".into(),
                    },
                    span,
                );
            }
        }
        Pattern::BoolLit(_) => {
            if !matches!(expected_ty, Type::Bool | Type::Var(_) | Type::Error) {
                ctx.error(
                    TypeErrorKind::PatternMismatch {
                        expected: expected_ty.clone(),
                        found: "boolean literal".into(),
                    },
                    span,
                );
            }
        }
        Pattern::NilLit => {
            if !matches!(expected_ty, Type::Nil | Type::Var(_) | Type::Error) {
                ctx.error(
                    TypeErrorKind::PatternMismatch {
                        expected: expected_ty.clone(),
                        found: "nil".into(),
                    },
                    span,
                );
            }
        }
        Pattern::TypeName(name) => {
            // Nullary constructor pattern
            if let Some((type_name, fields)) = ctx.get_constructor(name) {
                let type_name = type_name.clone();
                if !fields.is_empty() {
                    ctx.error(
                        TypeErrorKind::ArityMismatch {
                            name: name.clone(),
                            expected: fields.len(),
                            got: 0,
                        },
                        span,
                    );
                }
                ctx.unify(expected_ty, &Type::Adt(type_name, vec![]), span);
            }
        }
        Pattern::Constructor(name, sub_patterns) => {
            if let Some((type_name, field_types)) = ctx.get_constructor(name) {
                let type_name = type_name.clone();
                let field_types = field_types.clone();
                ctx.unify(expected_ty, &Type::Adt(type_name, vec![]), span);
                if field_types.len() != sub_patterns.len() {
                    ctx.error(
                        TypeErrorKind::ArityMismatch {
                            name: name.clone(),
                            expected: field_types.len(),
                            got: sub_patterns.len(),
                        },
                        span,
                    );
                } else {
                    for (ft, sp) in field_types.iter().zip(sub_patterns.iter()) {
                        check_pattern(ctx, &sp.inner, ft, sp.span);
                    }
                }
            } else {
                // Unknown constructor — bind sub-patterns with fresh vars
                for sp in sub_patterns {
                    let fresh = ctx.fresh_var();
                    check_pattern(ctx, &sp.inner, &fresh, sp.span);
                }
            }
        }
    }
}

fn infer_binop(ctx: &mut TypeContext, op: BinOp, lhs: &Type, rhs: &Type, span: Span) -> Type {
    let lhs = ctx.resolve(lhs);
    let rhs = ctx.resolve(rhs);

    match op {
        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
            // String concatenation with +
            if op == BinOp::Add && lhs == Type::Str && rhs == Type::Str {
                return Type::Str;
            }
            // Numeric operations
            if lhs.is_numeric() && rhs.is_numeric() {
                ctx.unify(&lhs, &rhs, span);
                return lhs;
            }
            // Allow type vars
            if matches!(lhs, Type::Var(_)) || matches!(rhs, Type::Var(_)) {
                ctx.unify(&lhs, &rhs, span);
                return lhs;
            }
            if matches!(lhs, Type::Error) || matches!(rhs, Type::Error) {
                return Type::Error;
            }
            ctx.error(
                TypeErrorKind::InvalidOperator {
                    op: format!("{op:?}"),
                    lhs: lhs.clone(),
                    rhs: rhs.clone(),
                },
                span,
            );
            Type::Error
        }
        BinOp::Eq | BinOp::Neq => {
            ctx.unify(&lhs, &rhs, span);
            Type::Bool
        }
        BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => {
            if lhs.is_numeric() && rhs.is_numeric() {
                ctx.unify(&lhs, &rhs, span);
                return Type::Bool;
            }
            if matches!(lhs, Type::Var(_)) || matches!(rhs, Type::Var(_)) {
                ctx.unify(&lhs, &rhs, span);
                return Type::Bool;
            }
            if matches!(lhs, Type::Error) || matches!(rhs, Type::Error) {
                return Type::Bool;
            }
            ctx.error(
                TypeErrorKind::InvalidOperator {
                    op: format!("{op:?}"),
                    lhs: lhs.clone(),
                    rhs: rhs.clone(),
                },
                span,
            );
            Type::Bool
        }
        BinOp::And | BinOp::Or => {
            ctx.unify(&Type::Bool, &lhs, span);
            ctx.unify(&Type::Bool, &rhs, span);
            Type::Bool
        }
    }
}

fn infer_unaryop(ctx: &mut TypeContext, op: UnaryOp, operand: &Type, span: Span) -> Type {
    let operand = ctx.resolve(operand);
    match op {
        UnaryOp::Neg => {
            if operand.is_numeric() || matches!(operand, Type::Var(_) | Type::Error) {
                operand
            } else {
                ctx.error(
                    TypeErrorKind::InvalidUnaryOperator {
                        op: "Neg".into(),
                        operand: operand.clone(),
                    },
                    span,
                );
                Type::Error
            }
        }
        UnaryOp::Not => {
            ctx.unify(&Type::Bool, &operand, span);
            Type::Bool
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::TypeErrors;

    fn check(src: &str) -> TypeErrors {
        let program = llml_parser::parse(src).expect("parse error");
        let mut ctx = TypeContext::new();
        check_program(&mut ctx, &program);
        ctx.errors
    }

    #[test]
    fn test_valid_arithmetic() {
        let errors = check("(+ 1 2)");
        assert!(errors.is_empty(), "unexpected errors: {errors}");
    }

    #[test]
    fn test_type_mismatch_arithmetic() {
        let errors = check("(+ \"hello\" 42)");
        assert!(!errors.is_empty(), "expected type error");
    }

    #[test]
    fn test_valid_let_binding() {
        let errors = check("(do (let $x : @I32 42) $x)");
        assert!(errors.is_empty(), "unexpected errors: {errors}");
    }

    #[test]
    fn test_valid_function() {
        let errors = check(
            r#"
(fn $double (: @I32 -> @I32) ($n : @I32) (* $n 2))
($double 21)
"#,
        );
        assert!(errors.is_empty(), "unexpected errors: {errors}");
    }

    #[test]
    fn test_arity_mismatch() {
        let errors = check(
            r#"
(fn $f (: @I32 -> @I32) ($x : @I32) $x)
($f 1 2)
"#,
        );
        assert!(!errors.is_empty(), "expected arity error");
    }

    #[test]
    fn test_if_condition_type() {
        let errors = check("(if 42 1 0)");
        assert!(
            !errors.is_empty(),
            "expected type error for non-bool condition"
        );
    }

    #[test]
    fn test_valid_match() {
        let errors = check(
            r#"
(mat 1
  (0 "zero")
  (1 "one")
  ($x "other"))
"#,
        );
        assert!(errors.is_empty(), "unexpected errors: {errors}");
    }

    #[test]
    fn test_immutable_set() {
        let errors = check("(do (let $x : @I32 5) (set $x 10))");
        assert!(!errors.is_empty(), "expected immutable binding error");
    }

    #[test]
    fn test_mutable_set() {
        let errors = check("(do (let mut $x : @I32 5) (set $x 10) $x)");
        assert!(errors.is_empty(), "unexpected errors: {errors}");
    }

    #[test]
    fn test_undefined_var() {
        let errors = check("$unknown");
        assert!(!errors.is_empty(), "expected undefined var error");
    }

    #[test]
    fn test_valid_adt() {
        let errors = check(
            r#"
(ty @Option (sum (@None) (@Some $val : @I32)))
(mat (@Some 42)
  ((@Some $v) $v)
  (@None 0))
"#,
        );
        assert!(errors.is_empty(), "unexpected errors: {errors}");
    }

    #[test]
    fn test_recursive_function() {
        let errors = check(
            r#"
(fn $fact (: @I32 -> @I32) ($n : @I32)
  (if (= $n 0) 1 (* $n ($fact (- $n 1)))))
($fact 5)
"#,
        );
        assert!(errors.is_empty(), "unexpected errors: {errors}");
    }

    #[test]
    fn test_higher_order_function() {
        let errors = check(
            r#"
(fn $apply (: (: @I32 -> @I32) @I32 -> @I32)
  ($f : (: @I32 -> @I32)) ($x : @I32)
  ($f $x))
(fn $double (: @I32 -> @I32) ($n : @I32) (* $n 2))
($apply $double 21)
"#,
        );
        assert!(errors.is_empty(), "unexpected errors: {errors}");
    }
}
