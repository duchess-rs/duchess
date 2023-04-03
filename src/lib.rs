//! Experiments with Java-Rust interop.

pub use duchess_macro::duchess;

mod array;
mod inspect;
mod jvm;
mod ops;
mod str;

pub use jni::errors::Result;
pub use jvm::Global;
pub use jvm::JavaObject;
pub use jvm::Jvm;
pub use jvm::JvmOp;
pub use jvm::Local;

pub use ops::{IntoJava, IntoRust};

/// Internal module containing non-semver protected
/// names used by generated code.
pub mod plumbing {
    pub use crate::array::IntoJavaArray;
    pub use crate::jvm::JavaObjectExt;
    pub use crate::str::{IntoJavaString, JavaString, ToJavaStringOp};
}
