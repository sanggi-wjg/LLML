//! LLML Standard Library — shared value types and built-in functions.

pub mod io;
pub mod math;
pub mod registry;
pub mod str;
pub mod value;

pub use registry::{BuiltinError, BuiltinRegistry};
pub use value::{Env, FnClosure, SetError, Value};
