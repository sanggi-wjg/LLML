use clap::{Parser, Subcommand};
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
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file } => cmd_run(&file),
        Commands::Parse { file } => cmd_parse(&file),
        Commands::Lex { file } => cmd_lex(&file),
    }
}

fn read_source(path: &PathBuf) -> Result<String> {
    std::fs::read_to_string(path)
        .map_err(|e| miette!("cannot read file `{}`: {}", path.display(), e))
}

fn cmd_run(path: &PathBuf) -> Result<()> {
    let source = read_source(path)?;

    let program = llml_parser::parse(&source).map_err(|e| miette!("{e}"))?;

    let mut interp = llml_interp::Interpreter::new();
    match interp.exec_program(&program) {
        Ok(val) => {
            // Print captured output
            for line in interp.output() {
                println!("{line}");
            }
            // Print final value if not nil
            match &val {
                llml_interp::Value::Nil => {}
                v => println!("{v}"),
            }
            Ok(())
        }
        Err(e) => Err(miette!("runtime error: {e}")),
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
