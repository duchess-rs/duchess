//! Experiments with Java-Rust interop.

mod array;
mod cast;
mod catch;
mod error;
mod find;
mod inspect;
mod jvm;
mod libjvm;
mod not_null;
mod ops;
mod raw;
mod ref_;
mod str;
mod thread;

/// Contains reusable declarations for classes distributed by the JDK under the `java.*` packages.
pub mod java;

pub use duchess_macro::java_package;
pub use error::{Error, GlobalResult, Result};
pub use jvm::JavaObject;
pub use jvm::JavaType;
pub use jvm::Jvm;
pub use ref_::{Global, Local};

pub use prelude::*;

/// Re-export the dependencies that are used by the generated code.
pub mod codegen_deps {
    pub use once_cell;
}

pub mod prelude {
    pub use crate::cast::by_type;
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
    pub use crate::error::check_exception;
    pub use crate::find::{find_class, find_constructor, find_method};
    pub use crate::jvm::JavaObjectExt;
    pub use crate::raw::{FromJniValue, HasEnvPtr, IntoJniValue, MethodPtr, ObjectPtr};
}
