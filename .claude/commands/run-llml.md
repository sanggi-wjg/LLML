# Run LLML Program

Execute an LLML program and display its output.

## Arguments
- `$ARGUMENTS`: Either a file path to a `.llml` file, or an inline LLML expression/program

## Instructions

1. **Determine input**:
   - If `$ARGUMENTS` contains a path ending in `.llml`, use it directly
   - If it's an inline expression (e.g., `(+ 1 2)` or `($print "hello")`), write it to a temporary file

2. **Execute**: Run `cargo run -q -p llml-cli -- run <file>`

3. **Display**: Show the program output. If there's an error, explain it clearly.

4. **Cleanup**: If a temp file was created, note it (don't delete — user may want to keep it).
