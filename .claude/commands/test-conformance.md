# Conformance Test Runner

Run all LLML conformance tests in `tests/conformance/`. Each test is a `.llml` source file paired with a `.expected` file containing expected stdout.

## Instructions

1. Build the workspace first: `cargo build --workspace`
2. For each `.llml` file in `tests/conformance/`:
   - Find its matching `.expected` file
   - Execute: `cargo run -q -p llml-cli -- run <file>`
   - Compare actual stdout with expected content
3. Report results:
   - List each test as PASS or FAIL
   - For failures, show a diff (expected vs actual, first 5 lines each)
   - Final summary: "N passed, M failed out of T total"
4. If all tests pass, confirm with a success message
5. If any tests fail, analyze the failure patterns and suggest likely causes

Use this bash pattern for running:
```
source "$HOME/.cargo/env"
for llml_file in tests/conformance/*.llml; do
  base=$(basename "$llml_file" .llml)
  expected_file="tests/conformance/${base}.expected"
  actual=$(cargo run -q -p llml-cli -- run "$llml_file" 2>&1)
  expected=$(cat "$expected_file")
  # compare and report
done
```
