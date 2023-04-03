//! Experiments with Java-Rust interop.

mod array;
mod collections;
mod inspect;
mod jvm;
mod ops;
mod str;

pub use duchess_macro::duchess;
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
    pub use crate::collections::list::{ArrayList, List, ListExt};
    pub use crate::collections::map::{HashMap, Map, MapExt};
    pub use crate::jvm::{JavaObjectExt, Upcast};
    pub use crate::str::{IntoJavaString, JavaString, ToJavaStringOp};
}

pub use crate::array::JavaArray;

pub mod java {
    pub mod util {
        pub use crate::collections::list::ArrayList;
        pub use crate::collections::list::List;
        pub use crate::collections::map::HashMap;
    }
}
