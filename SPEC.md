# LLML Language Specification

**Version:** 0.1.0
**Status:** Draft

---

## 1. Overview

LLML (Language for Large Model Logic) is a programming language designed for LLMs to read, write, and reason about. It uses a modified s-expression syntax with a sigil system for lexical disambiguation.

### Design Goals

- **Token efficiency.** Short keywords, minimal punctuation, s-expression structure keeps token counts low.
- **Structural unambiguity.** Every compound form begins with `(` followed by a keyword or callable, so the parser never backtracks. Sigils (`@`, `$`, `#`, etc.) make the role of every identifier visible at the token level.
- **Pattern regularity.** All forms follow the same parenthesized structure. There are no infix operators at the syntax level -- binary operations are prefix calls `(+ $a $b)`.

---

## 2. Lexical Structure

### 2.1 Whitespace and Comments

Whitespace (spaces, tabs, carriage returns, newlines) separates tokens and is otherwise ignored. Comments begin with `;;` and extend to the end of the line.

```llml
;; This is a comment
(let $x : @I32 42) ;; inline comment
```

### 2.2 Sigil System

Every identifier is prefixed with a sigil that declares its syntactic role.

| Sigil | Role | Regex | Examples |
|-------|------|-------|----------|
| `@` | Type name | `@[A-Z][a-zA-Z0-9_]*` | `@I32`, `@Str`, `@Expr` |
| `$` | Variable / function name | `\$[a-z_][a-zA-Z0-9_]*` | `$x`, `$fib`, `$my_var` |
| `#` | Module name | `#[a-z_][a-zA-Z0-9_]*` | `#std`, `#math` |
| `!` | Effect annotation | `![a-z_][a-zA-Z0-9_]*` | `!io`, `!err` |
| `&` | Reference type modifier | `&` (bare token) | `&@I32` |
| `~` | Linear type modifier | `~` (bare token) | `~@File` |
| `^` | Generic type parameter | `\^[A-Z][a-zA-Z0-9_]*` | `^T`, `^Element` |

### 2.3 Keywords

All keywords are lowercase, unsigilated, and reserved.

```
fn    let   if    mat   do    ty    mod   ef
use   pub   mut   sum   prod  ret   true  false
nil   for   in    lazy  region set  lin
```

### 2.4 Operators

Operators appear only in prefix position inside parenthesized forms.

| Token | Meaning |
|-------|---------|
| `+` `-` `*` `/` `%` | Arithmetic |
| `=` `!=` `<` `>` `<=` `>=` | Comparison |
| `&&` `\|\|` | Logical AND / OR |
| `!` | Logical NOT (bare, as unary) |
| `->` | Return type separator in type signatures |
| `:` | Type annotation separator |
| `.` | Module member access |

### 2.5 Delimiters

The only delimiters are `(` and `)`.

### 2.6 Literals

**Integers.** A sequence of decimal digits: `0`, `42`, `1000`.
Lexed as `i64`.

**Floats.** Digits, a dot, then more digits: `3.14`, `0.5`, `100.0`.
Lexed as `f64`.

**Strings.** Double-quoted, with backslash escape sequences (`\\`, `\"`, `\n`, `\t`):

```llml
"hello, world"
"line one\nline two"
"she said \"hi\""
```

**Booleans.** The keywords `true` and `false`.

**Nil.** The keyword `nil`.

---

## 3. Grammar (EBNF)

The following grammar uses these conventions: `{ X }` means zero or more repetitions of X, `[ X ]` means X is optional, `|` separates alternatives, and quoted strings are terminal tokens.

```ebnf
program     = { decl } ;

(* ── Declarations ────────────────────────────── *)

decl        = paren_decl | expr ;
paren_decl  = "(" decl_inner ")" ;
decl_inner  = fn_decl | let_decl | type_def | mod_decl | pub_decl | expr_inner ;

fn_decl     = "fn" VAR [ type_sig ] { param } expr ;
let_decl    = "let" [ "mut" ] VAR ":" type_expr expr ;
type_def    = "ty" TYPE [ "(" ":" { GENERIC } ")" ] type_body ;
mod_decl    = "mod" MODULE { decl } ;
pub_decl    = "pub" decl ;

(* ── Type Signatures ─────────────────────────── *)

type_sig    = "(" ":" { type_expr } "->" type_expr { EFFECT } ")" ;

param       = "(" [ "mut" ] VAR ":" type_expr ")" ;

type_body   = sum_body | prod_body | "lin" ;
sum_body    = "(" "sum" { variant } ")" ;
prod_body   = "(" "prod" { field } ")" ;
variant     = "(" TYPE { field } ")" ;
field       = VAR ":" type_expr ;

(* ── Type Expressions ────────────────────────── *)

type_expr   = TYPE { type_arg }
            | GENERIC
            | "~" type_expr
            | "&" type_expr
            | fn_type ;
type_arg    = TYPE | GENERIC ;
fn_type     = "(" ":" { type_expr } [ "->" type_expr ] ")" ;

(* ── Expressions ─────────────────────────────── *)

expr        = literal | VAR | TYPE | paren_expr ;
paren_expr  = "(" expr_inner ")" ;

expr_inner  = if_expr
            | mat_expr
            | do_expr
            | let_expr
            | fn_expr
            | ret_expr
            | set_expr
            | binop_expr
            | call_expr ;

if_expr     = "if" expr expr expr ;
mat_expr    = "mat" expr { match_arm } ;
do_expr     = "do" { expr } ;
let_expr    = "let" [ "mut" ] VAR ":" type_expr expr ;
fn_expr     = "fn" VAR [ type_sig ] { param } expr ;
ret_expr    = "ret" expr ;
set_expr    = "set" VAR expr ;
binop_expr  = BINOP expr expr ;
call_expr   = expr { expr } ;

match_arm   = "(" pattern expr ")" ;

(* ── Patterns ────────────────────────────────── *)

pattern     = "_"
            | VAR
            | INTEGER
            | FLOAT
            | STRING
            | BOOL
            | "nil"
            | TYPE
            | "(" TYPE { pattern } ")" ;

(* ── Literals ────────────────────────────────── *)

literal     = INTEGER | FLOAT | STRING | BOOL | "nil" ;

(* ── Tokens ──────────────────────────────────── *)

VAR         = "$" [a-z_][a-zA-Z0-9_]* ;
TYPE        = "@" [A-Z][a-zA-Z0-9_]* ;
MODULE      = "#" [a-z_][a-zA-Z0-9_]* ;
EFFECT      = "!" [a-z_][a-zA-Z0-9_]* ;
GENERIC     = "^" [A-Z][a-zA-Z0-9_]* ;
BINOP       = "+" | "-" | "*" | "/" | "%" | "=" | "!="
            | "<" | ">" | "<=" | ">=" | "&&" | "||" ;
INTEGER     = [0-9]+ ;
FLOAT       = [0-9]+ "." [0-9]+ ;
STRING      = '"' { any_char | escape } '"' ;
BOOL        = "true" | "false" ;
```

### 3.1 Disambiguation Rules

1. When `(` is followed by a keyword (`fn`, `let`, `if`, `mat`, `do`, `ty`, `mod`, `pub`, `ret`, `set`), it is parsed as that keyword's form.
2. When `(` is followed by an operator token, it is parsed as a binary (or unary) operation.
3. When `(` is followed by a type sigil `@Name`, it is parsed as a type constructor call.
4. Otherwise, `(` starts a function call: the first sub-expression is the callee.
5. Parameters `($name : @Type)` are distinguished from calls by 3-token lookahead: `( $var :`.

---

## 4. Type System

### 4.1 Primitive Types

| Type | Description |
|------|-------------|
| `@I8` `@I16` `@I32` `@I64` | Signed integers (8/16/32/64-bit) |
| `@U8` `@U16` `@U32` `@U64` | Unsigned integers |
| `@F32` `@F64` | IEEE 754 floating point |
| `@Bool` | Boolean (`true` / `false`) |
| `@Str` | UTF-8 string |
| `@Nil` | Unit / void type (single value: `nil`) |
| `@Byte` | Raw byte |

### 4.2 Algebraic Data Types

#### Sum Types

A sum type defines a tagged union of variants, each carrying zero or more fields.

```llml
(ty @Option ^T (sum
  (@None)
  (@Some $val : ^T)))

(ty @Expr (sum
  (@Num $val : @F64)
  (@Add $l : @Expr $r : @Expr)
  (@Neg $e : @Expr)))
```

Constructors are used as functions:

```llml
(@Some 42)        ;; => (@Some 42)
(@Num 3.14)       ;; => (@Num 3.14)
@None             ;; => @None  (nullary constructor)
```

#### Product Types

A product type is a record with named fields.

```llml
(ty @Point (prod
  $x : @F64
  $y : @F64))
```

### 4.3 Generic Types

Type parameters are written with the `^` sigil and appear after the type name in definitions.

```llml
(ty @Pair ^A ^B (prod
  $fst : ^A
  $snd : ^B))
```

### 4.4 Function Types

Function types are written using the `(:` ... `->` ... `)` syntax.

```llml
(: @I32 -> @I32)              ;; function from I32 to I32
(: @I32 @I32 -> @Bool)        ;; two I32 params, returns Bool
(: (: @I32 -> @I32) @I32 -> @I32)  ;; higher-order: takes a function and I32
```

### 4.5 Linear Types

The `~` modifier marks a type as linear. A linear value must be used exactly once -- it cannot be duplicated or silently dropped.

```llml
(ty @File (lin))
;; ~@File means: a linear File value

(fn $open (: @Str -> ~@File !io) ($path : @Str)
  ...)
```

### 4.6 Reference Types

The `&` modifier creates a borrowed reference to a value.

```llml
(fn $length (: &@Str -> @I32) ($s : &@Str)
  ...)
```

### 4.7 Effect Annotations

Side effects are declared in type signatures after the return type using effect sigils.

```llml
(fn $read_file (: @Str -> @Str !io !err)
  ($path : @Str)
  ...)
```

Effects document and constrain what a function may do. Effect names are user-defined (`!io`, `!err`, `!mut`, etc.).

---

## 5. Expressions

LLML is expression-oriented: every form produces a value.

### 5.1 Literals

```llml
42          ;; integer
3.14        ;; float
"hello"     ;; string
true        ;; boolean
nil         ;; nil value
```

### 5.2 Variables

Variables are referenced by their sigiled name. Lookup follows lexical scoping.

```llml
$x          ;; variable reference
$my_func    ;; function reference
```

### 5.3 Let Bindings

Bind a value to a name. At top level, the binding persists for the rest of the program. Inside a `do` block, it is scoped.

```llml
(let $x : @I32 42)
(let mut $counter : @I32 0)   ;; mutable binding
```

The type annotation is required. The bound name is available in subsequent expressions.

### 5.4 If Expressions

Three-part conditional. Both branches must be present. The result is the value of the taken branch.

```llml
(if (> $x 0) "positive" "non-positive")
(if (= $n 0) 1 (* $n ($fact (- $n 1))))
```

### 5.5 Match Expressions

Pattern matching on a scrutinee value. Arms are tried top to bottom.

```llml
(mat $expr
  ((@Num $v) $v)
  ((@Add $l $r) (+ ($eval $l) ($eval $r)))
  ((@Neg $e) (- ($eval $e))))
```

See Section 7 for pattern syntax. Match should be exhaustive.

### 5.6 Do Blocks

A sequence of expressions evaluated in order. The value of the block is the value of the last expression.

```llml
(do
  (let $x : @I32 10)
  (let $y : @I32 20)
  ($print ($to_str (+ $x $y)))
  (+ $x $y))
```

### 5.7 Function Definitions

Functions are defined with `fn`. A function is also a value (first-class).

```llml
;; Named function
(fn $add (: @I32 @I32 -> @I32) ($a : @I32) ($b : @I32)
  (+ $a $b))

;; Higher-order function
(fn $twice (: (: @I32 -> @I32) @I32 -> @I32)
  ($f : (: @I32 -> @I32)) ($x : @I32)
  ($f ($f $x)))
```

### 5.8 Function Calls

A parenthesized form where the first element is a callable expression.

```llml
($add 1 2)              ;; call named function
($f $x)                 ;; call variable holding a function
($print "hello")        ;; call built-in
(@Num 3.14)             ;; type constructor call
```

### 5.9 Binary and Unary Operators

Operators use prefix syntax. Binary operators take exactly two operands.

```llml
;; Arithmetic
(+ $a $b)     ;; addition
(- $a $b)     ;; subtraction
(* $a $b)     ;; multiplication
(/ $a $b)     ;; division (integer or float)
(% $a $b)     ;; modulo

;; Comparison (returns @Bool)
(= $a $b)     ;; equality
(!= $a $b)    ;; inequality
(< $a $b)     ;; less than
(> $a $b)     ;; greater than
(<= $a $b)    ;; less or equal
(>= $a $b)    ;; greater or equal

;; Logical (operate on @Bool)
(&& $a $b)    ;; logical AND
(|| $a $b)    ;; logical OR
```

The `+` operator on strings performs concatenation:

```llml
(+ "Hello, " (+ $name "!"))  ;; => "Hello, LLML!"
```

Unary minus is expressed as subtraction from a single operand or via binary form:

```llml
(- $x)        ;; negate (unary, when single operand)
```

### 5.10 Return

Early return from a function.

```llml
(fn $check (: @I32 -> @Str) ($n : @I32)
  (if (< $n 0) (ret "negative") "non-negative"))
```

### 5.11 Set (Mutation)

Mutate a variable previously declared with `mut`.

```llml
(let mut $x : @I32 0)
(set $x 10)
```

### 5.12 Module Access

Access a member of a module using `#module.$member` syntax.

```llml
#math.$pi
#io.$read_line
```

### 5.13 Type Constructors

Type names used as expressions construct values of sum types.

```llml
@None                        ;; nullary constructor
(@Some 42)                   ;; unary constructor
(@Add (@Num 1.0) (@Num 2.0)) ;; nested constructors
```

---

## 6. Declarations

### 6.1 Function Declaration

```llml
(fn $name (: @ParamType1 @ParamType2 -> @ReturnType)
  ($param1 : @ParamType1) ($param2 : @ParamType2)
  body_expr)
```

- The type signature `(: ... -> ...)` is optional.
- Parameters are written as `($name : @Type)`.
- Mutable parameters: `(mut $name : @Type)`.
- The body is a single expression (use `do` for sequencing).

### 6.2 Type Definitions

```llml
;; Sum type (tagged union)
(ty @Name (sum
  (@Variant1 $field : @Type)
  (@Variant2)))

;; Product type (record)
(ty @Name (prod
  $field1 : @Type1
  $field2 : @Type2))

;; Linear type marker
(ty @Handle (lin))

;; Generic type
(ty @List ^T (sum
  (@Nil)
  (@Cons $head : ^T $tail : @List)))
```

### 6.3 Module Declarations

```llml
(mod #math
  (fn $square (: @I32 -> @I32) ($n : @I32) (* $n $n))
  (let $pi : @F64 3.14159265))
```

### 6.4 Pub Modifier

Marks a declaration as publicly visible outside its module.

```llml
(mod #utils
  (pub (fn $helper (: @I32 -> @I32) ($x : @I32) (+ $x 1)))
  (fn $internal (: @I32 -> @I32) ($x : @I32) (* $x 2)))
```

---

## 7. Pattern Matching

The `mat` form matches a scrutinee against a list of arms. Each arm is `(pattern body)`.

### 7.1 Pattern Forms

| Pattern | Syntax | Matches |
|---------|--------|---------|
| Wildcard | `_` | Anything (value discarded) |
| Variable | `$name` | Anything (value bound to `$name`) |
| Integer literal | `42` | Equal integer |
| Float literal | `3.14` | Equal float |
| String literal | `"hello"` | Equal string |
| Boolean literal | `true` / `false` | Equal boolean |
| Nil literal | `nil` | Nil value |
| Type name | `@None` | Nullary constructor |
| Constructor | `(@Name $a $b)` | Constructor with sub-patterns |

### 7.2 Examples

```llml
;; Matching on an integer
(mat $n
  (0 "zero")
  (1 "one")
  ($other ($to_str $other)))

;; Matching on a sum type
(mat $opt
  (@None "nothing")
  ((@Some $v) ($to_str $v)))

;; Nested constructor patterns
(mat $expr
  ((@Num $v) $v)
  ((@Add $l $r) (+ ($eval $l) ($eval $r)))
  ((@Mul $l $r) (* ($eval $l) ($eval $r)))
  ((@Neg $inner) (- ($eval $inner))))
```

### 7.3 Exhaustiveness

Match expressions should be exhaustive -- every possible value of the scrutinee's type should be covered by at least one arm. A wildcard `_` or variable `$name` arm can serve as a catch-all. A non-exhaustive match produces a runtime error if no arm matches.

---

## 8. Semantics

### 8.1 Evaluation Strategy

**Strict evaluation.** All arguments are evaluated before being passed to a function. The `lazy` keyword is reserved for future lazy evaluation support.

### 8.2 Expression-Oriented

Every syntactic form is an expression and produces a value.

- `if` returns the value of the taken branch.
- `do` returns the value of its last sub-expression (or `nil` if empty).
- `let` at top level returns the bound value.
- `mat` returns the value of the matched arm's body.
- `fn` declarations return `nil` as a top-level side effect; the function itself is bound in the environment.

### 8.3 Immutability by Default

Bindings are immutable unless declared with `mut`. Attempting to `set` an immutable binding is an error.

```llml
(let $x : @I32 5)
(set $x 10)            ;; ERROR: $x is not mutable

(let mut $y : @I32 5)
(set $y 10)            ;; OK
```

### 8.4 No Null

There is no null pointer or null reference. The `nil` keyword is the sole value of type `@Nil` and represents an explicit "no value." Absence is modeled with sum types:

```llml
(ty @Option ^T (sum (@None) (@Some $val : ^T)))
```

### 8.5 No Implicit Conversions

There are no implicit type coercions. Integer-to-float, number-to-string, and similar conversions must be explicit (via built-in functions like `$to_str`).

### 8.6 Lexical Scoping

Variable lookup follows lexical scope. Closures capture variables from their defining environment.

```llml
(fn $make_adder (: @I32 -> (: @I32 -> @I32)) ($n : @I32)
  (fn $adder (: @I32 -> @I32) ($x : @I32) (+ $x $n)))

(let $add5 : (: @I32 -> @I32) ($make_adder 5))
($add5 10)  ;; => 15
```

### 8.7 Recursion

Functions may reference themselves by name in their body, enabling direct recursion.

```llml
(fn $fib (: @I32 -> @I32)
  ($n : @I32)
  (if (<= $n 1) $n (+ ($fib (- $n 1)) ($fib (- $n 2)))))
```

### 8.8 Operator Semantics

- Arithmetic operators (`+`, `-`, `*`, `/`, `%`) work on `@I32`/`@I64` and `@F32`/`@F64`.
- `+` also concatenates `@Str` values.
- `/` on integers performs integer division. Division by zero is a runtime error.
- Comparison operators return `@Bool`.
- `&&` and `||` are strict (both operands evaluated).

---

## 9. Built-in Functions

The following functions are available without import.

| Function | Type Signature | Description |
|----------|----------------|-------------|
| `$print` | `(: @Str -> @Nil !io)` | Print a string to stdout with a newline |
| `$to_str` | `(: ^T -> @Str)` | Convert any value to its string representation |
| `$str_concat` | `(: @Str @Str -> @Str)` | Concatenate two strings |
| `$len` | `(: @Str -> @I32)` | Return the length of a string |
| `$not` | `(: @Bool -> @Bool)` | Logical negation |
| `$abs` | `(: @I32 -> @I32)` | Absolute value of an integer |

### Examples

```llml
($print "Hello, LLML!")
($print ($to_str 42))
($print ($str_concat "one" "two"))
($print ($to_str ($len "four")))
($print ($to_str ($not true)))
($print ($to_str ($abs -7)))
```

---

## 10. File Extension

LLML source files use the `.llml` extension.

```
my_program.llml
```

---

## Appendix A: Complete Example

```llml
;; Expression tree evaluator

;; Define the expression ADT
(ty @Expr (sum
  (@Num $val : @F64)
  (@Add $l : @Expr $r : @Expr)
  (@Mul $l : @Expr $r : @Expr)
  (@Neg $e : @Expr)))

;; Recursive evaluator
(fn $eval (: @Expr -> @F64) ($e : @Expr)
  (mat $e
    ((@Num $v) $v)
    ((@Add $l $r) (+ ($eval $l) ($eval $r)))
    ((@Mul $l $r) (* ($eval $l) ($eval $r)))
    ((@Neg $inner) (- ($eval $inner)))))

;; Build and evaluate: (1.0 + 2.0) * 3.0
(let $expr : @Expr
  (@Mul
    (@Add (@Num 1.0) (@Num 2.0))
    (@Num 3.0)))

($print (+ "Result: " ($to_str ($eval $expr))))
;; Output: Result: 9
```

## Appendix B: Reserved Keywords

The following identifiers are reserved and cannot be used as variable, type, or module names: `fn`, `let`, `if`, `mat`, `do`, `ty`, `mod`, `ef`, `use`, `pub`, `mut`, `sum`, `prod`, `ret`, `true`, `false`, `nil`, `for`, `in`, `lazy`, `region`, `set`, `lin`.
