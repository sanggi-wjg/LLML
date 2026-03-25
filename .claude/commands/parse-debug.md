# Parse Debug Pipeline

Show the full compilation pipeline for an LLML source snippet or file.

## Arguments
- `$ARGUMENTS`: Either a file path (e.g., `examples/hello.llml`) or an inline LLML snippet (e.g., `(+ 1 2)`)

## Instructions

1. **Determine input**: If `$ARGUMENTS` looks like a file path (contains `.llml`), read that file. Otherwise, treat the argument as an inline LLML snippet.

2. **Stage 1 — Lexer**: Run `cargo run -q -p llml-cli -- lex <file>` to show all tokens with their byte spans. If inline snippet, write it to a temp file first.

3. **Stage 2 — Parser**: Run `cargo run -q -p llml-cli -- parse <file>` to show the AST structure.

4. **Stage 3 — Interpreter**: Run `cargo run -q -p llml-cli -- run <file>` to show execution output.

5. **Summary**: Present all three stages clearly labeled, highlighting any errors at each stage.

If any stage fails, show the error and explain what likely went wrong (e.g., "The lexer failed on `_` — this character is not in the token set" or "The parser expected `:` after a variable sigil in a parameter declaration").
