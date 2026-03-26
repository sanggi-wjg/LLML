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

/// A collection of parser errors — returned when error recovery is used.
#[derive(Debug, Clone, thiserror::Error)]
#[error("{}", format_errors(&self.errors))]
pub struct ParseErrors {
    pub errors: Vec<ParseError>,
}

impl ParseErrors {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn push(&mut self, error: ParseError) {
        self.errors.push(error);
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn len(&self) -> usize {
        self.errors.len()
    }
}

impl Default for ParseErrors {
    fn default() -> Self {
        Self::new()
    }
}

fn format_errors(errors: &[ParseError]) -> String {
    errors
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}
