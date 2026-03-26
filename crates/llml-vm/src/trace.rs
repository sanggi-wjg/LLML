//! JSON execution trace for the VM.

use serde::Serialize;

/// A complete execution trace.
#[derive(Debug, Clone, Serialize)]
pub struct Trace {
    pub steps: Vec<TraceStep>,
}

/// A single step in the execution trace.
#[derive(Debug, Clone, Serialize)]
pub struct TraceStep {
    pub step: u64,
    pub op: String,
    pub stack_depth: usize,
    pub span_start: usize,
    pub span_end: usize,
}

impl Trace {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    pub fn record(&mut self, step: u64, op: &str, stack_depth: usize, start: usize, end: usize) {
        self.steps.push(TraceStep {
            step,
            op: op.to_string(),
            stack_depth,
            span_start: start,
            span_end: end,
        });
    }
}

impl Default for Trace {
    fn default() -> Self {
        Self::new()
    }
}
