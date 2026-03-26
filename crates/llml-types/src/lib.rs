//! LLML Type Checker — static type checking for LLML programs.

pub mod checker;
pub mod context;
pub mod errors;
pub mod types;

pub use checker::check_program;
pub use context::TypeContext;
pub use errors::{TypeErrorKind, TypeErrors};
pub use types::Type;

/// Convenience function: type-check a program, returning errors if any.
pub fn check(program: &llml_parser::ast::Program) -> Result<(), TypeErrors> {
    let mut ctx = TypeContext::new();
    check_program(&mut ctx, program);
    if ctx.errors.is_empty() {
        Ok(())
    } else {
        Err(ctx.errors)
    }
}
