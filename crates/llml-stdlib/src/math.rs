use crate::registry::{BuiltinDef, BuiltinError};
use crate::value::Value;

/// `$not` — boolean negation.
pub fn builtin_not() -> BuiltinDef {
    BuiltinDef {
        name: "not",
        arity: 1,
        eval: |args, _output| match &args[0] {
            Value::Bool(b) => Ok(Value::Bool(!b)),
            _ => Err(BuiltinError::TypeError("not expects @Bool".to_string())),
        },
    }
}

/// `$abs` — absolute value for integers and floats.
pub fn builtin_abs() -> BuiltinDef {
    BuiltinDef {
        name: "abs",
        arity: 1,
        eval: |args, _output| match &args[0] {
            Value::Int(n) => Ok(Value::Int(n.abs())),
            Value::Float(n) => Ok(Value::Float(n.abs())),
            _ => Err(BuiltinError::TypeError(
                "abs expects numeric type".to_string(),
            )),
        },
    }
}
