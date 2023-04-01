//! Experiments with Java-Rust interop.

pub use duchess_macro::duchess;

mod ops;

/// Internal module containing non-semver protected
/// names used by generated code.
pub mod plumbing;

pub use plumbing::Global;
pub use plumbing::JavaObject;
pub use plumbing::Jvm;
pub use plumbing::JvmOp;
pub use plumbing::Local;
