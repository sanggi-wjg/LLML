# LLML Annotated Examples

This document walks through complete LLML programs with line-by-line explanations.

---

## 1. Hello World

```llml
;; hello.llml
;; Demonstrates function definition and string operations

;; Define a greeting function.
;; Type signature: takes a @Str, returns @Nil (because $print returns nil).
;; The body concatenates strings with + and prints the result.
(fn $greet (: @Str -> @Nil) ($name : @Str)
  ($print (+ "Hello, " (+ $name "!"))))

;; Call the function with a string argument.
($greet "LLML")
```

**Output:**
```
Hello, LLML!
```

**Key concepts:**
- `(fn $name (: @ParamTypes -> @ReturnType) ($param : @Type) body)` defines a function.
- `$print` is a built-in that outputs a string.
- `+` on strings performs concatenation.
- The program is a sequence of top-level declarations and expressions.

---

## 2. Fibonacci

```llml
;; fibonacci.llml
;; Recursive fibonacci using if-expressions

;; Define the fibonacci function.
;; Base case: if n <= 1, return n.
;; Recursive case: fib(n-1) + fib(n-2).
(fn $fib (: @I32 -> @I32)
  ($n : @I32)
  (if (<= $n 1) $n (+ ($fib (- $n 1)) ($fib (- $n 2)))))

;; Print fib(0) through fib(10).
;; $to_str converts an integer to a string for printing.
($print ($to_str ($fib 0)))
($print ($to_str ($fib 1)))
($print ($to_str ($fib 2)))
($print ($to_str ($fib 3)))
($print ($to_str ($fib 4)))
($print ($to_str ($fib 5)))
($print ($to_str ($fib 6)))
($print ($to_str ($fib 7)))
($print ($to_str ($fib 8)))
($print ($to_str ($fib 9)))
($print ($to_str ($fib 10)))
```

**Output:**
```
0
1
1
2
3
5
8
13
21
34
55
```

**Key concepts:**
- Recursive functions call themselves by name: `($fib (- $n 1))`.
- `if` is an expression with exactly three sub-expressions: condition, then, else.
- `$to_str` converts any value to its string representation.
- There are no loops; recursion is the primary iteration mechanism.

---

## 3. Expression Evaluator (ADT + Pattern Matching)

```llml
;; expr_eval.llml
;; Demonstrates algebraic data types and recursive pattern matching

;; Define an expression tree as a sum type.
;; @Num holds a float value.
;; @Add and @Mul each hold two sub-expressions (left and right).
;; @Neg holds one sub-expression to negate.
(ty @Expr (sum
  (@Num $val : @F64)
  (@Add $l : @Expr $r : @Expr)
  (@Mul $l : @Expr $r : @Expr)
  (@Neg $e : @Expr)))

;; Evaluate an expression tree to a float.
;; Uses `mat` (pattern match) to destructure each variant.
;; Each arm is (pattern body):
;;   (@Num $v) => return the value directly
;;   (@Add $l $r) => recursively eval both sides and add
;;   (@Mul $l $r) => recursively eval both sides and multiply
;;   (@Neg $inner) => negate the recursive evaluation
(fn $eval (: @Expr -> @F64) ($e : @Expr)
  (mat $e
    ((@Num $v) $v)
    ((@Add $l $r) (+ ($eval $l) ($eval $r)))
    ((@Mul $l $r) (* ($eval $l) ($eval $r)))
    ((@Neg $inner) (- ($eval $inner)))))

;; Build the expression tree for (1.0 + 2.0) * 3.0
;; This constructs nested constructor values.
(let $expr : @Expr
  (@Mul
    (@Add (@Num 1.0) (@Num 2.0))
    (@Num 3.0)))

;; Evaluate and print.
($print (+ "Result: " ($to_str ($eval $expr))))
```

**Output:**
```
Result: 9.0
```

**Key concepts:**
- `(ty @Name (sum ...))` defines a sum type (tagged union) with multiple variants.
- Variants can be recursive: `@Add` contains two `@Expr` sub-expressions.
- Constructors are called like functions: `(@Num 1.0)`, `(@Add left right)`.
- `mat` destructures constructors: `(@Add $l $r)` binds `$l` and `$r` to the fields.
- The evaluator is a classic recursive descent over the AST.

---

## 4. Higher-Order Functions

```llml
;; higher_order.llml
;; Demonstrates functions as values and higher-order composition

;; $twice takes a function and a value, applies the function twice.
;; The first parameter type is (: @I32 -> @I32) -- a function type.
(fn $twice (: (: @I32 -> @I32) @I32 -> @I32)
  ($f : (: @I32 -> @I32)) ($x : @I32)
  ($f ($f $x)))

;; Simple increment function.
(fn $inc (: @I32 -> @I32) ($n : @I32)
  (+ $n 1))

;; Simple doubling function.
(fn $double (: @I32 -> @I32) ($n : @I32)
  (* $n 2))

;; twice(inc, 5) => inc(inc(5)) => inc(6) => 7
($print ($to_str ($twice $inc 5)))

;; twice(double, 3) => double(double(3)) => double(6) => 12
($print ($to_str ($twice $double 3)))
```

**Output:**
```
7
12
```

**Key concepts:**
- Functions are first-class values: `$inc` and `$double` are passed as arguments.
- Function types in signatures use `(: @ParamType -> @ReturnType)`.
- When a parameter has a function type, its type annotation uses the same `(: ...)` syntax.
- `($f ($f $x))` calls `$f` on `$x`, then calls `$f` again on the result.

---

## 5. FizzBuzz

LLML does not have loops, so FizzBuzz is implemented with a helper function called repeatedly.

```llml
;; fizzbuzz.llml
;; FizzBuzz using recursive iteration

;; Classify a single number as fizz, buzz, fizzbuzz, or its string form.
(fn $fizzbuzz (: @I32 -> @Str) ($n : @I32)
  (if (= (% $n 15) 0) "FizzBuzz"
    (if (= (% $n 3) 0) "Fizz"
      (if (= (% $n 5) 0) "Buzz"
        ($to_str $n)))))

;; Print fizzbuzz for a range using recursion.
;; $from is the current number, $to is the upper bound (inclusive).
(fn $run (: @I32 @I32 -> @Nil)
  ($from : @I32) ($to : @I32)
  (if (> $from $to)
    nil
    (do
      ($print ($fizzbuzz $from))
      ($run (+ $from 1) $to))))

;; Run FizzBuzz from 1 to 20.
($run 1 20)
```

**Output:**
```
1
2
Fizz
4
Buzz
Fizz
7
8
Fizz
Buzz
11
Fizz
13
14
FizzBuzz
16
17
Fizz
19
Buzz
```

**Key concepts:**
- Without loops, recursion replaces `for`/`while`. The function `$run` calls itself with `(+ $from 1)`.
- `(% $n 15)` computes the remainder (modulo).
- Nested `if` expressions act as `if / else if / else` chains.
- `do` blocks sequence multiple side-effectful operations (print, then recurse).
- `nil` is used as the base-case return value when there is nothing meaningful to return.

---

## 6. Option Type and Safe Division

```llml
;; safe_division.llml
;; Demonstrates defining and using an Option type for error handling

;; Define an Option type that holds an @I32.
(ty @OptI32 (sum
  (@Some $val : @I32)
  @None))

;; Safe division: returns @None on division by zero.
(fn $safe_div (: @I32 @I32 -> @OptI32)
  ($a : @I32) ($b : @I32)
  (if (= $b 0)
    (@None)
    (@Some (/ $a $b))))

;; Unwrap an option with a default value.
(fn $unwrap_or (: @OptI32 @I32 -> @I32)
  ($opt : @OptI32) ($default : @I32)
  (mat $opt
    ((@Some $v) $v)
    ((@None) $default)))

;; Use it
(do
  (let $result1 : @OptI32 ($safe_div 10 3))
  (let $result2 : @OptI32 ($safe_div 10 0))

  ($print ($to_str ($unwrap_or $result1 0)))
  ($print ($to_str ($unwrap_or $result2 0))))
```

**Output:**
```
3
0
```

**Key concepts:**
- Sum types model success/failure without exceptions.
- `(@None)` constructs the empty variant; `(@Some value)` wraps a value.
- `mat` with `((@Some $v) $v)` and `((@None) $default)` handles both cases.
- This pattern replaces null checks and try/catch with explicit, total handling.

---

## 7. Product Type (Struct)

```llml
;; point.llml
;; Demonstrates product types (structs) and destructuring

;; Define a 2D point as a product type.
(ty @Point (prod $x : @I32 $y : @I32))

;; Construct a point.
(let $p : @Point (@Point 3 4))

;; Destructure with pattern matching to access fields.
(mat $p
  ((@Point $x $y)
    (do
      ($print ($str_concat "x = " ($to_str $x)))
      ($print ($str_concat "y = " ($to_str $y)))
      ($print ($str_concat "x^2 + y^2 = " ($to_str (+ (* $x $x) (* $y $y))))))))
```

**Output:**
```
x = 3
y = 4
x^2 + y^2 = 25
```

**Key concepts:**
- `(ty @Point (prod $x : @I32 $y : @I32))` defines a struct-like type.
- `(@Point 3 4)` constructs a value. The constructor name matches the type name.
- `mat` with `(@Point $x $y)` destructures the product to access individual fields.
- `$str_concat` joins multiple string arguments (alternative to chaining `+`).

---

## Summary of Patterns

| Pattern | When to use |
|---------|------------|
| `(fn $name ...)` at top level | Define reusable functions |
| `(if cond then else)` | Two-way branching |
| Nested `if` | Multi-way branching (no `cond`/`switch` construct) |
| `(mat expr (pat body) ...)` | Destructure ADTs, dispatch on values |
| `(do ...)` | Sequence let-bindings and side effects |
| `(ty @Name (sum ...))` | Model variants / enums / tagged unions |
| `(ty @Name (prod ...))` | Model structs / records |
| Recursion with base case | Iteration / looping |
| `(@None)` / `(@Some $v)` | Optional values / error handling |
| `($print ($to_str expr))` | Print non-string values |
