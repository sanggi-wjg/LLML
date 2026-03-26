use crate::registry::{BuiltinDef, BuiltinError};
use crate::value::Value;

/// `$to_str` — convert any value to its string representation.
pub fn builtin_to_str() -> BuiltinDef {
    BuiltinDef {
        name: "to_str",
        arity: 1,
        eval: |args, _output| Ok(Value::Str(args[0].to_string())),
    }
}

/// `$str_concat` — concatenate two values as strings.
pub fn builtin_str_concat() -> BuiltinDef {
    BuiltinDef {
        name: "str_concat",
        arity: 2,
        eval: |args, _output| Ok(Value::Str(format!("{}{}", args[0], args[1]))),
    }
}

/// `$len` — return the length of a string.
pub fn builtin_len() -> BuiltinDef {
    BuiltinDef {
        name: "len",
        arity: 1,
        eval: |args, _output| match &args[0] {
            Value::Str(s) => Ok(Value::Int(s.len() as i64)),
            _ => Err(BuiltinError::TypeError("len expects @Str".to_string())),
        },
    }
}
