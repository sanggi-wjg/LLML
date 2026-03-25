# LLML Development Roadmap

Items identified during Phase 1 completion that should be addressed in future phases.

---

## Phase 2: Core Infrastructure

### Type Checker (`llml-types`)
**Priority**: High — Required for Phase 2

The interpreter currently performs no type checking. Expressions like `(+ "hello" 42)` only fail at runtime. A type checker should:

- Validate function signatures match call sites
- Enforce type annotations on `let` bindings
- Check pattern match exhaustiveness statically
- Track linear types (`~`) for resource safety
- Track effect types (`!`) in function signatures
- Report errors with source spans via `miette`

**Entry point**: New crate `crates/llml-types/` with `check(program: &Program) -> Result<TypedAST, TypeError>`

### Bytecode VM (`llml-vm`)
**Priority**: High — Required for Phase 2

Replace the tree-walk interpreter with a stack-based bytecode VM for:

- Deterministic execution with step counting
- Structured execution traces (JSON format for LLM consumption)
- Resource limits (max steps, max memory)
- Foundation for future WASM compilation

**Components**:
- `bytecode.rs` — Opcode definitions
- `compiler.rs` — AST/MIR → bytecode compilation
- `vm.rs` — Stack-based execution engine
- `trace.rs` — JSON execution trace output

### Standard Library (`llml-stdlib`)
**Priority**: Medium — Required for Phase 2

Built-in functions (`$print`, `$to_str`, etc.) are currently hardcoded in `crates/llml-interp/src/eval.rs`. Extract into a separate crate:

- `#std.io` — `$print`, `$read_line`
- `#std.str` — `$to_str`, `$str_concat`, `$len`
- `#std.math` — `$abs`, `$min`, `$max`, `$sqrt`
- `#std.collections` — `@List`, `@Map`, `@Set` operations

---

## Phase 2.5: Developer Experience

### Error Recovery in Parser
**Priority**: Medium — Required before LSP

The parser currently stops at the first error. For LSP support and better LLM feedback, it should:

- Continue parsing after an error (skip to next balanced paren)
- Collect multiple errors in a single pass
- Produce partial ASTs for incomplete programs
- Provide "did you mean?" suggestions for common mistakes

**Affected file**: `crates/llml-parser/src/parser.rs` — `parse_expr_after_lparen()` and `parse_decl_inner()`

### REPL Mode
**Priority**: Medium

Add `llml repl` subcommand for interactive execution:

- Read-eval-print loop with persistent environment
- Show types of evaluated expressions
- History and multi-line input support
- `:type <expr>` to show inferred type without evaluation
- `:ast <expr>` to show parsed AST

**Implementation**: Add `Commands::Repl` variant in `crates/llml-cli/src/main.rs`, use `rustyline` crate for line editing.

### Conformance Test Runner in Rust
**Priority**: Low — Quality of life

The conformance test suite currently runs via bash script or `/project:test-conformance` command. Embedding it as a Rust integration test provides:

- `cargo test` runs conformance tests automatically
- Proper error reporting with diffs
- Parallel test execution
- Test filtering (`cargo test --test conformance -- basic`)

**Implementation**: Add `tests/conformance_runner.rs` in `crates/llml-cli/` that:
1. Globs `tests/conformance/*.llml`
2. For each file, runs the interpreter and compares stdout to `.expected`
3. Uses `#[test]` per file via `test_case` or `datatest-stable` crate

---

## Phase 3: Tooling (Future)

- **LSP server** (`llml-lsp`) — Requires error recovery parser
- **WASM codegen** (`llml-codegen-wasm`) — Via Cranelift, requires MIR
- **MCP server** (`llml mcp-serve`) — Expose LLML as MCP tool for LLM agents
- **JSON structured output** — `llml check --json` for machine-readable diagnostics
- **Web playground** — Compiler compiled to WASM, running in browser

## Phase 4: Optimization (Future)

- **MIR optimization passes** — DCE, constant folding, inlining
- **SMT solver integration** — Compile-time `requires`/`ensures` verification via Z3
- **Incremental compilation** — Query-based architecture
- **Benchmark suite** — Performance regression tracking
