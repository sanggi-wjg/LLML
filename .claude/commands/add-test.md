# Add Conformance Test

Create a new conformance test pair (.llml source + .expected output).

## Arguments
- `$ARGUMENTS`: A description of what the test should cover (e.g., "nested pattern matching with multiple constructors")

## Instructions

1. **Analyze the request**: Understand what language feature or behavior the test should exercise.

2. **Check existing tests**: Look at `tests/conformance/` to avoid duplicating coverage.

3. **Generate the test**:
   - Choose a descriptive snake_case name for the test files
   - Write the `.llml` file with:
     - A comment header describing the test
     - LLML code that exercises the feature
     - Use `($print ...)` to produce observable output
   - Write the `.expected` file with the exact expected stdout

4. **Verify the test**: Run `cargo run -q -p llml-cli -- run tests/conformance/<name>.llml` and compare output with the `.expected` file.

5. **Report**: Show the test files created and whether they pass.

## Naming Convention
- Feature tests: `<feature>_<detail>.llml` (e.g., `match_nested.llml`)
- Bug regression tests: `regression_<issue>.llml`
- Edge case tests: `edge_<case>.llml`
