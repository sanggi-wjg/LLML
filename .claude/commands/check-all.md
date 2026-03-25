# Full Quality Check

Run the complete quality assurance pipeline for the LLML project.

## Instructions

Run ALL of these steps in sequence. Report each step's result before moving to the next.

1. **Format**: `cargo fmt --all -- --check`
   - If formatting issues found, run `cargo fmt --all` to fix them automatically
   - Report what was reformatted

2. **Lint**: `cargo clippy --workspace -- -D warnings`
   - Report any warnings or errors
   - If there are clippy warnings, fix them

3. **Unit Tests**: `cargo test --workspace`
   - Report pass/fail count for each crate
   - If any test fails, show the failure details

4. **Conformance Tests**: Run the conformance test suite
   - Execute each `.llml` + `.expected` pair in `tests/conformance/`
   - Report pass/fail with diffs for failures

5. **Summary**: Final report with:
   - Format: ✓ or ✗
   - Clippy: ✓ or ✗ (with count of warnings fixed)
   - Unit tests: N/N passed
   - Conformance: N/N passed
   - Overall: PASS or FAIL
