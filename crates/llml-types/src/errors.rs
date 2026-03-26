//! Type checking error types.

use crate::types::Type;
use llml_lexer::Span;

/// A single type error with source location.
#[derive(Debug, Clone)]
pub struct TypeError {
    pub kind: TypeErrorKind,
    pub span: Span,
}

/// The specific kind of type error.
#[derive(Debug, Clone, thiserror::Error)]
pub enum TypeErrorKind {
    #[error("type mismatch: expected {expected}, found {found}")]
    Mismatch { expected: Type, found: Type },

    #[error("undefined variable: ${0}")]
    UndefinedVar(String),

    #[error("undefined type: @{0}")]
    UndefinedType(String),

    #[error("arity mismatch: {name} expects {expected} args, got {got}")]
    ArityMismatch {
        name: String,
        expected: usize,
        got: usize,
    },

    #[error("cannot call non-function type: {0}")]
    NotCallable(Type),

    #[error("operator {op} cannot be applied to {lhs} and {rhs}")]
    InvalidOperator { op: String, lhs: Type, rhs: Type },

    #[error("unary operator {op} cannot be applied to {operand}")]
    InvalidUnaryOperator { op: String, operand: Type },

    #[error("non-exhaustive match: missing patterns for {missing}")]
    NonExhaustiveMatch { missing: String },

    #[error("cannot mutate immutable binding: ${0}")]
    ImmutableBinding(String),

    #[error("duplicate definition: {0}")]
    DuplicateDefinition(String),

    #[error("pattern type mismatch: expected {expected}, found pattern for {found}")]
    PatternMismatch { expected: Type, found: String },

    #[error("cannot unify types: {0} and {1}")]
    UnificationFailure(Type, Type),

    #[error("{0}")]
    Other(String),
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at {}..{}", self.kind, self.span.start, self.span.end)
    }
}

/// Collection of type errors from a checking pass.
#[derive(Debug, Clone)]
pub struct TypeErrors {
    pub errors: Vec<TypeError>,
}

impl TypeErrors {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn push(&mut self, kind: TypeErrorKind, span: Span) {
        self.errors.push(TypeError { kind, span });
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn len(&self) -> usize {
        self.errors.len()
    }
}

impl Default for TypeErrors {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TypeErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, err) in self.errors.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{err}")?;
        }
        Ok(())
    }
}

impl std::error::Error for TypeErrors {}
