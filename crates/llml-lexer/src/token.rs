use logos::Logos;
use std::fmt;

/// Span in source code: byte offset range [start, end)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

/// A token with its span in the source.
#[derive(Debug, Clone, PartialEq)]
pub struct Spanned {
    pub token: Token,
    pub span: Span,
}

/// All tokens in the LLML language.
///
/// LLML uses modified s-expressions with sigils for identifier disambiguation.
/// Sigils: @ (type), $ (variable), # (module), ! (effect), & (ref), ~ (linear), ^ (generic)
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n]+")]
pub enum Token {
    // ── Delimiters ──────────────────────────────
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,

    // ── Keywords ────────────────────────────────
    #[token("fn")]
    Fn,
    #[token("let")]
    Let,
    #[token("if")]
    If,
    #[token("mat")]
    Mat,
    #[token("do")]
    Do,
    #[token("ty")]
    Ty,
    #[token("mod")]
    Mod,
    #[token("ef")]
    Ef,
    #[token("use")]
    Use,
    #[token("pub")]
    Pub,
    #[token("mut")]
    Mut,
    #[token("sum")]
    Sum,
    #[token("prod")]
    Prod,
    #[token("ret")]
    Ret,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("nil")]
    Nil,
    #[token("for")]
    For,
    #[token("in")]
    In,
    #[token("lazy")]
    Lazy,
    #[token("region")]
    Region,
    #[token("set")]
    Set,
    #[token("lin")]
    Lin,
    #[token("_")]
    Underscore,

    // ── Operators ───────────────────────────────
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("=")]
    Eq,
    #[token("!=")]
    Neq,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("<=")]
    Le,
    #[token(">=")]
    Ge,
    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token("!")]
    Bang,
    #[token("->")]
    Arrow,
    #[token(":")]
    Colon,
    #[token(".")]
    Dot,

    // ── Sigils ──────────────────────────────────
    // Type sigil: @Name
    #[regex(r"@[A-Z][a-zA-Z0-9_]*", |lex| lex.slice()[1..].to_string())]
    TypeSigil(String),

    // Variable sigil: $name
    #[regex(r"\$[a-z_][a-zA-Z0-9_]*", |lex| lex.slice()[1..].to_string())]
    VarSigil(String),

    // Module sigil: #name
    #[regex(r"#[a-z_][a-zA-Z0-9_]*", |lex| lex.slice()[1..].to_string())]
    ModSigil(String),

    // Effect sigil: !name (only after effect-related contexts)
    // We use !lowercase to distinguish from the Bang operator
    #[regex(r"![a-z_][a-zA-Z0-9_]*", |lex| lex.slice()[1..].to_string())]
    EffectSigil(String),

    // Reference sigil: &
    #[token("&")]
    Ampersand,

    // Linear/owned sigil: ~
    #[token("~")]
    Tilde,

    // Generic sigil: ^T
    #[regex(r"\^[A-Z][a-zA-Z0-9_]*", |lex| lex.slice()[1..].to_string())]
    GenericSigil(String),

    // ── Literals ────────────────────────────────
    // Float literal (must come before integer to match first)
    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse::<f64>().ok())]
    Float(f64),

    // Integer literal
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    Integer(i64),

    // String literal
    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        // Strip surrounding quotes
        s[1..s.len()-1].to_string()
    })]
    Str(String),

    // ── Comments (skipped) ──────────────────────
    #[regex(r";;[^\n]*", logos::skip)]
    Comment,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::Fn => write!(f, "fn"),
            Token::Let => write!(f, "let"),
            Token::If => write!(f, "if"),
            Token::Mat => write!(f, "mat"),
            Token::Do => write!(f, "do"),
            Token::Ty => write!(f, "ty"),
            Token::Mod => write!(f, "mod"),
            Token::Ef => write!(f, "ef"),
            Token::Use => write!(f, "use"),
            Token::Pub => write!(f, "pub"),
            Token::Mut => write!(f, "mut"),
            Token::Sum => write!(f, "sum"),
            Token::Prod => write!(f, "prod"),
            Token::Ret => write!(f, "ret"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::Nil => write!(f, "nil"),
            Token::For => write!(f, "for"),
            Token::In => write!(f, "in"),
            Token::Lazy => write!(f, "lazy"),
            Token::Region => write!(f, "region"),
            Token::Set => write!(f, "set"),
            Token::Lin => write!(f, "lin"),
            Token::Underscore => write!(f, "_"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            Token::Eq => write!(f, "="),
            Token::Neq => write!(f, "!="),
            Token::Lt => write!(f, "<"),
            Token::Gt => write!(f, ">"),
            Token::Le => write!(f, "<="),
            Token::Ge => write!(f, ">="),
            Token::And => write!(f, "&&"),
            Token::Or => write!(f, "||"),
            Token::Bang => write!(f, "!"),
            Token::Arrow => write!(f, "->"),
            Token::Colon => write!(f, ":"),
            Token::Dot => write!(f, "."),
            Token::TypeSigil(s) => write!(f, "@{s}"),
            Token::VarSigil(s) => write!(f, "${s}"),
            Token::ModSigil(s) => write!(f, "#{s}"),
            Token::EffectSigil(s) => write!(f, "!{s}"),
            Token::Ampersand => write!(f, "&"),
            Token::Tilde => write!(f, "~"),
            Token::GenericSigil(s) => write!(f, "^{s}"),
            Token::Float(n) => write!(f, "{n}"),
            Token::Integer(n) => write!(f, "{n}"),
            Token::Str(s) => write!(f, "\"{s}\""),
            Token::Comment => write!(f, ""),
        }
    }
}

/// Tokenize LLML source code into a sequence of spanned tokens.
pub fn tokenize(source: &str) -> Result<Vec<Spanned>, LexError> {
    let mut lexer = Token::lexer(source);
    let mut tokens = Vec::new();

    while let Some(result) = lexer.next() {
        let span = lexer.span();
        match result {
            Ok(token) => {
                tokens.push(Spanned {
                    token,
                    span: Span::new(span.start, span.end),
                });
            }
            Err(()) => {
                return Err(LexError {
                    span: Span::new(span.start, span.end),
                    source_fragment: source[span.start..span.end].to_string(),
                });
            }
        }
    }

    Ok(tokens)
}

/// Error produced during lexing.
#[derive(Debug, Clone, thiserror::Error)]
#[error("unexpected character `{source_fragment}` at byte {}", span.start)]
pub struct LexError {
    pub span: Span,
    pub source_fragment: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let tokens =
            tokenize("(fn $add (: @I32 @I32 -> @I32) ($a : @I32) ($b : @I32) (+ $a $b))").unwrap();
        assert_eq!(tokens[0].token, Token::LParen);
        assert_eq!(tokens[1].token, Token::Fn);
        assert_eq!(tokens[2].token, Token::VarSigil("add".to_string()));
    }

    #[test]
    fn test_literals() {
        let tokens = tokenize("42 3.14 \"hello\"").unwrap();
        assert_eq!(tokens[0].token, Token::Integer(42));
        assert_eq!(tokens[1].token, Token::Float(3.14));
        assert_eq!(tokens[2].token, Token::Str("hello".to_string()));
    }

    #[test]
    fn test_sigils() {
        let tokens = tokenize("@Vec $x #std !io ^T").unwrap();
        assert_eq!(tokens[0].token, Token::TypeSigil("Vec".to_string()));
        assert_eq!(tokens[1].token, Token::VarSigil("x".to_string()));
        assert_eq!(tokens[2].token, Token::ModSigil("std".to_string()));
        assert_eq!(tokens[3].token, Token::EffectSigil("io".to_string()));
        assert_eq!(tokens[4].token, Token::GenericSigil("T".to_string()));
    }

    #[test]
    fn test_comments_skipped() {
        let tokens = tokenize(";; this is a comment\n42").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token, Token::Integer(42));
    }

    #[test]
    fn test_operators() {
        let tokens = tokenize("+ - * / <= >= != && ||").unwrap();
        assert_eq!(tokens[0].token, Token::Plus);
        assert_eq!(tokens[1].token, Token::Minus);
        assert_eq!(tokens[2].token, Token::Star);
        assert_eq!(tokens[3].token, Token::Slash);
        assert_eq!(tokens[4].token, Token::Le);
        assert_eq!(tokens[5].token, Token::Ge);
        assert_eq!(tokens[6].token, Token::Neq);
        assert_eq!(tokens[7].token, Token::And);
        assert_eq!(tokens[8].token, Token::Or);
    }

    #[test]
    fn test_full_function() {
        let src = r#"(fn $fib (: @I32 -> @I32)
  ($n : @I32)
  (if (<= $n 1) $n (+ ($fib (- $n 1)) ($fib (- $n 2)))))"#;
        let tokens = tokenize(src).unwrap();
        // Should tokenize without error
        assert!(tokens.len() > 20);
    }

    #[test]
    fn test_let_binding() {
        let tokens = tokenize("(let $x : @I32 42)").unwrap();
        assert_eq!(tokens[0].token, Token::LParen);
        assert_eq!(tokens[1].token, Token::Let);
        assert_eq!(tokens[2].token, Token::VarSigil("x".to_string()));
        assert_eq!(tokens[3].token, Token::Colon);
        assert_eq!(tokens[4].token, Token::TypeSigil("I32".to_string()));
        assert_eq!(tokens[5].token, Token::Integer(42));
        assert_eq!(tokens[6].token, Token::RParen);
    }

    #[test]
    fn test_keywords() {
        let tokens = tokenize("fn let if mat do ty mod ef use pub mut sum prod ret").unwrap();
        assert_eq!(tokens[0].token, Token::Fn);
        assert_eq!(tokens[1].token, Token::Let);
        assert_eq!(tokens[2].token, Token::If);
        assert_eq!(tokens[3].token, Token::Mat);
        assert_eq!(tokens[4].token, Token::Do);
        assert_eq!(tokens[5].token, Token::Ty);
        assert_eq!(tokens[6].token, Token::Mod);
        assert_eq!(tokens[7].token, Token::Ef);
        assert_eq!(tokens[8].token, Token::Use);
        assert_eq!(tokens[9].token, Token::Pub);
        assert_eq!(tokens[10].token, Token::Mut);
        assert_eq!(tokens[11].token, Token::Sum);
        assert_eq!(tokens[12].token, Token::Prod);
        assert_eq!(tokens[13].token, Token::Ret);
    }
}
