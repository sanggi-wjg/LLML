# LLML — Language for Large Model Logic

A programming language optimized for LLMs. Prioritizes token efficiency and structural unambiguity over human readability.

## Build & Test
- `cargo build --workspace` — build all crates
- `cargo test --workspace` — unit tests (lexer 8, parser 12, interp 28)
- `cargo fmt --all` — format code (PostToolUse hook auto-runs on .rs file edits)
- `cargo clippy --workspace` — lint
- `cargo run -p llml-cli -- run <file.llml>` — run an LLML program
- `cargo run -p llml-cli -- lex <file.llml>` — display token stream
- `cargo run -p llml-cli -- parse <file.llml>` — display AST

## Architecture
```
Source (.llml) → Lexer (logos) → Parser (recursive descent) → AST → Interpreter (tree-walk) → Output
```

| Crate | Role | Key Files |
|-------|------|-----------|
| `llml-lexer` | Modified s-expression tokenizer | `src/token.rs` — Token enum + `tokenize()` |
| `llml-parser` | Recursive descent parser | `src/ast.rs` — AST nodes, `src/parser.rs` — `parse()` |
| `llml-interp` | Tree-walk interpreter | `src/eval.rs` — `Interpreter`, `src/value.rs` — `Value` |
| `llml-cli` | CLI driver | `src/main.rs` — run/parse/lex subcommands |

## LLML Language Quick Reference
- All compound forms are `(keyword ...)` s-expressions
- Sigils: `$` variable, `@` type, `#` module, `!` effect, `^` generic
- Keywords: `fn` `let` `if` `mat` `do` `ty` `mod` `pub` `mut` `sum` `prod` `ret` `set`
- Comments: `;;`
- Built-in functions: `$print` `$to_str` `$str_concat` `$len` `$not` `$abs`

## Conventions
- Each crate must compile independently
- All public APIs must have doc comments
- Conformance tests live in `tests/conformance/` as `.llml` + `.expected` pairs
- LLML source files use `.llml` extension
- When adding new language features: update SPEC.md → write conformance tests → implement

## Documentation
- `SPEC.md` — Language specification (EBNF grammar, type system, semantics)
- `docs/llm-guide.md` — Complete LLML reference for LLMs (~890 lines, full tutorial)
- `docs/llml-reference-card.md` — Compact LLML reference for system prompts (~60 lines)
- `docs/llm-integration.md` — How to integrate LLML with other LLMs (tool use, MCP, system prompt)
- `docs/installation.md` — Build and setup instructions
- `docs/examples.md` — Annotated example programs
- `docs/roadmap.md` — Phase 2+ development plans (type checker, VM, REPL, LSP)

## Known Parser Behaviors
- Parameter `($name : @Type)` vs function call `($f $x)` disambiguation: 3-token lookahead (`(` + `VarSigil` + `:`)
- Type application only allows `^` generic parameters (`@List ^T`). Juxtaposition of `@` types is not application
- `do` blocks support `fn`, `ty`, `let` declarations (treated as declarations in expression position)
- `-` operator: `(- $x)` → unary negation, `(- $a $b)` → binary subtraction (distinguished by argument count)

## Custom Commands
- `/project:test-conformance` — run all 20 conformance tests
- `/project:parse-debug <file>` — show full Lex→Parse→Run pipeline
- `/project:check-all` — fmt + clippy + test + conformance full QA
- `/project:add-test <desc>` — auto-generate a conformance test
- `/project:run-llml <code>` — run an LLML program immediately
- `/project:token-compare <program>` — compare token efficiency across languages

## Custom Agents
- `spec-validator` — cross-reference SPEC.md ↔ test coverage
- `test-generator` — generate conformance tests from feature descriptions
- `llml-reviewer` — review LLML code (sigils, types, token efficiency)
- `compiler-debugger` — trace compiler bugs (step-by-step Lex→Parse→Eval analysis)
- `project-reviewer` — comprehensive project review (code quality, spec consistency, docs, tests, config)

## Development Phases
- **Phase 1 (complete)**: Core MVP — Lexer, Parser, Interpreter, CLI, 20 conformance tests
- **Phase 2 (next)**: Bytecode VM + Type Checker — `llml-types`, `llml-mir`, `llml-vm`
- **Phase 3**: Tooling — LSP, WASM codegen, JSON structured output
- **Phase 4**: Optimization — MIR passes, SMT verification, incremental compilation
