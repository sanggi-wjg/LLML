# LLML Installation and Setup

## Prerequisites

- **Rust toolchain** (1.85+ recommended, edition 2024)
  - Install via [rustup](https://rustup.rs/): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Cargo** (included with Rust)
- **Git** (to clone the repository)

## Building from Source

```bash
# Clone the repository
git clone <repo-url>
cd LLML

# Build all crates in the workspace
cargo build --workspace

# Build in release mode (optimized)
cargo build --workspace --release
```

The LLML workspace contains four crates:

| Crate | Purpose |
|-------|---------|
| `llml-lexer` | Logos-based tokenizer for LLML's modified s-expression syntax |
| `llml-parser` | Recursive descent parser producing an AST |
| `llml-interp` | Tree-walk interpreter for AST evaluation |
| `llml-cli` | CLI driver binary (`llml`) |

## Installing the CLI

```bash
# Install the llml binary to ~/.cargo/bin/
cargo install --path crates/llml-cli
```

After installation, `llml` is available in your PATH (assuming `~/.cargo/bin` is on your PATH).

## Running Programs

### `llml run` -- Execute an LLML program

```bash
llml run path/to/program.llml
```

This lexes, parses, and interprets the file. Output from `$print` calls is written to stdout. If the program's final expression produces a non-nil value, that value is also printed.

Example:

```bash
$ llml run examples/hello.llml
Hello, LLML!
```

### `llml parse` -- Display the AST

```bash
llml parse path/to/program.llml
```

Parses the file and prints the AST in Rust debug format. Useful for debugging syntax issues or understanding how LLML structures code.

### `llml lex` -- Display the token stream

```bash
llml lex path/to/program.llml
```

Tokenizes the file and prints every token with its byte-offset span. Useful for diagnosing lexer-level issues.

Example output:

```
   0..1   (
   1..3   fn
   4..8   $greet
   ...
(42 tokens)
```

## Running Tests

```bash
# Run all tests across the workspace
cargo test --workspace

# Run tests for a specific crate
cargo test -p llml-lexer
cargo test -p llml-parser
cargo test -p llml-interp
```

### Conformance tests

Conformance tests live in `tests/conformance/` as paired `.llml` and `.expected` files. Each `.llml` file is a program; the `.expected` file contains the expected stdout output. These are run as part of the workspace test suite.

## Development

```bash
# Format all code
cargo fmt --all

# Lint all code
cargo clippy --workspace

# Build and test in one step
cargo test --workspace
```

## File Extension

LLML source files use the `.llml` extension. Example: `hello.llml`, `fibonacci.llml`.

## Project Structure

```
LLML/
  Cargo.toml              # Workspace root
  crates/
    llml-lexer/            # Tokenizer
    llml-parser/           # Parser and AST
    llml-interp/           # Interpreter
    llml-cli/              # CLI binary
  examples/                # Example LLML programs
    hello.llml
    fibonacci.llml
    expr_eval.llml
    higher_order.llml
  tests/
    conformance/           # Conformance test pairs (.llml + .expected)
  docs/                    # Documentation
```
