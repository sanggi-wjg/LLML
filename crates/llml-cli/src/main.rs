mod repl;

use clap::{Parser, Subcommand, ValueEnum};
use miette::{Result, miette};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "llml", version = "0.1.0")]
#[command(about = "LLML — Language for Large Model Logic")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run an LLML source file
    Run {
        /// Path to the .llml source file
        file: PathBuf,
        /// Execution backend
        #[arg(long, default_value = "interp")]
        backend: Backend,
        /// Enable execution trace (VM backend only)
        #[arg(long)]
        trace: bool,
    },
    /// Parse and display the AST of an LLML source file
    Parse {
        /// Path to the .llml source file
        file: PathBuf,
    },
    /// Tokenize and display tokens of an LLML source file
    Lex {
        /// Path to the .llml source file
        file: PathBuf,
    },
    /// Start an interactive REPL
    Repl,
}

/// Execution backend.
#[derive(Debug, Clone, ValueEnum)]
enum Backend {
    /// Tree-walk interpreter (default)
    Interp,
    /// Bytecode VM
    Vm,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            file,
            backend,
            trace,
        } => cmd_run(&file, backend, trace),
        Commands::Parse { file } => cmd_parse(&file),
        Commands::Lex { file } => cmd_lex(&file),
        Commands::Repl => repl::run_repl(),
    }
}

fn read_source(path: &PathBuf) -> Result<String> {
    std::fs::read_to_string(path)
        .map_err(|e| miette!("cannot read file `{}`: {}", path.display(), e))
}

fn cmd_run(path: &PathBuf, backend: Backend, trace: bool) -> Result<()> {
    let source = read_source(path)?;

    let program = llml_parser::parse(&source).map_err(|e| miette!("{e}"))?;

    // Static type checking
    if let Err(errors) = llml_types::check(&program) {
        for err in &errors.errors {
            eprintln!("type warning: {err}");
        }
    }

    match backend {
        Backend::Interp => run_interp(&program),
        Backend::Vm => run_vm(&program, trace),
    }
}

fn run_interp(program: &llml_parser::ast::Program) -> Result<()> {
    let mut interp = llml_interp::Interpreter::new();
    match interp.exec_program(program) {
        Ok(val) => {
            for line in interp.output() {
                println!("{line}");
            }
            match &val {
                llml_interp::Value::Nil => {}
                v => println!("{v}"),
            }
            Ok(())
        }
        Err(e) => Err(miette!("runtime error: {e}")),
    }
}

fn run_vm(program: &llml_parser::ast::Program, trace: bool) -> Result<()> {
    let limits = llml_vm::Limits {
        trace,
        ..Default::default()
    };
    match llml_vm::compile_and_run(program, limits) {
        Ok(result) => {
            for line in &result.output {
                println!("{line}");
            }
            match &result.value {
                llml_stdlib::Value::Nil => {}
                v => println!("{v}"),
            }
            if let Some(trace) = &result.trace {
                eprintln!("--- trace ({} steps) ---", trace.steps.len());
                eprintln!(
                    "{}",
                    serde_json::to_string_pretty(trace).unwrap_or_default()
                );
            }
            eprintln!("(executed in {} steps)", result.steps);
            Ok(())
        }
        Err(e) => Err(miette!("vm error: {e}")),
    }
}

fn cmd_parse(path: &PathBuf) -> Result<()> {
    let source = read_source(path)?;
    let program = llml_parser::parse(&source).map_err(|e| miette!("{e}"))?;
    println!("{program:#?}");
    Ok(())
}

fn cmd_lex(path: &PathBuf) -> Result<()> {
    let source = read_source(path)?;
    let tokens = llml_lexer::tokenize(&source).map_err(|e| miette!("{e}"))?;
    for spanned in &tokens {
        println!(
            "{:>4}..{:<4} {:?}",
            spanned.span.start, spanned.span.end, spanned.token
        );
    }
    println!("({} tokens)", tokens.len());
    Ok(())
}
