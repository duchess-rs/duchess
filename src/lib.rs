//! Experiments with Java-Rust interop.

mod array;
mod cast;
mod catch;
mod collections;
mod error;
mod inspect;
mod jvm;
mod not_null;
mod ops;
mod str;

/// Contains reusable declarations for classes distributed by the JDK under the `java.*` packages.
pub mod java;

pub use duchess_macro::java_package;
pub use error::{Error, GlobalResult, Result};
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
    pub use crate::cast::Upcast;
    pub use crate::catch::try_catch;
    pub use crate::jvm::{FromJValue, JavaObjectExt};
    pub use crate::str::ToJavaStringOp;
    pub use duchess_macro::duchess_javap;
}
