//! Resource limit configuration for the VM.

/// Resource limits for VM execution.
#[derive(Debug, Clone)]
pub struct Limits {
    /// Maximum number of instructions to execute.
    pub max_steps: u64,
    /// Maximum call stack depth.
    pub max_stack_depth: usize,
    /// Whether to record an execution trace.
    pub trace: bool,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_steps: 1_000_000,
            max_stack_depth: 256,
            trace: false,
        }
    }
}
