# LLML — Language for Large Model Logic

**A programming language optimized for LLMs**

LLML is a programming language designed to maximize accuracy and minimize token consumption when LLMs (Large Language Models) generate, verify, and reason about code.

## Why?

Every existing programming language was designed for human cognition. LLML starts from a different question:

> What should syntax look like for an LLM to generate correct code most reliably?

## Design Principles

- **Token density** — Express the same semantics with 30-40% fewer tokens than Python
- **Structural unambiguity** — Every construct has exactly one interpretation (no dangling-else, no operator precedence issues)
- **Sigil system** — `$variable`, `@Type`, `#module`, `!effect`, `^Generic` make identifier kinds instantly recognizable
- **Explicit contracts** — No implicit conversions, no null, mandatory type annotations on all function signatures

## Quick Start

```bash
# Build
cargo build --workspace

# Run
cargo run -p llml-cli -- run examples/hello.llml
```

## Code Examples

```clojure
;; Fibonacci
(fn $fib (: @I32 -> @I32)
  ($n : @I32)
  (if (<= $n 1) $n (+ ($fib (- $n 1)) ($fib (- $n 2)))))

($print ($to_str ($fib 10)))  ;; 55
```

```clojure
;; Algebraic data types + pattern matching
(ty @Expr (sum
  (@Num $val : @F64)
  (@Add $l : @Expr $r : @Expr)
  (@Mul $l : @Expr $r : @Expr)))

(fn $eval (: @Expr -> @F64) ($e : @Expr)
  (mat $e
    ((@Num $v) $v)
    ((@Add $l $r) (+ ($eval $l) ($eval $r)))
    ((@Mul $l $r) (* ($eval $l) ($eval $r)))))
```

## Project Structure

```
crates/
├── llml-lexer/    # logos-based tokenizer
├── llml-parser/   # recursive descent parser
├── llml-interp/   # tree-walk interpreter
└── llml-cli/      # CLI (llml run/parse/lex)
```

## Documentation

- [SPEC.md](SPEC.md) — Language specification (EBNF grammar, type system, semantics)
- [docs/llm-guide.md](docs/llm-guide.md) — Complete LLML reference for LLMs
- [docs/llml-reference-card.md](docs/llml-reference-card.md) — Compact reference card for system prompts
- [docs/llm-integration.md](docs/llm-integration.md) — How to integrate LLML with other LLMs
- [docs/installation.md](docs/installation.md) — Build and setup instructions
- [docs/examples.md](docs/examples.md) — Annotated example programs
- [docs/roadmap.md](docs/roadmap.md) — Phase 2+ development plans

## License

MIT
