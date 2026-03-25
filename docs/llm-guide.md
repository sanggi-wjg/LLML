# LLML Language Guide for LLMs

This document is a complete reference for generating valid LLML code. LLML (Language for Large Model Logic) is a programming language with modified s-expression syntax designed for unambiguous, reliable code generation by large language models.

---

## Table of Contents

1. [What is LLML](#what-is-llml)
2. [Quick Start](#quick-start)
3. [Syntax Overview](#syntax-overview)
4. [Sigil System](#sigil-system)
5. [Literals](#literals)
6. [Variables](#variables)
7. [Let Bindings](#let-bindings)
8. [Functions](#functions)
9. [If Expressions](#if-expressions)
10. [Pattern Matching](#pattern-matching)
11. [Do Blocks](#do-blocks)
12. [Type Definitions (ADTs)](#type-definitions-adts)
13. [Constructors](#constructors)
14. [Function Calls](#function-calls)
15. [Operators](#operators)
16. [Type System](#type-system)
17. [Built-in Functions](#built-in-functions)
18. [Comments](#comments)
19. [Common Patterns](#common-patterns)
20. [Error Messages](#error-messages)
21. [Anti-patterns](#anti-patterns)

---

## What is LLML

LLML is a statically-structured, expression-oriented programming language that uses modified s-expressions. Every compound form is wrapped in parentheses and begins with a keyword or callable expression. Identifiers are prefixed with sigils (`$` for variables, `@` for types) so there is never ambiguity between names, types, and keywords. This makes it straightforward to generate syntactically correct code.

Key properties:
- **Everything is an expression** -- every construct produces a value.
- **No infix operators** -- all operations use prefix notation: `(+ 1 2)`, not `1 + 2`.
- **Sigil-based naming** -- variables are `$x`, types are `@I32`, generics are `^T`.
- **No semicolons, no commas** -- whitespace separates tokens, parentheses delimit forms.
- **File extension**: `.llml`

---

## Quick Start

### Installation

```bash
# Clone the repository and build from source
git clone <repo-url>
cd LLML
cargo build --workspace --release

# The binary is at target/release/llml
# Or install locally:
cargo install --path crates/llml-cli
```

### Running a program

```bash
llml run program.llml
```

### Other CLI commands

```bash
llml parse program.llml   # Print the AST
llml lex program.llml     # Print the token stream
```

### Minimal example

```llml
;; hello.llml
(fn $greet (: @Str -> @Nil) ($name : @Str)
  ($print (+ "Hello, " (+ $name "!"))))

($greet "World")
```

```
$ llml run hello.llml
Hello, World!
```

---

## Syntax Overview

Every compound form in LLML is an s-expression: `(keyword ...)` or `(callable arg ...)`.

```
(keyword-or-expr  arg1  arg2  ...  argN)
```

Programs are a sequence of top-level declarations and expressions. The final value of the program is the value of the last top-level expression (printed unless it is `nil`).

---

## Sigil System

Sigils are mandatory prefixes that distinguish the role of every identifier. They eliminate the ambiguity between variable names, type names, modules, and keywords.

| Sigil | Meaning | Naming convention | Examples |
|-------|---------|-------------------|----------|
| `$` | Variable or function name | `lowercase_snake` | `$x`, `$my_func`, `$count` |
| `@` | Type name or constructor | `PascalCase` | `@I32`, `@Str`, `@Option`, `@Some`, `@None` |
| `#` | Module name | `lowercase_snake` | `#std`, `#math` |
| `!` | Effect name | `lowercase_snake` | `!io`, `!async` |
| `^` | Generic type parameter | `PascalCase` | `^T`, `^A`, `^Item` |
| `&` | Reference (planned) | prefix on type | `&@I32` |
| `~` | Linear/owned (planned) | prefix on type | `~@File` |

**Rules:**
- Variable sigil `$` must be followed by a lowercase letter or underscore: `$x`, `$my_var`.
- Type sigil `@` must be followed by an uppercase letter: `@I32`, `@MyType`.
- Generic sigil `^` must be followed by an uppercase letter: `^T`, `^Element`.
- Keywords (`fn`, `let`, `if`, `mat`, `do`, `ty`, etc.) have **no** sigil.

---

## Literals

LLML has five literal types.

| Literal | Syntax | Type | Examples |
|---------|--------|------|----------|
| Integer | Digits | `@I32` | `0`, `42`, `1000` |
| Float | Digits with decimal point | `@F64` | `3.14`, `0.5`, `100.0` |
| String | Double-quoted | `@Str` | `"hello"`, `""`, `"line\n"` |
| Boolean | `true` / `false` | `@Bool` | `true`, `false` |
| Nil | `nil` | `@Nil` | `nil` |

```llml
42          ;; integer
3.14        ;; float
"hello"     ;; string
true        ;; boolean
nil         ;; nil (unit value)
```

---

## Variables

Variables are referenced with the `$` sigil. A variable must be defined (via `let`, `fn` parameter, or pattern binding) before use.

```llml
$x
$my_variable
$count
```

---

## Let Bindings

Bind a value to a variable name.

### Syntax

```
(let $name : @Type value)
```

### Examples

```llml
;; Bind an integer
(let $x : @I32 42)

;; Bind a string
(let $name : @Str "Alice")

;; Bind a computed value
(let $sum : @I32 (+ 10 20))
```

**Important:** At the top level, a `let` binding defines the variable for subsequent top-level forms. Inside a `do` block, `let` bindings are sequenced and scoped to the block.

---

## Functions

### Syntax

```
(fn $name (: @ParamType1 @ParamType2 ... -> @ReturnType)
  ($param1 : @ParamType1) ($param2 : @ParamType2) ...
  body-expr)
```

The `(: ... -> ...)` form is the type signature. Parameters follow as `($name : @Type)` pairs. The last expression is the function body (its return value).

### Examples

```llml
;; A function that adds two integers
(fn $add (: @I32 @I32 -> @I32)
  ($a : @I32) ($b : @I32)
  (+ $a $b))

;; A function with one parameter
(fn $double (: @I32 -> @I32)
  ($n : @I32)
  (* $n 2))

;; A recursive function
(fn $factorial (: @I32 -> @I32)
  ($n : @I32)
  (if (= $n 0) 1 (* $n ($factorial (- $n 1)))))
```

### Function type as a parameter type

When a function takes another function as an argument, its type is written with `(: ... -> ...)`:

```llml
(fn $apply (: (: @I32 -> @I32) @I32 -> @I32)
  ($f : (: @I32 -> @I32)) ($x : @I32)
  ($f $x))
```

---

## If Expressions

### Syntax

```
(if condition then-expr else-expr)
```

Both branches are required. The condition must evaluate to `@Bool`.

### Examples

```llml
;; Simple conditional
(if true "yes" "no")

;; Conditional with comparison
(if (< $x 0) "negative" "non-negative")

;; Nested if
(if (= $n 0) "zero"
  (if (> $n 0) "positive" "negative"))
```

---

## Pattern Matching

### Syntax

```
(mat scrutinee
  (pattern1 body1)
  (pattern2 body2)
  ...)
```

The keyword is `mat` (not `match`). Each arm is `(pattern body)`. The first matching arm is evaluated.

### Pattern kinds

| Pattern | Matches | Example |
|---------|---------|---------|
| `$name` | Anything (binds to `$name`) | `$x` |
| Integer literal | Equal integer | `0`, `42` |
| Float literal | Equal float | `3.14` |
| String literal | Equal string | `"hello"` |
| `true` / `false` | Equal boolean | `true` |
| `nil` | Nil value | `nil` |
| `@Name` | Nullary constructor | `@None` |
| `(@Name $a $b ...)` | Constructor with fields | `(@Some $v)` |

### Examples

```llml
;; Match on integers
(mat $n
  (0 "zero")
  (1 "one")
  ($other "many"))

;; Match on constructors (Option pattern)
(mat $opt
  ((@Some $v) $v)
  (@None 0))

;; Match on a multi-field constructor
(mat $shape
  ((@Circle $r) (* 3.14 (* $r $r)))
  ((@Rectangle $w $h) (* $w $h)))
```

**Note:** The nullary constructor `@None` in a pattern is written without extra parentheses. A constructor with fields is written as `(@Name $field1 $field2 ...)`.

---

## Do Blocks

A `do` block sequences multiple expressions. Each expression is evaluated in order. `let` bindings inside a `do` block are visible to subsequent expressions. The value of the block is the value of the last expression.

### Syntax

```
(do expr1 expr2 ... exprN)
```

### Examples

```llml
;; Sequential computation
(do
  (let $a : @I32 10)
  (let $b : @I32 20)
  (+ $a $b))
;; => 30

;; Multiple steps with side effects
(do
  (let $x : @I32 5)
  ($print ($to_str $x))
  (let $y : @I32 (* $x $x))
  ($print ($to_str $y))
  $y)

;; Nesting do blocks
(do
  (let $base : @I32 100)
  (let $result : @I32
    (do
      (let $offset : @I32 42)
      (+ $base $offset)))
  $result)
;; => 142
```

---

## Type Definitions (ADTs)

Define algebraic data types with `ty`. Two forms exist: sum types and product types.

### Sum types (tagged unions / enums)

```
(ty @Name (sum
  (@Variant1 @FieldType1 @FieldType2 ...)
  (@Variant2 @FieldType ...)
  (@NullaryVariant)))
```

Variants with named fields use: `(@Variant $field1 : @Type1 $field2 : @Type2)`.
Variants with positional (unnamed) fields use: `(@Variant @Type1 @Type2)`.

### Product types (structs)

```
(ty @Name (prod ($field1 @Type1) ($field2 @Type2)))
```

### Examples

```llml
;; Option type (sum with named fields)
(ty @Option (sum
  (@Some $val : @I32)
  (@None)))

;; Shape type (sum with positional fields)
(ty @Shape (sum
  (@Circle @F64)
  (@Rectangle @F64 @F64)))

;; Expression tree (recursive sum type)
(ty @Expr (sum
  (@Num $val : @F64)
  (@Add $l : @Expr $r : @Expr)
  (@Mul $l : @Expr $r : @Expr)
  (@Neg $e : @Expr)))

;; Point (product type / struct)
(ty @Point (prod ($x @I32) ($y @I32)))
```

---

## Constructors

Constructors use the `@` sigil. A nullary constructor is just `@Name`. A constructor with arguments is `(@Name arg1 arg2 ...)`.

### Examples

```llml
;; Nullary constructor
@None

;; Unary constructor
(@Some 42)

;; Multi-field constructor
(@Rectangle 3.0 4.0)

;; Nested constructors
(@Add (@Num 1.0) (@Num 2.0))

;; Constructing a product type
(@Point 10 20)
```

---

## Function Calls

Call a function by wrapping the function name and arguments in parentheses.

### Syntax

```
($function-name arg1 arg2 ...)
```

### Examples

```llml
;; Call a user-defined function
($double 21)

;; Call with multiple arguments
($add 3 4)

;; Nested calls
($fib (- $n 1))

;; Call a built-in
($print "hello")

;; Call with computed arguments
($print ($to_str (+ 1 2)))
```

---

## Operators

All operators use prefix notation: `(op left right)`.

### Arithmetic operators

| Operator | Meaning | Types | Example |
|----------|---------|-------|---------|
| `+` | Addition | `@I32`, `@F64`, `@Str` (concat) | `(+ 1 2)` => `3` |
| `-` | Subtraction / negation | `@I32`, `@F64` | `(- 10 4)` => `6`, `(- 5)` => `-5` |
| `*` | Multiplication | `@I32`, `@F64` | `(* 3 7)` => `21` |
| `/` | Division | `@I32`, `@F64` | `(/ 15 3)` => `5` |
| `%` | Modulo | `@I32` | `(% 10 3)` => `1` |

### Comparison operators

| Operator | Meaning | Example |
|----------|---------|---------|
| `=` | Equal | `(= 5 5)` => `true` |
| `!=` | Not equal | `(!= 5 3)` => `true` |
| `<` | Less than | `(< 1 2)` => `true` |
| `>` | Greater than | `(> 2 1)` => `true` |
| `<=` | Less or equal | `(<= 5 5)` => `true` |
| `>=` | Greater or equal | `(>= 3 5)` => `false` |

### Logical operators

| Operator | Meaning | Example |
|----------|---------|---------|
| `&&` | Logical AND | `(&& true false)` => `false` |
| `\|\|` | Logical OR | `(\|\| false true)` => `true` |
| `!` | Logical NOT (unary) | `(! true)` => `false` |

### String concatenation

The `+` operator works on strings:

```llml
(+ "hello " "world")
;; => "hello world"
```

---

## Type System

### Primitive types

| Type | Description | Literals |
|------|-------------|----------|
| `@I32` | 64-bit signed integer (runtime) | `42`, `0`, `-5` |
| `@F64` | 64-bit floating point | `3.14`, `0.0` |
| `@Str` | String | `"hello"` |
| `@Bool` | Boolean | `true`, `false` |
| `@Nil` | Unit/void | `nil` |

### Algebraic data types

Sum types (tagged unions) and product types (structs) are defined with `ty`. See [Type Definitions](#type-definitions-adts).

### Function types

Function types are written as `(: @ParamType1 @ParamType2 -> @ReturnType)`:

```llml
(: @I32 -> @I32)               ;; function from I32 to I32
(: @I32 @I32 -> @Bool)          ;; function from two I32s to Bool
(: (: @I32 -> @I32) @I32 -> @I32)  ;; higher-order: takes a function and an I32
```

### Generic types (planned)

Generic type parameters use the `^` sigil: `^T`, `^A`. These are parsed but not yet fully enforced at runtime.

```llml
;; Planned syntax
(ty @List ^T (sum
  (@Cons $head : ^T $tail : (@List ^T))
  (@Empty)))
```

### Linear and reference types (planned)

- `~@Type` marks a linear (must-use) type.
- `&@Type` marks a reference (borrow) type.

These are recognized by the lexer and parser but not enforced in the current interpreter.

---

## Built-in Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `$print` | `@Str -> @Nil` | Print a string to output |
| `$to_str` | `^T -> @Str` | Convert any value to its string representation |
| `$str_concat` | `@Str @Str ... -> @Str` | Concatenate multiple values into a string |
| `$len` | `@Str -> @I32` | Return the length of a string |
| `$not` | `@Bool -> @Bool` | Logical negation |
| `$abs` | `@I32 / @F64 -> @I32 / @F64` | Absolute value |

### Examples

```llml
($print "hello world")
($print ($to_str 42))
($print ($str_concat "x = " ($to_str $x)))
($print ($to_str ($len "hello")))    ;; prints 5
($print ($to_str ($abs (- 42))))     ;; prints 42
```

---

## Comments

Line comments start with `;;` and extend to the end of the line.

```llml
;; This is a comment
(let $x : @I32 42)  ;; inline comment
```

There are no block comments.

---

## Common Patterns

### Recursive functions

```llml
;; Fibonacci
(fn $fib (: @I32 -> @I32)
  ($n : @I32)
  (if (<= $n 1) $n (+ ($fib (- $n 1)) ($fib (- $n 2)))))

($fib 10)  ;; => 55
```

```llml
;; Factorial
(fn $fact (: @I32 -> @I32)
  ($n : @I32)
  (if (= $n 0) 1 (* $n ($fact (- $n 1)))))

($fact 5)  ;; => 120
```

### Higher-order functions

```llml
;; Apply a function twice
(fn $twice (: (: @I32 -> @I32) @I32 -> @I32)
  ($f : (: @I32 -> @I32)) ($x : @I32)
  ($f ($f $x)))

(fn $inc (: @I32 -> @I32) ($n : @I32) (+ $n 1))

($twice $inc 5)  ;; => 7
```

```llml
;; Generic apply
(fn $apply (: (: @I32 -> @I32) @I32 -> @I32)
  ($f : (: @I32 -> @I32)) ($x : @I32)
  ($f $x))

(fn $double (: @I32 -> @I32) ($n : @I32) (* $n 2))

($apply $double 21)  ;; => 42
```

### Pattern matching on ADTs

```llml
;; Define an Option type and unwrap it
(ty @Option (sum (@Some @I32) (@None)))

(fn $unwrap_or (: @Option @I32 -> @I32)
  ($opt : @Option) ($default : @I32)
  (mat $opt
    ((@Some $v) $v)
    ((@None) $default)))

($unwrap_or (@Some 42) 0)  ;; => 42
($unwrap_or (@None) 0)     ;; => 0
```

### Do blocks with sequential bindings

```llml
(do
  (let $a : @I32 1)
  (let $b : @I32 (+ $a 1))
  (let $c : @I32 (+ $a $b))
  (let $msg : @Str ($str_concat "result: " ($to_str $c)))
  ($print $msg)
  $c)
;; Output: result: 3
;; => 3
```

### Expression evaluator (ADT + pattern matching)

```llml
;; Define a recursive expression tree
(ty @Expr (sum
  (@Num $val : @F64)
  (@Add $l : @Expr $r : @Expr)
  (@Mul $l : @Expr $r : @Expr)
  (@Neg $e : @Expr)))

;; Evaluate an expression tree to a float
(fn $eval (: @Expr -> @F64) ($e : @Expr)
  (mat $e
    ((@Num $v) $v)
    ((@Add $l $r) (+ ($eval $l) ($eval $r)))
    ((@Mul $l $r) (* ($eval $l) ($eval $r)))
    ((@Neg $inner) (- ($eval $inner)))))

;; Build (1.0 + 2.0) * 3.0 and evaluate
(let $expr : @Expr
  (@Mul (@Add (@Num 1.0) (@Num 2.0)) (@Num 3.0)))

($print (+ "Result: " ($to_str ($eval $expr))))
;; Output: Result: 9.0
```

### Shape area calculation

```llml
(ty @Shape (sum
  (@Circle @F64)
  (@Rectangle @F64 @F64)))

(fn $area (: @Shape -> @F64) ($s : @Shape)
  (mat $s
    ((@Circle $r) (* 3.14159 (* $r $r)))
    ((@Rectangle $w $h) (* $w $h))))

($print ($to_str ($area (@Circle 5.0))))
($print ($to_str ($area (@Rectangle 3.0 4.0))))
```

---

## Error Messages

### Parse errors

```
unexpected token `)` at byte 15, expected expression
```

**Fix:** Check for missing arguments or mismatched parentheses.

```
unexpected end of input, expected )
```

**Fix:** You have an unclosed parenthesis. Count your `(` and `)`.

### Runtime errors

```
runtime error: undefined variable: $foo
```

**Fix:** The variable `$foo` has not been defined. Check spelling and ensure it is bound via `let`, `fn` parameter, or pattern.

```
runtime error: arity mismatch: add expects 2 args, got 1
```

**Fix:** You called `$add` with the wrong number of arguments.

```
runtime error: type error: if condition must be @Bool
```

**Fix:** The condition in an `if` must be a boolean. Use a comparison operator like `(= $x 0)` instead of bare `$x`.

```
runtime error: match exhausted: no arm matched the value
```

**Fix:** Add a catch-all arm `($x ...)` or ensure all possible cases are covered.

```
runtime error: division by zero
```

**Fix:** Guard division with a check: `(if (= $d 0) 0 (/ $n $d))`.

```
runtime error: cannot call non-function value: 42
```

**Fix:** You tried to call a non-function. Ensure the callee is a `$function_name`, not a literal or other value.

---

## Anti-patterns

### DO NOT use infix operators

```llml
;; WRONG
1 + 2

;; CORRECT
(+ 1 2)
```

### DO NOT omit the else branch of if

```llml
;; WRONG -- missing else branch
(if (> $x 0) "positive")

;; CORRECT
(if (> $x 0) "positive" "non-positive")
```

### DO NOT forget sigils on variables and types

```llml
;; WRONG
(let x : I32 42)

;; CORRECT
(let $x : @I32 42)
```

### DO NOT use `match` -- the keyword is `mat`

```llml
;; WRONG
(match $x (0 "zero") ($n "other"))

;; CORRECT
(mat $x (0 "zero") ($n "other"))
```

### DO NOT use commas or semicolons

```llml
;; WRONG
(fn $add (: @I32, @I32 -> @I32) ($a : @I32, $b : @I32) (+ $a $b));

;; CORRECT
(fn $add (: @I32 @I32 -> @I32) ($a : @I32) ($b : @I32) (+ $a $b))
```

### DO NOT put parameters inside a single set of parentheses

```llml
;; WRONG -- parameters grouped together
(fn $add (: @I32 @I32 -> @I32) ($a : @I32 $b : @I32) (+ $a $b))

;; CORRECT -- each parameter gets its own parenthesized form
(fn $add (: @I32 @I32 -> @I32) ($a : @I32) ($b : @I32) (+ $a $b))
```

### DO NOT use bare keywords as identifiers

Keywords like `fn`, `let`, `if`, `mat`, `do`, `ty`, `mod`, `sum`, `prod`, `ret`, `true`, `false`, `nil` are reserved. Do not use them as variable or type names.

### DO NOT forget that `-` with one argument is negation

```llml
(- 5)    ;; => -5   (negation, not subtraction)
(- 5 3)  ;; => 2    (subtraction)
```

### DO NOT call a constructor like a function without parentheses when it has fields

```llml
;; WRONG
@Some 42

;; CORRECT
(@Some 42)
```

### DO NOT mix numeric types in operations

```llml
;; WRONG -- mixing @I32 and @F64
(+ 1 2.0)

;; CORRECT -- use the same type
(+ 1 2)
(+ 1.0 2.0)
```

---

## Keywords Reference

| Keyword | Purpose | Example |
|---------|---------|---------|
| `fn` | Define a function | `(fn $f (: @I32 -> @I32) ($x : @I32) $x)` |
| `let` | Bind a value | `(let $x : @I32 42)` |
| `if` | Conditional | `(if true 1 0)` |
| `mat` | Pattern match | `(mat $x (0 "zero") ($n "other"))` |
| `do` | Sequence expressions | `(do (let $x : @I32 1) $x)` |
| `ty` | Define a type | `(ty @Color (sum (@Red) (@Blue)))` |
| `sum` | Sum type body | Used inside `ty` |
| `prod` | Product type body | Used inside `ty` |
| `ret` | Early return | `(ret $value)` |
| `set` | Mutate a variable | `(set $x 10)` |
| `mut` | Mutable parameter | `($x : mut @I32)` |
| `mod` | Module | `(mod #utils ...)` |
| `pub` | Public visibility | `(pub (fn $f ...))` |
| `use` | Import | `(use #module)` |
| `true` | Boolean true | `true` |
| `false` | Boolean false | `false` |
| `nil` | Nil value | `nil` |

---

## Complete Program Template

```llml
;; Program description

;; Type definitions
(ty @MyType (sum
  (@Variant1 $field : @I32)
  (@Variant2)))

;; Function definitions
(fn $process (: @MyType -> @Str) ($val : @MyType)
  (mat $val
    ((@Variant1 $n) ($to_str $n))
    (@Variant2 "empty")))

;; Main logic
(let $value : @MyType (@Variant1 42))
($print ($process $value))
```
