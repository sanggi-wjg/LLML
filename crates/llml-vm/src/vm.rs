//! Stack-based bytecode virtual machine.

use crate::bytecode::{Chunk, Constant, FunctionProto, Op};
use crate::limits::Limits;
use crate::trace::Trace;
use llml_stdlib::{BuiltinRegistry, Value};

/// A call frame on the VM call stack.
#[derive(Debug, Clone)]
struct CallFrame {
    /// The function being executed.
    proto: FunctionProto,
    /// Instruction pointer into the chunk.
    ip: usize,
    /// Stack base offset for this frame's locals.
    base: usize,
}

/// The LLML bytecode virtual machine.
pub struct Vm {
    stack: Vec<Value>,
    frames: Vec<CallFrame>,
    step_count: u64,
    limits: Limits,
    output: Vec<String>,
    builtins: BuiltinRegistry,
    trace: Option<Trace>,
}

/// Result of VM execution.
pub struct VmResult {
    pub value: Value,
    pub output: Vec<String>,
    pub steps: u64,
    pub trace: Option<Trace>,
}

/// Runtime error from the VM.
#[derive(Debug, Clone, thiserror::Error)]
pub enum VmError {
    #[error("stack underflow")]
    StackUnderflow,
    #[error("step limit exceeded ({0})")]
    StepLimitExceeded(u64),
    #[error("stack overflow (depth {0})")]
    StackOverflow(usize),
    #[error("type error: {0}")]
    TypeError(String),
    #[error("division by zero")]
    DivisionByZero,
    #[error("not callable")]
    NotCallable,
    #[error("undefined builtin: {0}")]
    UndefinedBuiltin(String),
    #[error("match exhausted")]
    MatchExhausted,
    #[error("{0}")]
    Other(String),
}

type Result<T> = std::result::Result<T, VmError>;

impl Vm {
    /// Create a new VM with given limits.
    pub fn new(limits: Limits) -> Self {
        let trace = if limits.trace {
            Some(Trace::new())
        } else {
            None
        };
        Self {
            stack: Vec::with_capacity(256),
            frames: Vec::new(),
            step_count: 0,
            limits,
            output: Vec::new(),
            builtins: BuiltinRegistry::standard(),
            trace,
        }
    }

    /// Execute a compiled function prototype.
    pub fn run(&mut self, proto: FunctionProto) -> Result<VmResult> {
        self.frames.push(CallFrame {
            proto,
            ip: 0,
            base: 0,
        });

        let result = self.execute();

        let value = match result {
            Ok(()) => self.stack.pop().unwrap_or(Value::Nil),
            Err(e) => return Err(e),
        };

        Ok(VmResult {
            value,
            output: std::mem::take(&mut self.output),
            steps: self.step_count,
            trace: self.trace.take(),
        })
    }

    fn execute(&mut self) -> Result<()> {
        loop {
            // Extract what we need from the current frame without holding a borrow
            let (op, span, base) = {
                let frame = match self.frames.last() {
                    Some(f) => f,
                    None => return Ok(()),
                };
                if frame.ip >= frame.proto.chunk.code.len() {
                    return Ok(());
                }
                let op = frame.proto.chunk.code[frame.ip].clone();
                let span = frame.proto.chunk.spans[frame.ip];
                let base = frame.base;
                (op, span, base)
            };

            // Check limits
            self.step_count += 1;
            if self.step_count > self.limits.max_steps {
                return Err(VmError::StepLimitExceeded(self.limits.max_steps));
            }

            // Record trace if enabled
            if let Some(ref mut trace) = self.trace {
                trace.record(
                    self.step_count,
                    &format!("{op:?}"),
                    self.stack.len(),
                    span.start,
                    span.end,
                );
            }

            // Advance IP
            self.frames.last_mut().unwrap().ip += 1;

            match op {
                Op::Const(idx) => {
                    let val = {
                        let chunk = &self.frames.last().unwrap().proto.chunk;
                        match &chunk.constants[idx as usize] {
                            Constant::Int(n) => Value::Int(*n),
                            Constant::Float(n) => Value::Float(*n),
                            Constant::Str(s) => Value::Str(s.clone()),
                            Constant::Function(_) => Value::Nil,
                        }
                    };
                    self.push(val);
                }
                Op::Pop => {
                    self.pop()?;
                }
                Op::GetLocal(slot) => {
                    let val = self.stack[base + slot as usize].clone();
                    self.push(val);
                }
                Op::SetLocal(slot) => {
                    let val = self.peek()?.clone();
                    self.stack[base + slot as usize] = val;
                }
                Op::Nil => self.push(Value::Nil),
                Op::True => self.push(Value::Bool(true)),
                Op::False => self.push(Value::Bool(false)),

                // ── Arithmetic ──
                Op::AddI => self.binary_op(|a, b| match (a, b) {
                    (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x + y)),
                    (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x + y)),
                    (Value::Str(x), Value::Str(y)) => Ok(Value::Str(format!("{x}{y}"))),
                    _ => Err(VmError::TypeError("+ requires matching types".into())),
                })?,
                Op::SubI => self.binary_op(|a, b| match (a, b) {
                    (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x - y)),
                    (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x - y)),
                    _ => Err(VmError::TypeError("- requires numeric types".into())),
                })?,
                Op::MulI => self.binary_op(|a, b| match (a, b) {
                    (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x * y)),
                    (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x * y)),
                    _ => Err(VmError::TypeError("* requires numeric types".into())),
                })?,
                Op::DivI => self.binary_op(|a, b| match (a, b) {
                    (Value::Int(_), Value::Int(0)) => Err(VmError::DivisionByZero),
                    (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x / y)),
                    (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x / y)),
                    _ => Err(VmError::TypeError("/ requires numeric types".into())),
                })?,
                Op::ModI => self.binary_op(|a, b| match (a, b) {
                    (Value::Int(_), Value::Int(0)) => Err(VmError::DivisionByZero),
                    (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x % y)),
                    _ => Err(VmError::TypeError("% requires integer types".into())),
                })?,
                Op::NegI => {
                    let val = self.pop()?;
                    match val {
                        Value::Int(n) => self.push(Value::Int(-n)),
                        Value::Float(n) => self.push(Value::Float(-n)),
                        _ => return Err(VmError::TypeError("- requires numeric type".into())),
                    }
                }

                Op::AddF | Op::SubF | Op::MulF | Op::DivF | Op::NegF => {
                    // These are handled by the I variants which dispatch by type
                    unreachable!("float-specific ops not used in current compiler");
                }

                Op::ConcatS => self.binary_op(|a, b| Ok(Value::Str(format!("{a}{b}"))))?,

                // ── Comparison ──
                Op::EqI => self.binary_op(|a, b| Ok(Value::Bool(a == b)))?,
                Op::NeqI => self.binary_op(|a, b| Ok(Value::Bool(a != b)))?,
                Op::LtI => self.cmp_op(|a, b| a < b)?,
                Op::GtI => self.cmp_op(|a, b| a > b)?,
                Op::LeI => self.cmp_op(|a, b| a <= b)?,
                Op::GeI => self.cmp_op(|a, b| a >= b)?,

                Op::EqF
                | Op::NeqF
                | Op::LtF
                | Op::GtF
                | Op::LeF
                | Op::GeF
                | Op::EqS
                | Op::NeqS
                | Op::EqB
                | Op::NeqB => {
                    // Handled by I variants which dispatch by type
                    unreachable!("type-specific comparison ops not used");
                }

                // ── Boolean ──
                Op::And => self.binary_op(|a, b| match (a, b) {
                    (Value::Bool(x), Value::Bool(y)) => Ok(Value::Bool(x && y)),
                    _ => Err(VmError::TypeError("&& requires bool".into())),
                })?,
                Op::Or => self.binary_op(|a, b| match (a, b) {
                    (Value::Bool(x), Value::Bool(y)) => Ok(Value::Bool(x || y)),
                    _ => Err(VmError::TypeError("|| requires bool".into())),
                })?,
                Op::Not => {
                    let val = self.pop()?;
                    match val {
                        Value::Bool(b) => self.push(Value::Bool(!b)),
                        _ => return Err(VmError::TypeError("! requires bool".into())),
                    }
                }

                // ── Control flow ──
                Op::Jump(offset) => {
                    let frame = self.frames.last_mut().unwrap();
                    frame.ip = (frame.ip as i64 + offset as i64) as usize;
                }
                Op::JumpIfFalse(offset) => {
                    let val = self.pop()?;
                    if let Value::Bool(false) = val {
                        let frame = self.frames.last_mut().unwrap();
                        frame.ip = (frame.ip as i64 + offset as i64) as usize;
                    }
                }

                // ── Functions ──
                Op::Call(argc) => {
                    let argc = argc as usize;
                    let callee_idx = self.stack.len() - argc - 1;
                    let callee = self.stack[callee_idx].clone();

                    match callee {
                        Value::Fn(_closure) => {
                            // Find the FunctionProto from the closure name
                            // In our VM, closures store a reference to their proto
                            // For now, we need to get the proto from the constant pool
                            // This is a simplification — proper closures need upvalues
                            return Err(VmError::Other(
                                "closure calls not yet supported in VM — use tree-walk interpreter"
                                    .into(),
                            ));
                        }
                        Value::BuiltinFn(name) => {
                            let args: Vec<Value> = self.stack.drain(callee_idx + 1..).collect();
                            self.stack.pop(); // pop the builtin marker
                            let result = self
                                .builtins
                                .call(&name, &args, &mut self.output)
                                .map_err(|e| VmError::Other(e.to_string()))?;
                            self.push(result);
                        }
                        _ => return Err(VmError::NotCallable),
                    }
                }
                Op::Return => {
                    let result = self.pop()?;
                    let frame = self.frames.pop().unwrap();
                    // Clean up the stack to the frame's base
                    self.stack.truncate(frame.base);
                    self.push(result);

                    if self.frames.is_empty() {
                        return Ok(());
                    }
                }
                Op::Closure(const_idx) => {
                    let proto_name = {
                        let frame = self.frames.last().unwrap();
                        if let Constant::Function(proto) =
                            &frame.proto.chunk.constants[const_idx as usize]
                        {
                            proto.name.clone()
                        } else {
                            return Err(VmError::Other("expected function constant".into()));
                        }
                    };
                    self.push(Value::BuiltinFn(format!("__fn:{proto_name}")));
                }

                // ── Constructors ──
                Op::Construct(tag_idx, field_count) => {
                    let tag = {
                        let frame = self.frames.last().unwrap();
                        match &frame.proto.chunk.constants[tag_idx as usize] {
                            Constant::Str(s) => s.clone(),
                            _ => return Err(VmError::Other("expected string tag".into())),
                        }
                    };
                    let fc = field_count as usize;
                    let fields: Vec<Value> = if fc > 0 {
                        self.stack.drain(self.stack.len() - fc..).collect()
                    } else {
                        vec![]
                    };
                    self.push(Value::Constructor(tag, fields));
                }
                Op::GetField(idx) => {
                    let val = self.peek()?.clone();
                    match val {
                        Value::Constructor(_, fields) => {
                            if (idx as usize) < fields.len() {
                                self.push(fields[idx as usize].clone());
                            } else {
                                return Err(VmError::Other("field index out of bounds".into()));
                            }
                        }
                        _ => {
                            return Err(VmError::TypeError("GetField requires constructor".into()));
                        }
                    }
                }
                Op::TestTag(tag_idx) => {
                    let expected_tag = {
                        let frame = self.frames.last().unwrap();
                        match &frame.proto.chunk.constants[tag_idx as usize] {
                            Constant::Str(s) => s.clone(),
                            _ => return Err(VmError::Other("expected string tag".into())),
                        }
                    };
                    let val = self.peek()?.clone();
                    let matches = match &val {
                        Value::Constructor(tag, _) => tag == &expected_tag,
                        _ => false,
                    };
                    self.push(Value::Bool(matches));
                }

                // ── Built-ins ──
                Op::CallBuiltin(name_idx, argc) => {
                    let name = {
                        let frame = self.frames.last().unwrap();
                        match &frame.proto.chunk.constants[name_idx as usize] {
                            Constant::Str(s) => s.clone(),
                            _ => {
                                return Err(VmError::Other(
                                    "expected string for builtin name".into(),
                                ));
                            }
                        }
                    };
                    let argc = argc as usize;
                    if argc == 0 {
                        // This is a placeholder initialization, skip
                        self.push(Value::Nil);
                        continue;
                    }
                    let args: Vec<Value> = self.stack.drain(self.stack.len() - argc..).collect();
                    let result = self
                        .builtins
                        .call(&name, &args, &mut self.output)
                        .map_err(|e| VmError::Other(e.to_string()))?;
                    self.push(result);
                }

                Op::Halt => return Ok(()),
            }
        }
    }

    fn push(&mut self, val: Value) {
        self.stack.push(val);
    }

    fn pop(&mut self) -> Result<Value> {
        self.stack.pop().ok_or(VmError::StackUnderflow)
    }

    fn peek(&self) -> Result<&Value> {
        self.stack.last().ok_or(VmError::StackUnderflow)
    }

    fn binary_op(&mut self, f: impl FnOnce(Value, Value) -> Result<Value>) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        let result = f(a, b)?;
        self.push(result);
        Ok(())
    }

    fn cmp_op(&mut self, f: impl FnOnce(i64, i64) -> bool) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        match (&a, &b) {
            (Value::Int(x), Value::Int(y)) => {
                self.push(Value::Bool(f(*x, *y)));
                Ok(())
            }
            (Value::Float(x), Value::Float(y)) => {
                // For float comparison, apply the same comparison semantics
                // by using total_cmp which gives a consistent ordering
                let ord = x.total_cmp(y);
                let (xi, yi): (i64, i64) = match ord {
                    std::cmp::Ordering::Less => (0, 1),
                    std::cmp::Ordering::Equal => (0, 0),
                    std::cmp::Ordering::Greater => (1, 0),
                };
                self.push(Value::Bool(f(xi, yi)));
                Ok(())
            }
            _ => Err(VmError::TypeError(
                "comparison requires numeric types".into(),
            )),
        }
    }

    #[allow(dead_code)]
    fn load_constant(&self, chunk: &Chunk, idx: u16) -> Value {
        match &chunk.constants[idx as usize] {
            Constant::Int(n) => Value::Int(*n),
            Constant::Float(n) => Value::Float(*n),
            Constant::Str(s) => Value::Str(s.clone()),
            Constant::Function(_) => Value::Nil, // Shouldn't load function as value directly
        }
    }

    /// Get captured output.
    pub fn output(&self) -> &[String] {
        &self.output
    }
}
