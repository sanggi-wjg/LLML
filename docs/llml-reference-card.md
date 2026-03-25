# LLML Reference Card (for LLM System Prompts)

LLML is a programming language using modified s-expressions. Every compound form is `(keyword ...)`.

## Sigils
| Sigil | Meaning | Example |
|-------|---------|---------|
| `$` | variable | `$x`, `$name` |
| `@` | type | `@I32`, `@Str` |
| `^` | generic | `^T` |

## Primitives
`@I32` `@I64` `@F64` `@Str` `@Bool` `@Nil`

## Syntax Forms

```clojure
;; Comment

;; Let binding
(let $x : @I32 42)

;; Function
(fn $add (: @I32 @I32 -> @I32) ($a : @I32) ($b : @I32)
  (+ $a $b))

;; If (always 3 args: cond, then, else)
(if (> $x 0) "positive" "non-positive")

;; Pattern match
(mat $expr
  ((@Some $v) $v)
  (@None 0))

;; Do block (sequential, last expr = return value)
(do
  (let $a : @I32 1)
  (let $b : @I32 2)
  (+ $a $b))

;; Sum type
(ty @Option (: ^T) (sum (@Some $val : ^T) @None))

;; Product type
(ty @Point (prod $x : @F64 $y : @F64))

;; Constructor
(@Some 42)
@None

;; Function call
($add 1 2)
```

## Operators (prefix, always parenthesized)
`(+ a b)` `(- a b)` `(* a b)` `(/ a b)` `(% a b)`
`(= a b)` `(!= a b)` `(< a b)` `(> a b)` `(<= a b)` `(>= a b)`
`(&& a b)` `(|| a b)`

## Built-ins
`($print val)` `($to_str val)` `($str_concat a b)` `($len str)` `($not bool)` `($abs num)`

## Rules
1. All function signatures require explicit types: `(: @Param -> @Return)`
2. No implicit conversions. No null. No default arguments.
3. `if` always requires else branch.
4. `mat` arms: `(pattern body)`. Patterns: literal, `$var`, `@TypeName`, `(@Ctor $a $b)`.
5. Strings: double-quoted `"hello"`. Booleans: `true`/`false`. Unit: `nil`.
