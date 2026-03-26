use crate::registry::{BuiltinDef, BuiltinError};
use crate::value::Value;

/// `$print` — output a value as a string, returns Nil.
pub fn builtin_print() -> BuiltinDef {
    BuiltinDef {
        name: "print",
        arity: 1,
        eval: |args, output| {
            output.push(args[0].to_string());
            Ok(Value::Nil)
        },
    }
}

/// `$read_line` — placeholder for future stdin reading.
pub fn builtin_read_line() -> BuiltinDef {
    BuiltinDef {
        name: "read_line",
        arity: 0,
        eval: |_args, _output| {
            Err(BuiltinError::TypeError(
                "read_line is not yet implemented".to_string(),
            ))
        },
    }
}
