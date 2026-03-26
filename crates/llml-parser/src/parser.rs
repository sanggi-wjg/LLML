use crate::ast::*;
use crate::error::ParseError;
use llml_lexer::{Span, Spanned, Token};

type Result<T> = std::result::Result<T, ParseError>;

/// Recursive descent parser for LLML.
///
/// LLML syntax is s-expression based, so every compound form starts with `(`
/// and a keyword. This makes the parser straightforward: peek at the first
/// token after `(` to decide which form to parse.
pub struct Parser {
    tokens: Vec<Spanned>,
    pos: usize,
    /// Accumulated errors during error-recovery parsing.
    errors: Vec<ParseError>,
}

impl Parser {
    /// Create a new parser from a token stream.
    pub fn new(tokens: Vec<Spanned>) -> Self {
        Self {
            tokens,
            pos: 0,
            errors: Vec::new(),
        }
    }

    /// Parse a complete program (sequence of top-level expressions/declarations).
    ///
    /// Uses error recovery: if a declaration fails to parse, the parser
    /// skips to the next balanced parenthesis and continues.
    pub fn parse_program(&mut self) -> Result<Program> {
        let mut decls = Vec::new();
        while !self.is_eof() {
            match self.parse_decl() {
                Ok(decl) => decls.push(decl),
                Err(e) => {
                    self.errors.push(e);
                    self.synchronize();
                }
            }
        }
        if self.errors.is_empty() {
            Ok(Program { decls })
        } else if decls.is_empty() {
            // No successful declarations — return the first error
            Err(self.errors.remove(0))
        } else {
            // Partial success — return what we have
            // Errors are accessible via `parser.errors()`
            Ok(Program { decls })
        }
    }

    /// Get accumulated errors from error-recovery parsing.
    pub fn errors(&self) -> &[ParseError] {
        &self.errors
    }

    /// Skip tokens until we reach a position where parsing can resume.
    ///
    /// Strategy: skip to the next balanced closing paren at top level.
    fn synchronize(&mut self) {
        let mut depth = 0i32;
        while !self.is_eof() {
            match self.peek_token() {
                Some(Token::LParen) => {
                    if depth == 0 {
                        // Found start of a new top-level form
                        return;
                    }
                    depth += 1;
                    self.pos += 1;
                }
                Some(Token::RParen) => {
                    depth -= 1;
                    self.pos += 1;
                    if depth <= 0 {
                        return;
                    }
                }
                _ => {
                    self.pos += 1;
                }
            }
        }
    }

    // ── Helpers ──────────────────────────────────

    fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn peek(&self) -> Option<&Spanned> {
        self.tokens.get(self.pos)
    }

    fn peek_token(&self) -> Option<&Token> {
        self.peek().map(|s| &s.token)
    }

    fn advance(&mut self) -> Result<&Spanned> {
        if self.is_eof() {
            return Err(ParseError::eof("token"));
        }
        let spanned = &self.tokens[self.pos];
        self.pos += 1;
        Ok(spanned)
    }

    fn expect(&mut self, expected: &Token) -> Result<Span> {
        let spanned = self.advance()?;
        if &spanned.token == expected {
            Ok(spanned.span)
        } else {
            Err(ParseError::unexpected(
                &spanned.token,
                spanned.span,
                &expected.to_string(),
            ))
        }
    }

    fn expect_lparen(&mut self) -> Result<Span> {
        self.expect(&Token::LParen)
    }

    fn expect_rparen(&mut self) -> Result<Span> {
        self.expect(&Token::RParen)
    }

    fn expect_var_sigil(&mut self) -> Result<(String, Span)> {
        let spanned = self.advance()?;
        match &spanned.token {
            Token::VarSigil(name) => Ok((name.clone(), spanned.span)),
            _ => Err(ParseError::unexpected(
                &spanned.token,
                spanned.span,
                "$variable",
            )),
        }
    }

    fn expect_type_sigil(&mut self) -> Result<(String, Span)> {
        let spanned = self.advance()?;
        match &spanned.token {
            Token::TypeSigil(name) => Ok((name.clone(), spanned.span)),
            _ => Err(ParseError::unexpected(
                &spanned.token,
                spanned.span,
                "@Type",
            )),
        }
    }

    fn current_span(&self) -> Span {
        if let Some(s) = self.peek() {
            s.span
        } else if let Some(last) = self.tokens.last() {
            Span::new(last.span.end, last.span.end)
        } else {
            Span::new(0, 0)
        }
    }

    fn span_from(&self, start: Span) -> Span {
        let end = if self.pos > 0 {
            self.tokens[self.pos - 1].span.end
        } else {
            start.end
        };
        Span::new(start.start, end)
    }

    // ── Declaration parsing ─────────────────────

    fn parse_decl(&mut self) -> Result<Node<Decl>> {
        let start = self.current_span();

        // Every declaration is a parenthesized form
        if self.peek_token() == Some(&Token::LParen) {
            self.expect_lparen()?;
            let decl = self.parse_decl_inner(start)?;
            Ok(decl)
        } else {
            // Bare expression at top level
            let expr = self.parse_expr()?;
            let span = expr.span;
            Ok(Node::new(Decl::Expr(expr.inner), span))
        }
    }

    fn parse_decl_inner(&mut self, start: Span) -> Result<Node<Decl>> {
        let keyword = self.peek_token().cloned();
        match keyword {
            Some(Token::Fn) => {
                self.advance()?;
                let fn_decl = self.parse_fn_decl()?;
                let end = self.expect_rparen()?;
                let span = Span::new(start.start, end.end);
                Ok(Node::new(Decl::Fn(fn_decl), span))
            }
            Some(Token::Let) => {
                self.advance()?;
                let let_decl = self.parse_let_decl()?;
                let end = self.expect_rparen()?;
                let span = Span::new(start.start, end.end);
                Ok(Node::new(Decl::Let(let_decl), span))
            }
            Some(Token::Ty) => {
                self.advance()?;
                let type_def = self.parse_type_def()?;
                let end = self.expect_rparen()?;
                let span = Span::new(start.start, end.end);
                Ok(Node::new(Decl::TypeDef(type_def), span))
            }
            Some(Token::Mod) => {
                self.advance()?;
                let mod_decl = self.parse_module_decl()?;
                let end = self.expect_rparen()?;
                let span = Span::new(start.start, end.end);
                Ok(Node::new(Decl::Module(mod_decl), span))
            }
            Some(Token::Pub) => {
                self.advance()?;
                let inner = self.parse_decl()?;
                let end = self.expect_rparen()?;
                let span = Span::new(start.start, end.end);
                Ok(Node::new(Decl::Pub(Box::new(inner)), span))
            }
            _ => {
                // It's an expression in parens — rewind by treating as expr
                let expr = self.parse_expr_after_lparen(start)?;
                let span = expr.span;
                Ok(Node::new(Decl::Expr(expr.inner), span))
            }
        }
    }

    // ── Function declaration ────────────────────

    fn parse_fn_decl(&mut self) -> Result<FnDecl> {
        // $name
        let (name, _) = self.expect_var_sigil()?;

        // Optional type signature: (: ...)
        let type_sig = if self.peek_token() == Some(&Token::LParen) {
            // Peek ahead to see if it's (: ...)
            if self.is_type_sig_next() {
                Some(self.parse_type_sig()?)
            } else {
                None
            }
        } else {
            None
        };

        // Parameters: ($name : @Type) ...
        let mut params = Vec::new();
        while self.peek_token() == Some(&Token::LParen) && self.is_param_next() {
            params.push(self.parse_param()?);
        }

        // Body: single expression (possibly a parenthesized form)
        let body = self.parse_expr()?;

        Ok(FnDecl {
            name,
            type_sig,
            params,
            body: Box::new(body),
        })
    }

    /// Check if the next form is (: ...) — a type signature.
    fn is_type_sig_next(&self) -> bool {
        if self.pos + 1 < self.tokens.len() {
            self.tokens[self.pos].token == Token::LParen
                && self.tokens[self.pos + 1].token == Token::Colon
        } else {
            false
        }
    }

    /// Check if the next form is ($var : @Type) — a parameter.
    /// Uses 3-token lookahead to distinguish from function calls like ($f $x).
    fn is_param_next(&self) -> bool {
        if self.pos + 2 < self.tokens.len() {
            self.tokens[self.pos].token == Token::LParen
                && matches!(
                    self.tokens[self.pos + 1].token,
                    Token::VarSigil(_) | Token::Mut
                )
                && (self.tokens[self.pos + 2].token == Token::Colon
                    || (self.tokens[self.pos + 1].token == Token::Mut
                        && matches!(
                            self.tokens.get(self.pos + 2).map(|s| &s.token),
                            Some(Token::VarSigil(_))
                        )))
        } else {
            false
        }
    }

    fn parse_type_sig(&mut self) -> Result<TypeSig> {
        self.expect_lparen()?;
        self.expect(&Token::Colon)?;

        let mut param_types = Vec::new();
        let mut return_type = None;
        let mut effects = Vec::new();

        loop {
            match self.peek_token() {
                Some(Token::RParen) => {
                    self.advance()?;
                    break;
                }
                Some(Token::Arrow) => {
                    self.advance()?;
                    // Everything after -> is the return type, then effects
                    return_type = Some(self.parse_type_expr()?);
                    // Parse effects: ! !name1 !name2 ...
                    while let Some(Token::EffectSigil(_)) = self.peek_token() {
                        if let Token::EffectSigil(name) = &self.advance()?.token.clone() {
                            effects.push(name.clone());
                        }
                    }
                    // Also check for effect types specified as types after !
                    // (for !err @Str pattern)
                }
                Some(Token::TypeSigil(_))
                | Some(Token::GenericSigil(_))
                | Some(Token::LParen)
                | Some(Token::Tilde)
                | Some(Token::Ampersand) => {
                    param_types.push(self.parse_type_expr()?);
                }
                _ => {
                    let spanned = self.advance()?;
                    return Err(ParseError::unexpected(
                        &spanned.token,
                        spanned.span,
                        "type expression, ->, or )",
                    ));
                }
            }
        }

        let return_type = return_type.unwrap_or_else(|| {
            let span = self.current_span();
            Node::new(TypeExpr::Named("Nil".to_string()), span)
        });

        Ok(TypeSig {
            param_types,
            return_type: Box::new(return_type),
            effects,
        })
    }

    fn parse_param(&mut self) -> Result<Param> {
        let start = self.expect_lparen()?;

        // Check for mut
        let is_mut = if self.peek_token() == Some(&Token::Mut) {
            self.advance()?;
            true
        } else {
            false
        };

        let (name, _) = self.expect_var_sigil()?;
        self.expect(&Token::Colon)?;
        let ty = self.parse_type_expr()?;
        let end = self.expect_rparen()?;

        Ok(Param {
            name,
            ty,
            is_mut,
            span: Span::new(start.start, end.end),
        })
    }

    // ── Type expressions ────────────────────────

    fn parse_type_expr(&mut self) -> Result<Node<TypeExpr>> {
        let start = self.current_span();
        match self.peek_token().cloned() {
            Some(Token::TypeSigil(name)) => {
                self.advance()?;
                // Check for type application: @List ^T (only generic params, not other types)
                let mut args = Vec::new();
                while matches!(self.peek_token(), Some(Token::GenericSigil(_))) {
                    args.push(self.parse_type_expr()?);
                }
                let span = self.span_from(start);
                if args.is_empty() {
                    Ok(Node::new(TypeExpr::Named(name), span))
                } else {
                    Ok(Node::new(TypeExpr::App(name, args), span))
                }
            }
            Some(Token::GenericSigil(name)) => {
                self.advance()?;
                let span = self.span_from(start);
                Ok(Node::new(TypeExpr::Generic(name), span))
            }
            Some(Token::Tilde) => {
                self.advance()?;
                let inner = self.parse_type_expr()?;
                let span = self.span_from(start);
                Ok(Node::new(TypeExpr::Linear(Box::new(inner)), span))
            }
            Some(Token::Ampersand) => {
                self.advance()?;
                let inner = self.parse_type_expr()?;
                let span = self.span_from(start);
                Ok(Node::new(TypeExpr::Ref(Box::new(inner)), span))
            }
            Some(Token::LParen) => {
                // Function type: (: @I32 -> @I32)
                self.parse_fn_type_expr()
            }
            _ => {
                let spanned = self.advance()?;
                Err(ParseError::unexpected(
                    &spanned.token,
                    spanned.span,
                    "type expression",
                ))
            }
        }
    }

    fn parse_fn_type_expr(&mut self) -> Result<Node<TypeExpr>> {
        let start = self.expect_lparen()?;
        self.expect(&Token::Colon)?;

        let mut params = Vec::new();
        while !matches!(self.peek_token(), Some(Token::Arrow) | Some(Token::RParen)) {
            params.push(self.parse_type_expr()?);
        }

        let ret = if self.peek_token() == Some(&Token::Arrow) {
            self.advance()?;
            self.parse_type_expr()?
        } else {
            let span = self.current_span();
            Node::new(TypeExpr::Named("Nil".to_string()), span)
        };

        let end = self.expect_rparen()?;
        let span = Span::new(start.start, end.end);
        Ok(Node::new(TypeExpr::FnType(params, Box::new(ret)), span))
    }

    // ── Let binding ─────────────────────────────

    fn parse_let_decl(&mut self) -> Result<LetDecl> {
        let is_mut = if self.peek_token() == Some(&Token::Mut) {
            self.advance()?;
            true
        } else {
            false
        };

        let (name, _) = self.expect_var_sigil()?;

        // Optional type: : @Type
        let ty = if self.peek_token() == Some(&Token::Colon) {
            self.advance()?;
            Some(self.parse_type_expr()?)
        } else {
            None
        };

        let value = self.parse_expr()?;

        Ok(LetDecl {
            name,
            is_mut,
            ty,
            value: Box::new(value),
        })
    }

    // ── Type definition ─────────────────────────

    fn parse_type_def(&mut self) -> Result<TypeDef> {
        let (name, _) = self.expect_type_sigil()?;

        // Optional generic params: (: ^T ^U)
        let mut params = Vec::new();
        if self.is_type_sig_next() {
            self.expect_lparen()?;
            self.expect(&Token::Colon)?;
            while let Some(Token::GenericSigil(_)) = self.peek_token() {
                if let Token::GenericSigil(p) = &self.advance()?.token.clone() {
                    params.push(p.clone());
                }
            }
            self.expect_rparen()?;
        }

        // Body: (sum ...) or (prod ...) or (lin)
        self.expect_lparen()?;
        let body = match self.peek_token() {
            Some(Token::Sum) => {
                self.advance()?;
                let mut variants = Vec::new();
                while self.peek_token() != Some(&Token::RParen) {
                    variants.push(self.parse_variant()?);
                }
                self.expect_rparen()?;
                TypeDefBody::Sum(variants)
            }
            Some(Token::Prod) => {
                self.advance()?;
                let mut fields = Vec::new();
                while self.peek_token() != Some(&Token::RParen) {
                    fields.push(self.parse_field()?);
                }
                self.expect_rparen()?;
                TypeDefBody::Prod(fields)
            }
            Some(Token::Lin) => {
                self.advance()?;
                self.expect_rparen()?;
                TypeDefBody::Linear
            }
            _ => {
                let spanned = self.advance()?;
                return Err(ParseError::unexpected(
                    &spanned.token,
                    spanned.span,
                    "sum, prod, or lin",
                ));
            }
        };

        Ok(TypeDef { name, params, body })
    }

    fn parse_variant(&mut self) -> Result<Variant> {
        // Could be just @Name or (@Name $field1 : @Type1 ...)
        match self.peek_token() {
            Some(Token::TypeSigil(_)) => {
                let (name, span) = self.expect_type_sigil()?;
                Ok(Variant {
                    name,
                    fields: vec![],
                    span,
                })
            }
            Some(Token::LParen) => {
                let start = self.expect_lparen()?;
                let (name, _) = self.expect_type_sigil()?;
                let mut fields = Vec::new();
                while self.peek_token() != Some(&Token::RParen) {
                    fields.push(self.parse_field()?);
                }
                let end = self.expect_rparen()?;
                Ok(Variant {
                    name,
                    fields,
                    span: Span::new(start.start, end.end),
                })
            }
            _ => {
                let spanned = self.advance()?;
                Err(ParseError::unexpected(
                    &spanned.token,
                    spanned.span,
                    "@VariantName or (@VariantName ...)",
                ))
            }
        }
    }

    fn parse_field(&mut self) -> Result<Field> {
        let (name, _) = self.expect_var_sigil()?;
        self.expect(&Token::Colon)?;
        let ty = self.parse_type_expr()?;
        Ok(Field { name, ty })
    }

    // ── Module declaration ──────────────────────

    fn parse_module_decl(&mut self) -> Result<ModuleDecl> {
        let spanned = self.advance()?;
        let name = match &spanned.token {
            Token::ModSigil(n) => n.clone(),
            _ => {
                return Err(ParseError::unexpected(
                    &spanned.token,
                    spanned.span,
                    "#module_name",
                ));
            }
        };

        let mut decls = Vec::new();
        while self.peek_token() != Some(&Token::RParen) {
            decls.push(self.parse_decl()?);
        }

        Ok(ModuleDecl { name, decls })
    }

    // ── Expression parsing ──────────────────────

    pub fn parse_expr(&mut self) -> Result<Node<Expr>> {
        let start = self.current_span();
        match self.peek_token().cloned() {
            Some(Token::LParen) => {
                self.advance()?;
                self.parse_expr_after_lparen(start)
            }
            Some(Token::Integer(n)) => {
                self.advance()?;
                Ok(Node::new(Expr::IntLit(n), self.span_from(start)))
            }
            Some(Token::Float(n)) => {
                self.advance()?;
                Ok(Node::new(Expr::FloatLit(n), self.span_from(start)))
            }
            Some(Token::Str(s)) => {
                self.advance()?;
                Ok(Node::new(Expr::StrLit(s), self.span_from(start)))
            }
            Some(Token::True) => {
                self.advance()?;
                Ok(Node::new(Expr::BoolLit(true), self.span_from(start)))
            }
            Some(Token::False) => {
                self.advance()?;
                Ok(Node::new(Expr::BoolLit(false), self.span_from(start)))
            }
            Some(Token::Nil) => {
                self.advance()?;
                Ok(Node::new(Expr::NilLit, self.span_from(start)))
            }
            Some(Token::VarSigil(name)) => {
                self.advance()?;
                Ok(Node::new(Expr::Var(name), self.span_from(start)))
            }
            Some(Token::TypeSigil(name)) => {
                self.advance()?;
                Ok(Node::new(
                    Expr::TypeConstructor(name),
                    self.span_from(start),
                ))
            }
            Some(Token::ModSigil(mod_name)) => {
                self.advance()?;
                // Expect .$ for qualified access: #mod.$name
                self.expect(&Token::Dot)?;
                let (var_name, _) = self.expect_var_sigil()?;
                Ok(Node::new(
                    Expr::ModAccess(mod_name, var_name),
                    self.span_from(start),
                ))
            }
            None => Err(ParseError::eof("expression")),
            _ => {
                let spanned = self.advance()?;
                Err(ParseError::unexpected(
                    &spanned.token,
                    spanned.span,
                    "expression",
                ))
            }
        }
    }

    /// Parse an expression after the opening `(` has been consumed.
    fn parse_expr_after_lparen(&mut self, start: Span) -> Result<Node<Expr>> {
        match self.peek_token().cloned() {
            // (if $cond $then $else)
            Some(Token::If) => {
                self.advance()?;
                let cond = self.parse_expr()?;
                let then_branch = self.parse_expr()?;
                let else_branch = self.parse_expr()?;
                let end = self.expect_rparen()?;
                let span = Span::new(start.start, end.end);
                Ok(Node::new(
                    Expr::If(Box::new(cond), Box::new(then_branch), Box::new(else_branch)),
                    span,
                ))
            }

            // (let $name : @Type $value)  — let as expression with implicit rest
            Some(Token::Let) => {
                self.advance()?;
                let let_decl = self.parse_let_decl()?;
                let end = self.expect_rparen()?;
                let span = Span::new(start.start, end.end);
                // In expression context, let has no continuation — return as part of do
                let nil_span = Span::new(end.end, end.end);
                Ok(Node::new(
                    Expr::Let(
                        Box::new(let_decl),
                        Box::new(Node::new(Expr::NilLit, nil_span)),
                    ),
                    span,
                ))
            }

            // (do $expr1 $expr2 ... $exprN)
            Some(Token::Do) => {
                self.advance()?;
                let mut exprs = Vec::new();
                while self.peek_token() != Some(&Token::RParen) {
                    exprs.push(self.parse_decl_or_expr_in_do()?);
                }
                let end = self.expect_rparen()?;
                let span = Span::new(start.start, end.end);
                Ok(Node::new(Expr::Do(exprs), span))
            }

            // (mat $expr ($pat1 $body1) ($pat2 $body2) ...)
            Some(Token::Mat) => {
                self.advance()?;
                let scrutinee = self.parse_expr()?;
                let mut arms = Vec::new();
                while self.peek_token() == Some(&Token::LParen) {
                    arms.push(self.parse_match_arm()?);
                }
                let end = self.expect_rparen()?;
                let span = Span::new(start.start, end.end);
                Ok(Node::new(Expr::Match(Box::new(scrutinee), arms), span))
            }

            // (ret $expr)
            Some(Token::Ret) => {
                self.advance()?;
                let expr = self.parse_expr()?;
                let end = self.expect_rparen()?;
                let span = Span::new(start.start, end.end);
                Ok(Node::new(Expr::Return(Box::new(expr)), span))
            }

            // (set $var $expr)
            Some(Token::Set) => {
                self.advance()?;
                let (name, _) = self.expect_var_sigil()?;
                let expr = self.parse_expr()?;
                let end = self.expect_rparen()?;
                let span = Span::new(start.start, end.end);
                Ok(Node::new(Expr::SetExpr(name, Box::new(expr)), span))
            }

            // (fn $name ...) — anonymous/local function
            Some(Token::Fn) => {
                self.advance()?;
                let fn_decl = self.parse_fn_decl()?;
                let end = self.expect_rparen()?;
                let span = Span::new(start.start, end.end);
                Ok(Node::new(Expr::FnExpr(fn_decl), span))
            }

            // Binary operators: (+ $a $b), (* $a $b), etc.
            Some(Token::Plus) => self.parse_binop(start, BinOp::Add),
            Some(Token::Minus) => {
                self.advance()?;
                // Could be unary (- $x) or binary (- $a $b)
                let first = self.parse_expr()?;
                if self.peek_token() == Some(&Token::RParen) {
                    let end = self.expect_rparen()?;
                    let span = Span::new(start.start, end.end);
                    Ok(Node::new(
                        Expr::UnaryOp(UnaryOp::Neg, Box::new(first)),
                        span,
                    ))
                } else {
                    let second = self.parse_expr()?;
                    let end = self.expect_rparen()?;
                    let span = Span::new(start.start, end.end);
                    Ok(Node::new(
                        Expr::BinOp(BinOp::Sub, Box::new(first), Box::new(second)),
                        span,
                    ))
                }
            }
            Some(Token::Star) => self.parse_binop(start, BinOp::Mul),
            Some(Token::Slash) => self.parse_binop(start, BinOp::Div),
            Some(Token::Percent) => self.parse_binop(start, BinOp::Mod),
            Some(Token::Eq) => self.parse_binop(start, BinOp::Eq),
            Some(Token::Neq) => self.parse_binop(start, BinOp::Neq),
            Some(Token::Lt) => self.parse_binop(start, BinOp::Lt),
            Some(Token::Gt) => self.parse_binop(start, BinOp::Gt),
            Some(Token::Le) => self.parse_binop(start, BinOp::Le),
            Some(Token::Ge) => self.parse_binop(start, BinOp::Ge),
            Some(Token::And) => self.parse_binop(start, BinOp::And),
            Some(Token::Or) => self.parse_binop(start, BinOp::Or),

            // Function call: ($f $arg1 $arg2 ...) or (@Constructor $arg1 ...)
            _ => {
                let callee = self.parse_expr()?;
                let mut args = Vec::new();
                while self.peek_token() != Some(&Token::RParen) {
                    args.push(self.parse_expr()?);
                }
                let end = self.expect_rparen()?;
                let span = Span::new(start.start, end.end);
                if args.is_empty() {
                    // Just a parenthesized expression
                    Ok(Node::new(callee.inner, span))
                } else {
                    Ok(Node::new(Expr::Call(Box::new(callee), args), span))
                }
            }
        }
    }

    fn parse_binop(&mut self, start: Span, op: BinOp) -> Result<Node<Expr>> {
        self.advance()?; // consume operator
        let lhs = self.parse_expr()?;
        let rhs = self.parse_expr()?;
        let end = self.expect_rparen()?;
        let span = Span::new(start.start, end.end);
        Ok(Node::new(
            Expr::BinOp(op, Box::new(lhs), Box::new(rhs)),
            span,
        ))
    }

    /// In a do block, we can have let bindings, fn/ty declarations, or expressions.
    fn parse_decl_or_expr_in_do(&mut self) -> Result<Node<Expr>> {
        if self.peek_token() == Some(&Token::LParen) && self.pos + 1 < self.tokens.len() {
            let next = &self.tokens[self.pos + 1].token;
            match next {
                Token::Let => {
                    let start = self.current_span();
                    self.expect_lparen()?;
                    self.advance()?; // consume 'let'
                    let let_decl = self.parse_let_decl()?;
                    let end = self.expect_rparen()?;
                    let span = Span::new(start.start, end.end);
                    let nil_span = Span::new(end.end, end.end);
                    return Ok(Node::new(
                        Expr::Let(
                            Box::new(let_decl),
                            Box::new(Node::new(Expr::NilLit, nil_span)),
                        ),
                        span,
                    ));
                }
                Token::Fn => {
                    // Parse fn declaration and register it as a FnExpr
                    let start = self.current_span();
                    self.expect_lparen()?;
                    self.advance()?; // consume 'fn'
                    let fn_decl = self.parse_fn_decl()?;
                    let end = self.expect_rparen()?;
                    let span = Span::new(start.start, end.end);
                    return Ok(Node::new(Expr::FnExpr(fn_decl), span));
                }
                Token::Ty => {
                    // Type definitions are no-ops at runtime, skip them
                    let start = self.current_span();
                    self.expect_lparen()?;
                    self.advance()?; // consume 'ty'
                    let _type_def = self.parse_type_def()?;
                    let end = self.expect_rparen()?;
                    let span = Span::new(start.start, end.end);
                    return Ok(Node::new(Expr::NilLit, span));
                }
                _ => {}
            }
        }
        self.parse_expr()
    }

    // ── Match arm ───────────────────────────────

    fn parse_match_arm(&mut self) -> Result<MatchArm> {
        let start = self.expect_lparen()?;
        let pattern = self.parse_pattern()?;
        let body = self.parse_expr()?;
        let end = self.expect_rparen()?;
        let span = Span::new(start.start, end.end);
        Ok(MatchArm {
            pattern,
            body,
            span,
        })
    }

    // ── Pattern parsing ─────────────────────────

    fn parse_pattern(&mut self) -> Result<Node<Pattern>> {
        let start = self.current_span();
        match self.peek_token().cloned() {
            Some(Token::Underscore) => {
                self.advance()?;
                Ok(Node::new(Pattern::Wildcard, self.span_from(start)))
            }
            Some(Token::VarSigil(name)) => {
                self.advance()?;
                Ok(Node::new(Pattern::Var(name), self.span_from(start)))
            }
            Some(Token::Integer(n)) => {
                self.advance()?;
                Ok(Node::new(Pattern::IntLit(n), self.span_from(start)))
            }
            Some(Token::Float(n)) => {
                self.advance()?;
                Ok(Node::new(Pattern::FloatLit(n), self.span_from(start)))
            }
            Some(Token::Str(s)) => {
                self.advance()?;
                Ok(Node::new(Pattern::StrLit(s), self.span_from(start)))
            }
            Some(Token::True) => {
                self.advance()?;
                Ok(Node::new(Pattern::BoolLit(true), self.span_from(start)))
            }
            Some(Token::False) => {
                self.advance()?;
                Ok(Node::new(Pattern::BoolLit(false), self.span_from(start)))
            }
            Some(Token::Nil) => {
                self.advance()?;
                Ok(Node::new(Pattern::NilLit, self.span_from(start)))
            }
            // @TypeName as a pattern (nullary constructor)
            Some(Token::TypeSigil(name)) => {
                self.advance()?;
                Ok(Node::new(Pattern::TypeName(name), self.span_from(start)))
            }
            // (@Constructor $a $b ...) — constructor pattern
            Some(Token::LParen) => {
                self.advance()?;
                let (name, _) = self.expect_type_sigil()?;
                let mut sub_patterns = Vec::new();
                while self.peek_token() != Some(&Token::RParen) {
                    sub_patterns.push(self.parse_pattern()?);
                }
                let end = self.expect_rparen()?;
                let span = Span::new(start.start, end.end);
                Ok(Node::new(Pattern::Constructor(name, sub_patterns), span))
            }
            None => Err(ParseError::eof("pattern")),
            _ => {
                let spanned = self.advance()?;
                Err(ParseError::unexpected(
                    &spanned.token,
                    spanned.span,
                    "pattern",
                ))
            }
        }
    }
}

/// Convenience function: parse source code into a program.
pub fn parse(source: &str) -> Result<Program> {
    let tokens = llml_lexer::tokenize(source)?;
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

/// Parse with error recovery: returns a partial program and accumulated errors.
///
/// Unlike `parse()`, this function never fails completely — it always returns
/// whatever declarations it managed to parse, along with any errors.
pub fn parse_recovering(
    source: &str,
) -> std::result::Result<(Program, Vec<ParseError>), ParseError> {
    let tokens = llml_lexer::tokenize(source)?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap_or(Program { decls: vec![] });
    let errors = parser.errors.clone();
    Ok((program, errors))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_integer_literal() {
        let prog = parse("42").unwrap();
        assert_eq!(prog.decls.len(), 1);
        match &prog.decls[0].inner {
            Decl::Expr(Expr::IntLit(42)) => {}
            other => panic!("expected IntLit(42), got {:?}", other),
        }
    }

    #[test]
    fn test_parse_let_binding() {
        let prog = parse("(let $x : @I32 42)").unwrap();
        assert_eq!(prog.decls.len(), 1);
        match &prog.decls[0].inner {
            Decl::Let(LetDecl {
                name, is_mut, ty, ..
            }) => {
                assert_eq!(name, "x");
                assert!(!is_mut);
                assert!(ty.is_some());
            }
            other => panic!("expected Let, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_simple_fn() {
        let src = "(fn $add (: @I32 @I32 -> @I32) ($a : @I32) ($b : @I32) (+ $a $b))";
        let prog = parse(src).unwrap();
        match &prog.decls[0].inner {
            Decl::Fn(FnDecl { name, params, .. }) => {
                assert_eq!(name, "add");
                assert_eq!(params.len(), 2);
                assert_eq!(params[0].name, "a");
                assert_eq!(params[1].name, "b");
            }
            other => panic!("expected Fn, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_if_expr() {
        let src = "(if true 1 0)";
        let prog = parse(src).unwrap();
        match &prog.decls[0].inner {
            Decl::Expr(Expr::If(_, _, _)) => {}
            other => panic!("expected If, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_do_block() {
        let src = "(do (let $x : @I32 1) (let $y : @I32 2) (+ $x $y))";
        let prog = parse(src).unwrap();
        match &prog.decls[0].inner {
            Decl::Expr(Expr::Do(exprs)) => {
                assert_eq!(exprs.len(), 3);
            }
            other => panic!("expected Do, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_fibonacci() {
        let src = r#"(fn $fib (: @I32 -> @I32)
  ($n : @I32)
  (if (<= $n 1) $n (+ ($fib (- $n 1)) ($fib (- $n 2)))))"#;
        let prog = parse(src).unwrap();
        match &prog.decls[0].inner {
            Decl::Fn(FnDecl { name, .. }) => {
                assert_eq!(name, "fib");
            }
            other => panic!("expected Fn, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_type_def_sum() {
        let src = r#"(ty @Option (: ^T)
  (sum
    (@Some $val : ^T)
    @None))"#;
        let prog = parse(src).unwrap();
        match &prog.decls[0].inner {
            Decl::TypeDef(TypeDef {
                name,
                params,
                body: TypeDefBody::Sum(variants),
            }) => {
                assert_eq!(name, "Option");
                assert_eq!(params, &["T"]);
                assert_eq!(variants.len(), 2);
                assert_eq!(variants[0].name, "Some");
                assert_eq!(variants[1].name, "None");
            }
            other => panic!("expected TypeDef(Sum), got {:?}", other),
        }
    }

    #[test]
    fn test_parse_type_def_prod() {
        let src = "(ty @Point (prod $x : @F64 $y : @F64))";
        let prog = parse(src).unwrap();
        match &prog.decls[0].inner {
            Decl::TypeDef(TypeDef {
                name,
                body: TypeDefBody::Prod(fields),
                ..
            }) => {
                assert_eq!(name, "Point");
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].name, "x");
                assert_eq!(fields[1].name, "y");
            }
            other => panic!("expected TypeDef(Prod), got {:?}", other),
        }
    }

    #[test]
    fn test_parse_match() {
        let src = r#"(mat $x
  ((@Some $v) $v)
  (@None 0))"#;
        let prog = parse(src).unwrap();
        match &prog.decls[0].inner {
            Decl::Expr(Expr::Match(_, arms)) => {
                assert_eq!(arms.len(), 2);
                match &arms[0].pattern.inner {
                    Pattern::Constructor(name, sub) => {
                        assert_eq!(name, "Some");
                        assert_eq!(sub.len(), 1);
                    }
                    other => panic!("expected Constructor pattern, got {:?}", other),
                }
                match &arms[1].pattern.inner {
                    Pattern::TypeName(name) => assert_eq!(name, "None"),
                    other => panic!("expected TypeName pattern, got {:?}", other),
                }
            }
            other => panic!("expected Match, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_function_call() {
        let src = "($add 1 2)";
        let prog = parse(src).unwrap();
        match &prog.decls[0].inner {
            Decl::Expr(Expr::Call(callee, args)) => {
                match &callee.inner {
                    Expr::Var(name) => assert_eq!(name, "add"),
                    other => panic!("expected Var, got {:?}", other),
                }
                assert_eq!(args.len(), 2);
            }
            other => panic!("expected Call, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_nested_expr() {
        let src = "(+ (* 2 3) (- 10 4))";
        let prog = parse(src).unwrap();
        match &prog.decls[0].inner {
            Decl::Expr(Expr::BinOp(BinOp::Add, _, _)) => {}
            other => panic!("expected BinOp(Add), got {:?}", other),
        }
    }

    #[test]
    fn test_parse_complete_program() {
        let src = r#"
(ty @Expr (sum
  (@Num $val : @F64)
  (@Add $l : @Expr $r : @Expr)
  (@Mul $l : @Expr $r : @Expr)))

(fn $eval (: @Expr -> @F64) ($e : @Expr)
  (mat $e
    ((@Num $v) $v)
    ((@Add $l $r) (+ ($eval $l) ($eval $r)))
    ((@Mul $l $r) (* ($eval $l) ($eval $r)))))
"#;
        let prog = parse(src).unwrap();
        assert_eq!(prog.decls.len(), 2);
    }

    #[test]
    fn test_error_recovery() {
        // First decl is valid, second has an error, third is valid
        let src = r#"
(let $x : @I32 42)
(let $y : @I32 )
(let $z : @I32 99)
"#;
        let (prog, errors) = parse_recovering(src).unwrap();
        // Should have recovered and parsed at least some declarations
        assert!(!errors.is_empty(), "expected at least one error");
        // Should have at least the valid declarations
        assert!(prog.decls.len() >= 1, "expected at least 1 recovered decl");
    }

    #[test]
    fn test_error_recovery_multiple_errors() {
        // Invalid: missing closing paren, then valid expression
        let src = r#"
42
(let $y : @I32
99
"#;
        let (prog, errors) = parse_recovering(src).unwrap();
        // 42 is valid, the let is malformed, 99 may or may not be recovered
        assert!(prog.decls.len() >= 1, "expected at least 1 recovered decl");
        assert!(!errors.is_empty(), "expected at least one error");
    }
}
