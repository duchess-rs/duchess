//! Experiments with Java-Rust interop.

mod array;
mod collections;
mod inspect;
mod jvm;
mod not_null;
mod ops;
mod str;

/// Contains reusable declarations for classes distributed by the JDK under the `java.*` packages.
pub mod java;

pub use duchess_macro::java_package;
pub use jni::errors::Result;
pub use jvm::Global;
pub use jvm::JavaObject;
pub use jvm::JavaType;
pub use jvm::Jvm;
pub use jvm::Local;

pub use prelude::*;

pub mod prelude {
    pub use crate::jvm::JvmOp;
    pub use crate::ops::{
        IntoJava, IntoLocal, IntoOptLocal, IntoRust, IntoScalar, IntoVoid, JavaMethod,
        ScalarMethod, VoidMethod,
    };
}

/// Internal module containing non-semver protected
/// names used by generated code.
pub mod plumbing {
    pub use crate::jvm::{FromJValue, JavaObjectExt, Upcast};
    pub use crate::str::{JavaString, ToJavaStringOp};
    pub use duchess_macro::duchess_javap;
}