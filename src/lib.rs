//! Experiments with Java-Rust interop.

pub use duchess_macro::duchess;

mod jvm;
mod ops;

pub use jvm::Global;
pub use jvm::JavaObject;
pub use jvm::JavaObjectExt;
pub use jvm::Jvm;
pub use jvm::JvmOp;
pub use jvm::Local;
