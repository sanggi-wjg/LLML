use llml_lexer::Span;

/// A node in the AST with source span information.
#[derive(Debug, Clone, PartialEq)]
pub struct Node<T> {
    pub inner: T,
    pub span: Span,
}

impl<T> Node<T> {
    pub fn new(inner: T, span: Span) -> Self {
        Self { inner, span }
    }
}

/// Top-level program: a sequence of declarations.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub decls: Vec<Node<Decl>>,
}

/// Top-level declaration.
#[derive(Debug, Clone, PartialEq)]
pub enum Decl {
    /// (fn $name (: ...) ($param : @Type) ... $body)
    Fn(FnDecl),
    /// (let $name : @Type $expr)
    Let(LetDecl),
    /// (ty @Name ...)
    TypeDef(TypeDef),
    /// (mod #name ...)
    Module(ModuleDecl),
    /// (pub ...)
    Pub(Box<Node<Decl>>),
    /// A bare expression at the top level
    Expr(Expr),
    /// Error node for parser recovery
    Error,
}

/// Function declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct FnDecl {
    pub name: String,
    pub type_sig: Option<TypeSig>,
    pub params: Vec<Param>,
    pub body: Box<Node<Expr>>,
}

/// Function type signature: (: T1 T2 ... -> R ! E1 E2)
#[derive(Debug, Clone, PartialEq)]
pub struct TypeSig {
    pub param_types: Vec<Node<TypeExpr>>,
    pub return_type: Box<Node<TypeExpr>>,
    pub effects: Vec<String>,
}

/// Parameter: ($name : @Type)
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Node<TypeExpr>,
    pub is_mut: bool,
    pub span: Span,
}

/// Let binding.
#[derive(Debug, Clone, PartialEq)]
pub struct LetDecl {
    pub name: String,
    pub is_mut: bool,
    pub ty: Option<Node<TypeExpr>>,
    pub value: Box<Node<Expr>>,
}

/// Type definition.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeDef {
    pub name: String,
    pub params: Vec<String>,
    pub body: TypeDefBody,
}

/// Type definition body.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeDefBody {
    /// (sum (@Variant1 ...) (@Variant2 ...))
    Sum(Vec<Variant>),
    /// (prod $field1 : @Type1 $field2 : @Type2)
    Prod(Vec<Field>),
    /// (lin) — linear marker
    Linear,
}

/// Sum type variant.
#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
    pub name: String,
    pub fields: Vec<Field>,
    pub span: Span,
}

/// A named, typed field.
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub ty: Node<TypeExpr>,
}

/// Module declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleDecl {
    pub name: String,
    pub decls: Vec<Node<Decl>>,
}

/// Type expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeExpr {
    /// A named type: @I32, @Str, @Vec, etc.
    Named(String),
    /// A generic type parameter: ^T
    Generic(String),
    /// A parameterized type: @List ^T → App(@List, [^T])
    App(String, Vec<Node<TypeExpr>>),
    /// A function type: (: @I32 @I32 -> @I32)
    FnType(Vec<Node<TypeExpr>>, Box<Node<TypeExpr>>),
    /// Linear wrapper: ~@Type
    Linear(Box<Node<TypeExpr>>),
    /// Reference: &@Type
    Ref(Box<Node<TypeExpr>>),
}

/// Expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // ── Literals ──────────────────────────────
    /// Integer literal
    IntLit(i64),
    /// Float literal
    FloatLit(f64),
    /// String literal
    StrLit(String),
    /// Boolean literal
    BoolLit(bool),
    /// Nil literal
    NilLit,

    // ── Identifiers ──────────────────────────
    /// Variable reference: $name
    Var(String),
    /// Type constructor: @Name (used as value, e.g. @None, @Some)
    TypeConstructor(String),

    // ── Core forms ───────────────────────────
    /// Let binding: (let $name : @Type $expr)
    Let(Box<LetDecl>, Box<Node<Expr>>),

    /// Function definition (as expression)
    FnExpr(FnDecl),

    /// Function call: ($f $arg1 $arg2 ...)
    Call(Box<Node<Expr>>, Vec<Node<Expr>>),

    /// If expression: (if $cond $then $else)
    If(Box<Node<Expr>>, Box<Node<Expr>>, Box<Node<Expr>>),

    /// Pattern match: (mat $expr ($pat $body) ...)
    Match(Box<Node<Expr>>, Vec<MatchArm>),

    /// Do block: (do $expr1 $expr2 ... $exprN)
    Do(Vec<Node<Expr>>),

    /// Binary operator: (+ $a $b), (* $a $b), etc.
    BinOp(BinOp, Box<Node<Expr>>, Box<Node<Expr>>),

    /// Unary operator
    UnaryOp(UnaryOp, Box<Node<Expr>>),

    /// Return: (ret $expr)
    Return(Box<Node<Expr>>),

    /// Set (mutation): (set $var $expr)
    SetExpr(String, Box<Node<Expr>>),

    /// Module-qualified access: #mod.$name
    ModAccess(String, String),

    /// Error node for parser recovery
    Error,
}

/// Match arm: ($pattern $body)
#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Node<Pattern>,
    pub body: Node<Expr>,
    pub span: Span,
}

/// Patterns for match expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// Wildcard: _
    Wildcard,
    /// Variable binding: $name
    Var(String),
    /// Integer literal
    IntLit(i64),
    /// Float literal
    FloatLit(f64),
    /// String literal
    StrLit(String),
    /// Boolean literal
    BoolLit(bool),
    /// Nil
    NilLit,
    /// Constructor pattern: (@Name $a $b ...)
    Constructor(String, Vec<Node<Pattern>>),
    /// Type name pattern (nullary constructor): @None
    TypeName(String),
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}
