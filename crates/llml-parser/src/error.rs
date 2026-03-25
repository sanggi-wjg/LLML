use llml_lexer::{Span, Token};

/// Parser error.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ParseError {
    #[error("unexpected token `{found}` at byte {}, expected {expected}", span.start)]
    Unexpected {
        found: String,
        expected: String,
        span: Span,
    },

    #[error("unexpected end of input, expected {expected}")]
    UnexpectedEof { expected: String },

    #[error("lex error: {0}")]
    Lex(#[from] llml_lexer::LexError),
}

impl ParseError {
    pub fn unexpected(token: &Token, span: Span, expected: &str) -> Self {
        Self::Unexpected {
            found: token.to_string(),
            expected: expected.to_string(),
            span,
        }
    }

    pub fn eof(expected: &str) -> Self {
        Self::UnexpectedEof {
            expected: expected.to_string(),
        }
    }
}
