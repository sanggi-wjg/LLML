//! Interactive REPL for LLML.

use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

/// Run the LLML REPL.
pub fn run_repl() -> miette::Result<()> {
    let mut rl = DefaultEditor::new().map_err(|e| miette::miette!("failed to init REPL: {e}"))?;
    let mut interp = llml_interp::Interpreter::new();

    println!("LLML REPL v0.1.0 — type :help for commands, :quit to exit");

    loop {
        let line = match rl.readline("llml> ") {
            Ok(line) => line,
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
                println!("Goodbye!");
                break;
            }
            Err(e) => {
                eprintln!("readline error: {e}");
                break;
            }
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let _ = rl.add_history_entry(&line);

        // Handle meta-commands
        if trimmed.starts_with(':') {
            handle_meta_command(trimmed, &mut interp);
            continue;
        }

        // Accumulate multi-line input if parens are unbalanced
        let source = match collect_multiline(&mut rl, &line) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("input error: {e}");
                continue;
            }
        };

        // Parse and execute
        match llml_parser::parse(&source) {
            Ok(program) => {
                // Clear output buffer from previous iteration
                let pre_output_len = interp.output().len();

                match interp.exec_program(&program) {
                    Ok(val) => {
                        // Print new output
                        for line in &interp.output()[pre_output_len..] {
                            println!("{line}");
                        }
                        if !matches!(val, llml_interp::Value::Nil) {
                            println!("=> {val}");
                        }
                    }
                    Err(e) => eprintln!("error: {e}"),
                }
            }
            Err(e) => eprintln!("parse error: {e}"),
        }
    }

    Ok(())
}

fn handle_meta_command(cmd: &str, _interp: &mut llml_interp::Interpreter) {
    match cmd {
        ":quit" | ":q" => {
            println!("Goodbye!");
            std::process::exit(0);
        }
        ":help" | ":h" => {
            println!("LLML REPL commands:");
            println!("  :help, :h     Show this help");
            println!("  :quit, :q     Exit the REPL");
            println!("  :ast <expr>   Parse and show AST");
            println!("  :type <expr>  Type-check an expression");
            println!("  :clear        Reset the environment");
        }
        ":clear" => {
            *_interp = llml_interp::Interpreter::new();
            println!("Environment cleared.");
        }
        _ if cmd.starts_with(":ast ") => {
            let source = &cmd[5..];
            match llml_parser::parse(source) {
                Ok(program) => println!("{program:#?}"),
                Err(e) => eprintln!("parse error: {e}"),
            }
        }
        _ if cmd.starts_with(":type ") => {
            let source = &cmd[6..];
            match llml_parser::parse(source) {
                Ok(program) => match llml_types::check(&program) {
                    Ok(()) => println!("type check: ok"),
                    Err(errors) => {
                        for err in &errors.errors {
                            eprintln!("type error: {err}");
                        }
                    }
                },
                Err(e) => eprintln!("parse error: {e}"),
            }
        }
        _ => {
            eprintln!("unknown command: {cmd}");
            eprintln!("type :help for available commands");
        }
    }
}

/// Collect multi-line input until parentheses are balanced.
fn collect_multiline(
    rl: &mut DefaultEditor,
    first_line: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut source = first_line.to_string();

    // Check if parens are balanced
    while !is_balanced(&source) {
        match rl.readline("...   ") {
            Ok(line) => {
                source.push('\n');
                source.push_str(&line);
            }
            Err(ReadlineError::Interrupted) => {
                return Err("interrupted".into());
            }
            Err(ReadlineError::Eof) => {
                return Err("unexpected end of input".into());
            }
            Err(e) => return Err(e.into()),
        }
    }

    Ok(source)
}

/// Check if parentheses are balanced in the source text.
fn is_balanced(source: &str) -> bool {
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape = false;

    for ch in source.chars() {
        if escape {
            escape = false;
            continue;
        }
        if ch == '\\' && in_string {
            escape = true;
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        // Skip comments
        if ch == ';' {
            // Rest of line is a comment — but we're iterating chars
            // Simple heuristic: skip ;; comments properly
            break; // conservative: assume rest of line is comment
        }
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ => {}
        }
    }

    depth <= 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balanced() {
        assert!(is_balanced("42"));
        assert!(is_balanced("(+ 1 2)"));
        assert!(is_balanced("(fn $f (: @I32 -> @I32) ($n : @I32) $n)"));
        assert!(!is_balanced("(+ 1"));
        assert!(!is_balanced("(fn $f (: @I32 -> @I32)"));
        assert!(is_balanced("\"hello\""));
        assert!(is_balanced("(+ 1 2) (+ 3 4)"));
    }
}
