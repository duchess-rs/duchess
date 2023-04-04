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
pub use jvm::Local;

pub use prelude::*;

pub mod prelude {
    pub use crate::jvm::JvmOp;
    pub use crate::ops::{IntoJava, IntoLocal, IntoOptLocal, IntoRust, IntoScalar, IntoVoid};
}

/// Internal module containing non-semver protected
/// names used by generated code.
pub mod plumbing {
    pub use crate::collections::list::{ArrayList, List, ListExt};
    pub use crate::collections::map::{HashMap, Map, MapExt};
    pub use crate::jvm::{FromJValue, JavaObjectExt, Upcast};
    pub use crate::str::{JavaString, ToJavaStringOp};
    pub use duchess_macro::duchess_javap;
}

pub mod java {
    pub use crate::array::JavaArray as Array;

    pub mod lang {
        pub use crate::str::JavaString as String;
    }
    pub mod util {
        pub use crate::collections::list::ArrayList;
        pub use crate::collections::list::List;
        pub use crate::collections::map::HashMap;
    }
}
