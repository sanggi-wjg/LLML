use std::collections::HashMap;

use crate::value::Value;

/// Error from a built-in function call.
#[derive(Debug, Clone, thiserror::Error)]
pub enum BuiltinError {
    #[error("arity mismatch: {name} expects {expected} args, got {got}")]
    ArityMismatch {
        name: String,
        expected: usize,
        got: usize,
    },
    #[error("type error: {0}")]
    TypeError(String),
}

/// Function pointer type for built-in function implementations.
///
/// The `output` parameter allows builtins like `$print` to capture output.
/// Pure builtins should ignore it.
pub type BuiltinFnPtr = fn(&[Value], &mut Vec<String>) -> Result<Value, BuiltinError>;

/// Metadata about a built-in function.
pub struct BuiltinDef {
    /// Function name without sigil (e.g., "print")
    pub name: &'static str,
    /// Expected number of arguments
    pub arity: usize,
    /// The implementation
    pub eval: BuiltinFnPtr,
}

/// Registry of all built-in functions.
pub struct BuiltinRegistry {
    builtins: HashMap<String, BuiltinDef>,
}

impl BuiltinRegistry {
    /// Create a registry with all standard built-in functions.
    pub fn standard() -> Self {
        let mut reg = Self {
            builtins: HashMap::new(),
        };
        reg.register(crate::io::builtin_print());
        reg.register(crate::str::builtin_to_str());
        reg.register(crate::str::builtin_str_concat());
        reg.register(crate::str::builtin_len());
        reg.register(crate::math::builtin_not());
        reg.register(crate::math::builtin_abs());
        reg
    }

    fn register(&mut self, def: BuiltinDef) {
        self.builtins.insert(def.name.to_string(), def);
    }

    /// Look up a built-in by name.
    pub fn get(&self, name: &str) -> Option<&BuiltinDef> {
        self.builtins.get(name)
    }

    /// Iterate over all registered builtins.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.builtins.keys().map(|s| s.as_str())
    }

    /// Call a built-in function by name.
    pub fn call(
        &self,
        name: &str,
        args: &[Value],
        output: &mut Vec<String>,
    ) -> Result<Value, BuiltinError> {
        let def = self
            .builtins
            .get(name)
            .ok_or_else(|| BuiltinError::TypeError(format!("unknown builtin: {name}")))?;
        if args.len() != def.arity {
            return Err(BuiltinError::ArityMismatch {
                name: name.to_string(),
                expected: def.arity,
                got: args.len(),
            });
        }
        (def.eval)(args, output)
    }
}
