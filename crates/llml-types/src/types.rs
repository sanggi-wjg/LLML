//! Core type representations for the LLML type system.

use std::fmt;

/// Unique identifier for type variables during unification.
pub type TypeVarId = u32;

/// Integer bit-width variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntSize {
    I8,
    I16,
    I32,
    I64,
}

/// Unsigned integer bit-width variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UIntSize {
    U8,
    U16,
    U32,
    U64,
}

/// Float bit-width variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FloatSize {
    F32,
    F64,
}

/// Internal representation of types during type checking.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    /// Signed integer types (@I8, @I16, @I32, @I64)
    Int(IntSize),
    /// Unsigned integer types (@U8, @U16, @U32, @U64)
    UInt(UIntSize),
    /// Float types (@F32, @F64)
    Float(FloatSize),
    /// Boolean type (@Bool)
    Bool,
    /// String type (@Str)
    Str,
    /// Nil/unit type (@Nil)
    Nil,
    /// Byte type (@Byte)
    Byte,
    /// Function type: param types -> return type, with effects
    Fn(FnType),
    /// Algebraic data type with optional type arguments
    Adt(String, Vec<Type>),
    /// Unification variable (resolved during generic checking)
    Var(TypeVarId),
    /// Linear type wrapper (~@Type)
    Linear(Box<Type>),
    /// Reference type (&@Type)
    Ref(Box<Type>),
    /// Poison type — used for error recovery so checking can continue
    Error,
}

/// Function type: parameter types, return type, and effect set.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FnType {
    pub params: Vec<Type>,
    pub ret: Box<Type>,
    pub effects: Vec<String>,
}

impl Type {
    /// Check if this type is numeric (integer or float).
    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::Int(_) | Type::UInt(_) | Type::Float(_))
    }

    /// Check if this type is an integer type.
    pub fn is_integer(&self) -> bool {
        matches!(self, Type::Int(_) | Type::UInt(_))
    }

    /// Check if this type is a float type.
    pub fn is_float(&self) -> bool {
        matches!(self, Type::Float(_))
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int(s) => write!(f, "@{s:?}"),
            Type::UInt(s) => write!(f, "@{s:?}"),
            Type::Float(s) => write!(f, "@{s:?}"),
            Type::Bool => write!(f, "@Bool"),
            Type::Str => write!(f, "@Str"),
            Type::Nil => write!(f, "@Nil"),
            Type::Byte => write!(f, "@Byte"),
            Type::Fn(ft) => {
                write!(f, "(:")?;
                for p in &ft.params {
                    write!(f, " {p}")?;
                }
                write!(f, " -> {}", ft.ret)?;
                for eff in &ft.effects {
                    write!(f, " !{eff}")?;
                }
                write!(f, ")")
            }
            Type::Adt(name, args) => {
                write!(f, "@{name}")?;
                for arg in args {
                    write!(f, " {arg}")?;
                }
                Ok(())
            }
            Type::Var(id) => write!(f, "?{id}"),
            Type::Linear(inner) => write!(f, "~{inner}"),
            Type::Ref(inner) => write!(f, "&{inner}"),
            Type::Error => write!(f, "<error>"),
        }
    }
}

/// Resolve a type name string (e.g., "I32", "Str") to a Type.
pub fn resolve_named_type(name: &str) -> Option<Type> {
    match name {
        "I8" => Some(Type::Int(IntSize::I8)),
        "I16" => Some(Type::Int(IntSize::I16)),
        "I32" => Some(Type::Int(IntSize::I32)),
        "I64" => Some(Type::Int(IntSize::I64)),
        "U8" => Some(Type::UInt(UIntSize::U8)),
        "U16" => Some(Type::UInt(UIntSize::U16)),
        "U32" => Some(Type::UInt(UIntSize::U32)),
        "U64" => Some(Type::UInt(UIntSize::U64)),
        "F32" => Some(Type::Float(FloatSize::F32)),
        "F64" => Some(Type::Float(FloatSize::F64)),
        "Bool" => Some(Type::Bool),
        "Str" => Some(Type::Str),
        "Nil" => Some(Type::Nil),
        "Byte" => Some(Type::Byte),
        _ => None,
    }
}
