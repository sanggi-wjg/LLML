use crate::value::{Env, FnClosure, SetError, Value};
use llml_parser::ast::*;
use llml_stdlib::BuiltinRegistry;

/// Interpreter error.
#[derive(Debug, Clone, thiserror::Error)]
pub enum EvalError {
    #[error("undefined variable: ${0}")]
    UndefinedVar(String),

    #[error("type error: {0}")]
    TypeError(String),

    #[error("arity mismatch: {name} expects {expected} args, got {got}")]
    ArityMismatch {
        name: String,
        expected: usize,
        got: usize,
    },

    #[error("match exhausted: no arm matched the value")]
    MatchExhausted,

    #[error("cannot call non-function value: {0}")]
    NotCallable(String),

    #[error("division by zero")]
    DivisionByZero,

    #[error("cannot mutate immutable binding: ${0}")]
    ImmutableBinding(String),

    #[error("early return")]
    Return(Box<Value>),
}

type Result<T> = std::result::Result<T, EvalError>;

/// Tree-walk interpreter for LLML.
pub struct Interpreter {
    env: Env,
    output: Vec<String>,
    builtins: BuiltinRegistry,
}

impl Interpreter {
    pub fn new() -> Self {
        let builtins = BuiltinRegistry::standard();
        let mut env = Env::new();
        // Register all built-in function names in the environment
        for name in builtins.names() {
            env.define(name.to_string(), Value::BuiltinFn(name.to_string()));
        }
        Self {
            env,
            output: Vec::new(),
            builtins,
        }
    }

    pub fn output(&self) -> &[String] {
        &self.output
    }

    /// Execute a complete program.
    pub fn exec_program(&mut self, program: &Program) -> Result<Value> {
        let mut last = Value::Nil;
        for decl in &program.decls {
            last = self.exec_decl(&decl.inner)?;
        }
        Ok(last)
    }

    fn exec_decl(&mut self, decl: &Decl) -> Result<Value> {
        match decl {
            Decl::Fn(fn_decl) => {
                let closure = Value::Fn(FnClosure {
                    name: fn_decl.name.clone(),
                    params: fn_decl.params.clone(),
                    body: *fn_decl.body.clone(),
                    env: self.env.clone(),
                });
                self.env.define(fn_decl.name.clone(), closure);
                Ok(Value::Nil)
            }
            Decl::Let(let_decl) => {
                let val = self.eval_expr(&let_decl.value.inner)?;
                self.env
                    .define_mut(let_decl.name.clone(), val, let_decl.is_mut);
                Ok(Value::Nil)
            }
            Decl::TypeDef(_) => {
                // Type definitions are compile-time only in Phase 1
                Ok(Value::Nil)
            }
            Decl::Module(mod_decl) => {
                // Execute module declarations in current scope for now
                for d in &mod_decl.decls {
                    self.exec_decl(&d.inner)?;
                }
                Ok(Value::Nil)
            }
            Decl::Pub(inner) => self.exec_decl(&inner.inner),
            Decl::Expr(expr) => self.eval_expr(expr),
        }
    }

    /// Evaluate an expression.
    pub fn eval_expr(&mut self, expr: &Expr) -> Result<Value> {
        match expr {
            // ── Literals ──────────────────────
            Expr::IntLit(n) => Ok(Value::Int(*n)),
            Expr::FloatLit(n) => Ok(Value::Float(*n)),
            Expr::StrLit(s) => Ok(Value::Str(s.clone())),
            Expr::BoolLit(b) => Ok(Value::Bool(*b)),
            Expr::NilLit => Ok(Value::Nil),

            // ── Variables ─────────────────────
            Expr::Var(name) => self
                .env
                .get(name)
                .cloned()
                .ok_or_else(|| EvalError::UndefinedVar(name.clone())),

            // ── Type constructors ─────────────
            Expr::TypeConstructor(name) => Ok(Value::Constructor(name.clone(), vec![])),

            // ── Let binding ───────────────────
            Expr::Let(let_decl, continuation) => {
                let val = self.eval_expr(&let_decl.value.inner)?;
                self.env
                    .define_mut(let_decl.name.clone(), val, let_decl.is_mut);
                self.eval_expr(&continuation.inner)
            }

            // ── Function expression ───────────
            Expr::FnExpr(fn_decl) => {
                let closure = Value::Fn(FnClosure {
                    name: fn_decl.name.clone(),
                    params: fn_decl.params.clone(),
                    body: *fn_decl.body.clone(),
                    env: self.env.clone(),
                });
                if !fn_decl.name.is_empty() && fn_decl.name != "_" {
                    self.env.define(fn_decl.name.clone(), closure.clone());
                }
                Ok(closure)
            }

            // ── Function call ─────────────────
            Expr::Call(callee, args) => {
                let callee_val = self.eval_expr(&callee.inner)?;
                let mut arg_vals = Vec::with_capacity(args.len());
                for arg in args {
                    arg_vals.push(self.eval_expr(&arg.inner)?);
                }
                self.call_fn(callee_val, arg_vals)
            }

            // ── If expression ─────────────────
            Expr::If(cond, then_branch, else_branch) => {
                let cond_val = self.eval_expr(&cond.inner)?;
                match cond_val {
                    Value::Bool(true) => self.eval_expr(&then_branch.inner),
                    Value::Bool(false) => self.eval_expr(&else_branch.inner),
                    _ => Err(EvalError::TypeError(
                        "if condition must be @Bool".to_string(),
                    )),
                }
            }

            // ── Match expression ──────────────
            Expr::Match(scrutinee, arms) => {
                let val = self.eval_expr(&scrutinee.inner)?;
                for arm in arms {
                    if let Some(bindings) = self.match_pattern(&arm.pattern.inner, &val) {
                        self.env.push_scope();
                        for (name, bound_val) in bindings {
                            self.env.define(name, bound_val);
                        }
                        let result = self.eval_expr(&arm.body.inner);
                        self.env.pop_scope();
                        return result;
                    }
                }
                Err(EvalError::MatchExhausted)
            }

            // ── Do block ──────────────────────
            Expr::Do(exprs) => {
                self.env.push_scope();
                let mut last = Value::Nil;
                for expr_node in exprs {
                    match self.eval_expr(&expr_node.inner) {
                        Ok(val) => last = val,
                        Err(EvalError::Return(boxed_val)) => {
                            self.env.pop_scope();
                            return Err(EvalError::Return(boxed_val));
                        }
                        Err(e) => {
                            self.env.pop_scope();
                            return Err(e);
                        }
                    }
                }
                self.env.pop_scope();
                Ok(last)
            }

            // ── Binary operators ──────────────
            Expr::BinOp(op, lhs, rhs) => {
                let l = self.eval_expr(&lhs.inner)?;
                let r = self.eval_expr(&rhs.inner)?;
                self.eval_binop(*op, l, r)
            }

            // ── Unary operators ───────────────
            Expr::UnaryOp(op, operand) => {
                let val = self.eval_expr(&operand.inner)?;
                self.eval_unaryop(*op, val)
            }

            // ── Return ────────────────────────
            Expr::Return(expr) => {
                let val = self.eval_expr(&expr.inner)?;
                Err(EvalError::Return(Box::new(val)))
            }

            // ── Set (mutation) ────────────────
            Expr::SetExpr(name, expr) => {
                let val = self.eval_expr(&expr.inner)?;
                match self.env.set(name, val) {
                    Ok(()) => Ok(Value::Nil),
                    Err(SetError::Immutable(_)) => Err(EvalError::ImmutableBinding(name.clone())),
                    Err(SetError::Undefined(_)) => Err(EvalError::UndefinedVar(name.clone())),
                }
            }

            // ── Module access ─────────────────
            Expr::ModAccess(_mod_name, var_name) => {
                // For now, just look up the var directly
                self.env
                    .get(var_name)
                    .cloned()
                    .ok_or_else(|| EvalError::UndefinedVar(var_name.clone()))
            }
        }
    }

    fn call_fn(&mut self, callee: Value, args: Vec<Value>) -> Result<Value> {
        match callee {
            Value::Fn(closure) => {
                if closure.params.len() != args.len() {
                    return Err(EvalError::ArityMismatch {
                        name: closure.name.clone(),
                        expected: closure.params.len(),
                        got: args.len(),
                    });
                }

                // Create a new environment from the closure's captured env
                let saved_env = std::mem::replace(&mut self.env, closure.env.clone());
                self.env.push_scope();

                // Bind the function itself for recursion
                self.env.define(
                    closure.name.clone(),
                    Value::Fn(FnClosure {
                        name: closure.name.clone(),
                        params: closure.params.clone(),
                        body: closure.body.clone(),
                        env: closure.env.clone(),
                    }),
                );

                // Bind parameters
                for (param, arg) in closure.params.iter().zip(args) {
                    self.env.define(param.name.clone(), arg);
                }

                let result = match self.eval_expr(&closure.body.inner) {
                    Ok(val) => Ok(val),
                    Err(EvalError::Return(boxed_val)) => Ok(*boxed_val),
                    Err(e) => Err(e),
                };

                self.env.pop_scope();
                self.env = saved_env;
                result
            }
            Value::BuiltinFn(name) => self.call_builtin(&name, args),
            Value::Constructor(ctor_name, _) => {
                // Type constructor call: (@Some $val) → Constructor("Some", [val])
                Ok(Value::Constructor(ctor_name, args))
            }
            other => Err(EvalError::NotCallable(format!("{other}"))),
        }
    }

    fn call_builtin(&mut self, name: &str, args: Vec<Value>) -> Result<Value> {
        self.builtins
            .call(name, &args, &mut self.output)
            .map_err(|e| match e {
                llml_stdlib::BuiltinError::ArityMismatch {
                    name,
                    expected,
                    got,
                } => EvalError::ArityMismatch {
                    name,
                    expected,
                    got,
                },
                llml_stdlib::BuiltinError::TypeError(msg) => EvalError::TypeError(msg),
            })
    }

    fn eval_binop(&self, op: BinOp, lhs: Value, rhs: Value) -> Result<Value> {
        match (op, &lhs, &rhs) {
            // Integer arithmetic
            (BinOp::Add, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (BinOp::Sub, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            (BinOp::Mul, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            (BinOp::Div, Value::Int(_), Value::Int(0)) => Err(EvalError::DivisionByZero),
            (BinOp::Div, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a / b)),
            (BinOp::Mod, Value::Int(_), Value::Int(0)) => Err(EvalError::DivisionByZero),
            (BinOp::Mod, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a % b)),

            // Float arithmetic
            (BinOp::Add, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (BinOp::Sub, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (BinOp::Mul, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (BinOp::Div, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),

            // String concatenation
            (BinOp::Add, Value::Str(a), Value::Str(b)) => Ok(Value::Str(format!("{a}{b}"))),

            // Integer comparison
            (BinOp::Eq, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a == b)),
            (BinOp::Neq, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a != b)),
            (BinOp::Lt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
            (BinOp::Gt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
            (BinOp::Le, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
            (BinOp::Ge, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),

            // Float comparison
            (BinOp::Eq, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a == b)),
            (BinOp::Neq, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a != b)),
            (BinOp::Lt, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
            (BinOp::Gt, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
            (BinOp::Le, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
            (BinOp::Ge, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),

            // String comparison
            (BinOp::Eq, Value::Str(a), Value::Str(b)) => Ok(Value::Bool(a == b)),
            (BinOp::Neq, Value::Str(a), Value::Str(b)) => Ok(Value::Bool(a != b)),

            // Boolean comparison
            (BinOp::Eq, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a == b)),
            (BinOp::Neq, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a != b)),

            // Boolean logic
            (BinOp::And, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a && *b)),
            (BinOp::Or, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a || *b)),

            // Constructor equality
            (BinOp::Eq, Value::Constructor(n1, f1), Value::Constructor(n2, f2)) => {
                Ok(Value::Bool(n1 == n2 && f1 == f2))
            }
            (BinOp::Neq, Value::Constructor(n1, f1), Value::Constructor(n2, f2)) => {
                Ok(Value::Bool(n1 != n2 || f1 != f2))
            }

            _ => Err(EvalError::TypeError(format!(
                "cannot apply {op:?} to {lhs} and {rhs}"
            ))),
        }
    }

    fn eval_unaryop(&self, op: UnaryOp, val: Value) -> Result<Value> {
        match (op, &val) {
            (UnaryOp::Neg, Value::Int(n)) => Ok(Value::Int(-n)),
            (UnaryOp::Neg, Value::Float(n)) => Ok(Value::Float(-n)),
            (UnaryOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
            _ => Err(EvalError::TypeError(format!(
                "cannot apply {op:?} to {val}"
            ))),
        }
    }

    /// Try to match a pattern against a value.
    /// Returns Some(bindings) if the pattern matches, None otherwise.
    fn match_pattern(&self, pattern: &Pattern, value: &Value) -> Option<Vec<(String, Value)>> {
        match (pattern, value) {
            (Pattern::Wildcard, _) => Some(vec![]),

            (Pattern::Var(name), val) => Some(vec![(name.clone(), val.clone())]),

            (Pattern::IntLit(p), Value::Int(v)) if p == v => Some(vec![]),
            (Pattern::FloatLit(p), Value::Float(v)) if p == v => Some(vec![]),
            (Pattern::StrLit(p), Value::Str(v)) if p == v => Some(vec![]),
            (Pattern::BoolLit(p), Value::Bool(v)) if p == v => Some(vec![]),
            (Pattern::NilLit, Value::Nil) => Some(vec![]),

            // Nullary constructor: @None matches Constructor("None", [])
            (Pattern::TypeName(pname), Value::Constructor(vname, fields))
                if pname == vname && fields.is_empty() =>
            {
                Some(vec![])
            }

            // Constructor pattern: (@Some $v) matches Constructor("Some", [val])
            (Pattern::Constructor(pname, sub_patterns), Value::Constructor(vname, fields))
                if pname == vname && sub_patterns.len() == fields.len() =>
            {
                let mut bindings = Vec::new();
                for (sub_pat, field_val) in sub_patterns.iter().zip(fields.iter()) {
                    match self.match_pattern(&sub_pat.inner, field_val) {
                        Some(sub_bindings) => bindings.extend(sub_bindings),
                        None => return None,
                    }
                }
                Some(bindings)
            }

            _ => None,
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llml_parser::parse;

    fn run(src: &str) -> Result<Value> {
        let program = parse(src).expect("parse error");
        let mut interp = Interpreter::new();
        interp.exec_program(&program)
    }

    fn run_with_output(src: &str) -> (Result<Value>, Vec<String>) {
        let program = parse(src).expect("parse error");
        let mut interp = Interpreter::new();
        let result = interp.exec_program(&program);
        (result, interp.output().to_vec())
    }

    #[test]
    fn test_int_literal() {
        assert_eq!(run("42").unwrap(), Value::Int(42));
    }

    #[test]
    fn test_float_literal() {
        assert_eq!(run("3.14").unwrap(), Value::Float(3.14));
    }

    #[test]
    fn test_string_literal() {
        assert_eq!(run("\"hello\"").unwrap(), Value::Str("hello".to_string()));
    }

    #[test]
    fn test_bool_literal() {
        assert_eq!(run("true").unwrap(), Value::Bool(true));
        assert_eq!(run("false").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_nil_literal() {
        assert_eq!(run("nil").unwrap(), Value::Nil);
    }

    #[test]
    fn test_arithmetic() {
        assert_eq!(run("(+ 2 3)").unwrap(), Value::Int(5));
        assert_eq!(run("(- 10 4)").unwrap(), Value::Int(6));
        assert_eq!(run("(* 3 7)").unwrap(), Value::Int(21));
        assert_eq!(run("(/ 15 3)").unwrap(), Value::Int(5));
        assert_eq!(run("(% 10 3)").unwrap(), Value::Int(1));
    }

    #[test]
    fn test_nested_arithmetic() {
        assert_eq!(run("(+ (* 2 3) (- 10 4))").unwrap(), Value::Int(12));
    }

    #[test]
    fn test_comparison() {
        assert_eq!(run("(< 1 2)").unwrap(), Value::Bool(true));
        assert_eq!(run("(> 1 2)").unwrap(), Value::Bool(false));
        assert_eq!(run("(= 5 5)").unwrap(), Value::Bool(true));
        assert_eq!(run("(!= 5 3)").unwrap(), Value::Bool(true));
        assert_eq!(run("(<= 5 5)").unwrap(), Value::Bool(true));
        assert_eq!(run("(>= 3 5)").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_boolean_logic() {
        assert_eq!(run("(&& true true)").unwrap(), Value::Bool(true));
        assert_eq!(run("(&& true false)").unwrap(), Value::Bool(false));
        assert_eq!(run("(|| false true)").unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_if_expression() {
        assert_eq!(run("(if true 1 0)").unwrap(), Value::Int(1));
        assert_eq!(run("(if false 1 0)").unwrap(), Value::Int(0));
        assert_eq!(
            run("(if (< 3 5) \"yes\" \"no\")").unwrap(),
            Value::Str("yes".to_string())
        );
    }

    #[test]
    fn test_let_binding() {
        assert_eq!(run("(do (let $x : @I32 42) $x)").unwrap(), Value::Int(42));
    }

    #[test]
    fn test_let_multiple() {
        assert_eq!(
            run("(do (let $x : @I32 10) (let $y : @I32 20) (+ $x $y))").unwrap(),
            Value::Int(30)
        );
    }

    #[test]
    fn test_function_def_and_call() {
        let src = r#"
(fn $double (: @I32 -> @I32) ($n : @I32) (* $n 2))
($double 21)
"#;
        assert_eq!(run(src).unwrap(), Value::Int(42));
    }

    #[test]
    fn test_recursive_function() {
        let src = r#"
(fn $fib (: @I32 -> @I32) ($n : @I32)
  (if (<= $n 1) $n (+ ($fib (- $n 1)) ($fib (- $n 2)))))
($fib 10)
"#;
        assert_eq!(run(src).unwrap(), Value::Int(55));
    }

    #[test]
    fn test_factorial() {
        let src = r#"
(fn $fact (: @I32 -> @I32) ($n : @I32)
  (if (= $n 0) 1 (* $n ($fact (- $n 1)))))
($fact 5)
"#;
        assert_eq!(run(src).unwrap(), Value::Int(120));
    }

    #[test]
    fn test_higher_order_function() {
        let src = r#"
(fn $apply (: (: @I32 -> @I32) @I32 -> @I32)
  ($f : (: @I32 -> @I32)) ($x : @I32)
  ($f $x))
(fn $double (: @I32 -> @I32) ($n : @I32) (* $n 2))
($apply $double 21)
"#;
        assert_eq!(run(src).unwrap(), Value::Int(42));
    }

    #[test]
    fn test_constructor() {
        let src = r#"
(@Some 42)
"#;
        assert_eq!(
            run(src).unwrap(),
            Value::Constructor("Some".to_string(), vec![Value::Int(42)])
        );
    }

    #[test]
    fn test_match_simple() {
        let src = r#"
(mat 1
  (0 "zero")
  (1 "one")
  ($x "other"))
"#;
        assert_eq!(run(src).unwrap(), Value::Str("one".to_string()));
    }

    #[test]
    fn test_match_constructor() {
        let src = r#"
(fn $unwrap_or (: @I32 -> @I32) ($default : @I32)
  (mat (@Some 42)
    ((@Some $v) $v)
    (@None $default)))
($unwrap_or 0)
"#;
        assert_eq!(run(src).unwrap(), Value::Int(42));
    }

    #[test]
    fn test_match_none() {
        let src = r#"
(mat @None
  ((@Some $v) $v)
  (@None 0))
"#;
        assert_eq!(run(src).unwrap(), Value::Int(0));
    }

    #[test]
    fn test_do_block() {
        let src = r#"
(do
  (let $a : @I32 10)
  (let $b : @I32 20)
  (let $c : @I32 (+ $a $b))
  (* $c 2))
"#;
        assert_eq!(run(src).unwrap(), Value::Int(60));
    }

    #[test]
    fn test_print_builtin() {
        let src = r#"
($print "hello world")
"#;
        let (result, output) = run_with_output(src);
        assert_eq!(result.unwrap(), Value::Nil);
        assert_eq!(output, vec!["hello world"]);
    }

    #[test]
    fn test_to_str_builtin() {
        let src = r#"
($to_str 42)
"#;
        assert_eq!(run(src).unwrap(), Value::Str("42".to_string()));
    }

    #[test]
    fn test_negation() {
        assert_eq!(run("(- 5)").unwrap(), Value::Int(-5));
    }

    #[test]
    fn test_string_concat() {
        assert_eq!(
            run(r#"(+ "hello " "world")"#).unwrap(),
            Value::Str("hello world".to_string())
        );
    }

    #[test]
    fn test_division_by_zero() {
        assert!(matches!(run("(/ 1 0)"), Err(EvalError::DivisionByZero)));
    }

    #[test]
    fn test_undefined_var() {
        assert!(matches!(run("$unknown"), Err(EvalError::UndefinedVar(_))));
    }

    #[test]
    fn test_set_immutable_fails() {
        let src = "(do (let $x : @I32 5) (set $x 10))";
        assert!(matches!(run(src), Err(EvalError::ImmutableBinding(_))));
    }

    #[test]
    fn test_set_mutable_succeeds() {
        let src = "(do (let mut $x : @I32 5) (set $x 10) $x)";
        assert_eq!(run(src).unwrap(), Value::Int(10));
    }

    #[test]
    fn test_complete_program() {
        let src = r#"
(ty @Expr (sum
  (@Num $val : @F64)
  (@Add $l : @Expr $r : @Expr)
  (@Mul $l : @Expr $r : @Expr)))

(fn $eval (: @Expr -> @F64) ($e : @Expr)
  (mat $e
    ((@Num $v) $v)
    ((@Add $l $r) (+ ($eval $l) ($eval $r)))
    ((@Mul $l $r) (* ($eval $l) ($eval $r)))))

(let $expr : @Expr
  (@Add (@Num 1.0) (@Mul (@Num 2.0) (@Num 3.0))))
($eval $expr)
"#;
        assert_eq!(run(src).unwrap(), Value::Float(7.0));
    }
}
