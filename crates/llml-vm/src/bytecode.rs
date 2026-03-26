//! Bytecode instruction set and data structures.

use llml_lexer::Span;
use llml_stdlib::Value;

/// A single bytecode instruction.
#[derive(Debug, Clone)]
pub enum Op {
    /// Push a constant from the constant pool onto the stack.
    Const(u16),
    /// Pop the top of the stack.
    Pop,

    /// Push the value of a local variable onto the stack.
    GetLocal(u16),
    /// Set a local variable to the top of stack (does not pop).
    SetLocal(u16),

    // ── Arithmetic (integer) ──
    AddI,
    SubI,
    MulI,
    DivI,
    ModI,
    NegI,

    // ── Arithmetic (float) ──
    AddF,
    SubF,
    MulF,
    DivF,
    NegF,

    // ── String ──
    ConcatS,

    // ── Comparison ──
    EqI,
    NeqI,
    LtI,
    GtI,
    LeI,
    GeI,
    EqF,
    NeqF,
    LtF,
    GtF,
    LeF,
    GeF,
    EqS,
    NeqS,
    EqB,
    NeqB,

    // ── Boolean ──
    And,
    Or,
    Not,

    // ── Control flow ──
    /// Unconditional jump by offset (relative to next instruction).
    Jump(i32),
    /// Pop top; jump if false.
    JumpIfFalse(i32),

    // ── Functions ──
    /// Call function on stack with N arguments.
    Call(u8),
    /// Return from current function.
    Return,
    /// Create a closure from a function prototype constant.
    Closure(u16),

    // ── Constructors ──
    /// Construct a value: constant index for tag name, field count.
    Construct(u16, u8),
    /// Get the nth field of a constructor value on the stack top.
    GetField(u8),
    /// Test if stack top has a given tag (constant index). Push bool.
    TestTag(u16),

    // ── Built-ins ──
    /// Call a built-in function by constant index with N arguments.
    CallBuiltin(u16, u8),

    // ── Literals ──
    Nil,
    True,
    False,

    /// Halt execution.
    Halt,
}

/// A compiled bytecode chunk for a function.
#[derive(Debug, Clone)]
pub struct Chunk {
    /// The instruction stream.
    pub code: Vec<Op>,
    /// Constant pool.
    pub constants: Vec<Constant>,
    /// Source spans for each instruction (parallel to `code`).
    pub spans: Vec<Span>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            spans: Vec::new(),
        }
    }

    /// Emit an instruction and return its index.
    pub fn emit(&mut self, op: Op, span: Span) -> usize {
        let idx = self.code.len();
        self.code.push(op);
        self.spans.push(span);
        idx
    }

    /// Add a constant and return its index.
    pub fn add_constant(&mut self, constant: Constant) -> u16 {
        let idx = self.constants.len();
        self.constants.push(constant);
        idx as u16
    }

    /// Patch a jump instruction at `idx` with the correct offset.
    pub fn patch_jump(&mut self, idx: usize) {
        let offset = self.code.len() as i32 - idx as i32 - 1;
        match &mut self.code[idx] {
            Op::Jump(o) | Op::JumpIfFalse(o) => *o = offset,
            _ => panic!("tried to patch non-jump instruction"),
        }
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

/// Constants stored in the constant pool.
#[derive(Debug, Clone)]
pub enum Constant {
    Int(i64),
    Float(f64),
    Str(String),
    /// A compiled function prototype.
    Function(FunctionProto),
}

/// A compiled function prototype.
#[derive(Debug, Clone)]
pub struct FunctionProto {
    pub name: String,
    pub arity: u8,
    pub chunk: Chunk,
}

/// Convert a runtime Value to a Constant (for literal embedding).
impl From<&Value> for Constant {
    fn from(val: &Value) -> Self {
        match val {
            Value::Int(n) => Constant::Int(*n),
            Value::Float(n) => Constant::Float(*n),
            Value::Str(s) => Constant::Str(s.clone()),
            _ => unreachable!("cannot convert {val:?} to constant"),
        }
    }
}
