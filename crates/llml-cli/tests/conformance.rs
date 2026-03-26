//! Conformance test runner — runs all `.llml` files in `tests/conformance/`
//! and compares their output against `.expected` files.

use std::path::Path;

/// Run a single conformance test: parse, interpret, and compare output.
fn run_conformance_test(path: &Path) -> datatest_stable::Result<()> {
    let source = std::fs::read_to_string(path)?;

    let expected_path = path.with_extension("expected");
    let expected = std::fs::read_to_string(&expected_path)
        .map_err(|e| format!("missing expected file `{}`: {e}", expected_path.display()))?;

    let program = llml_parser::parse(&source).map_err(|e| format!("parse error: {e}"))?;

    let mut interp = llml_interp::Interpreter::new();
    let result = interp
        .exec_program(&program)
        .map_err(|e| format!("runtime error: {e}"))?;

    let mut actual = String::new();
    for line in interp.output() {
        actual.push_str(line);
        actual.push('\n');
    }
    if !matches!(result, llml_interp::Value::Nil) {
        actual.push_str(&result.to_string());
        actual.push('\n');
    }

    // Normalize trailing whitespace for comparison
    let actual = actual.trim_end().to_string();
    let expected = expected.trim_end().to_string();

    if actual != expected {
        return Err(format!(
            "output mismatch for `{}`:\n--- expected ---\n{expected}--- actual ---\n{actual}",
            path.display()
        )
        .into());
    }

    Ok(())
}

datatest_stable::harness! {
    { test = run_conformance_test, root = "../../tests/conformance", pattern = r"\.llml$" },
}
