//! LLML Bytecode VM — stack-based execution engine.

pub mod bytecode;
pub mod compiler;
pub mod limits;
pub mod trace;
pub mod vm;

pub use compiler::{CompileError, compile};
pub use limits::Limits;
pub use trace::Trace;
pub use vm::{Vm, VmError, VmResult};

/// Convenience function: compile and run a program.
pub fn compile_and_run(
    program: &llml_parser::ast::Program,
    limits: Limits,
) -> Result<VmResult, Box<dyn std::error::Error>> {
    let proto = compile(program)?;
    let mut vm = Vm::new(limits);
    let result = vm.run(proto)?;
    Ok(result)
}
