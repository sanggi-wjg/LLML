use std::collections::HashMap;
use std::fmt;

use llml_parser::ast::Param;

/// Runtime values in the LLML interpreter.
#[derive(Debug, Clone)]
pub enum Value {
    /// Integer (i64)
    Int(i64),
    /// Float (f64)
    Float(f64),
    /// String
    Str(String),
    /// Boolean
    Bool(bool),
    /// Nil (unit)
    Nil,
    /// A constructed value: @ConstructorName(field1, field2, ...)
    Constructor(String, Vec<Value>),
    /// A function closure
    Fn(FnClosure),
    /// A built-in function
    BuiltinFn(String),
}

/// A closure captures its defining environment.
#[derive(Debug, Clone)]
pub struct FnClosure {
    pub name: String,
    pub params: Vec<Param>,
    pub body: llml_parser::ast::Node<llml_parser::ast::Expr>,
    pub env: Env,
}

/// A binding entry storing a value and its mutability flag.
#[derive(Debug, Clone)]
struct Binding {
    value: Value,
    mutable: bool,
}

/// Environment: stack of scopes mapping variable names to values.
///
/// Each scope tracks both the value and whether the binding is mutable.
/// Immutable bindings (the default) cannot be reassigned with `set`.
#[derive(Debug, Clone)]
pub struct Env {
    scopes: Vec<HashMap<String, Binding>>,
}

impl Env {
    /// Create a new environment with a single empty scope.
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    /// Push a new scope onto the scope stack.
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Pop the innermost scope from the scope stack.
    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    /// Define an immutable binding in the current scope.
    pub fn define(&mut self, name: String, val: Value) {
        self.define_with_mutability(name, val, false);
    }

    /// Define a binding in the current scope with explicit mutability.
    pub fn define_mut(&mut self, name: String, val: Value, mutable: bool) {
        self.define_with_mutability(name, val, mutable);
    }

    fn define_with_mutability(&mut self, name: String, val: Value, mutable: bool) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(
                name,
                Binding {
                    value: val,
                    mutable,
                },
            );
        }
    }

    /// Reassign a mutable binding. Returns `Ok(())` on success,
    /// `Err(SetError::Immutable)` if the binding is immutable,
    /// or `Err(SetError::Undefined)` if the variable is not found.
    pub fn set(&mut self, name: &str, val: Value) -> std::result::Result<(), SetError> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(binding) = scope.get_mut(name) {
                if !binding.mutable {
                    return Err(SetError::Immutable(name.to_string()));
                }
                binding.value = val;
                return Ok(());
            }
        }
        Err(SetError::Undefined(name.to_string()))
    }

    /// Look up a variable by name, searching from innermost to outermost scope.
    pub fn get(&self, name: &str) -> Option<&Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(binding) = scope.get(name) {
                return Some(&binding.value);
            }
        }
        None
    }
}

/// Errors from attempting to `set` a variable.
#[derive(Debug, Clone)]
pub enum SetError {
    /// The binding exists but is immutable.
    Immutable(String),
    /// The variable is not defined in any scope.
    Undefined(String),
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::Constructor(n1, f1), Value::Constructor(n2, f2)) => n1 == n2 && f1 == f2,
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{n}"),
            Value::Float(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{n:.1}")
                } else {
                    write!(f, "{n}")
                }
            }
            Value::Str(s) => write!(f, "{s}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Nil => write!(f, "nil"),
            Value::Constructor(name, fields) => {
                if fields.is_empty() {
                    write!(f, "@{name}")
                } else {
                    write!(f, "(@{name}")?;
                    for field in fields {
                        write!(f, " {field}")?;
                    }
                    write!(f, ")")
                }
            }
            Value::Fn(closure) => write!(f, "<fn ${}>", closure.name),
            Value::BuiltinFn(name) => write!(f, "<builtin ${name}>"),
        }
    }
}
