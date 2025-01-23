//! Experiments with Java-Rust interop.

mod array;
mod cast;
mod error;
mod find;
mod from_ref;
mod into_rust;
mod jvm;
mod libjvm;
mod link;
mod not_null;
mod null;
mod ops;
mod raw;
mod ref_;
mod refs;
mod str;
mod thread;
mod to_java;
mod try_catch;

/// Contains reusable declarations for classes distributed by the JDK under the `java.*` packages.
pub mod java;

pub use duchess_macro::{java_function, java_package, ToJava, ToRust};
pub use error::{Error, LocalResult, Result};
pub use into_rust::IntoRust;
pub use jvm::JavaObject;
pub use jvm::JavaType;
pub use jvm::Jvm;
pub use link::JavaFunction;
pub use null::Null;
pub use ref_::{Java, Local};
pub use refs::{AsJRef, JDeref, NullJRef, Nullable, TryJDeref};
pub use try_catch::TryCatch;

pub use prelude::*;

/// Contains traits with methods expected to be invoked by end-users.
pub mod prelude {
    pub use crate::java;
    pub use crate::jvm::JvmOp;
    pub use crate::link::JavaFn;
    pub use crate::ops::{
        IntoJava, IntoScalar, IntoVoid, JavaConstructor, JavaField, JavaMethod, ScalarField,
        ScalarMethod, VoidMethod,
    };
    pub use crate::refs::{AsJRef, JDeref, TryJDeref};
    pub use crate::to_java::ToJava;
    pub use crate::Java;
}

/// Internal module containing non-semver protected
/// names used by generated code.
#[doc(hidden)]
pub mod semver_unstable {
    pub use crate::cast::Upcast;
    pub use crate::find::{find_class, find_constructor, find_field, find_method};
    pub use crate::from_ref::FromRef;
    pub use crate::jvm::native_function_returning_object;
    pub use crate::jvm::native_function_returning_scalar;
    pub use crate::jvm::native_function_returning_unit;
    pub use crate::jvm::JavaObjectExt;
    pub use crate::jvm::JavaView;
    pub use crate::jvm::JvmRefOp;
    pub use crate::jvm::JvmScalarOp;
    pub use crate::link::JavaFn;
    pub use crate::link::JavaFunction;
    pub use crate::raw::{EnvPtr, FieldPtr, FromJniValue, IntoJniValue, MethodPtr, ObjectPtr};
    pub use crate::refs::NullJRef;
    pub use crate::to_java::{ToJavaImpl, ToJavaScalar};
    pub use duchess_macro_rules::{
        argument_impl_trait, field_output_trait, jni_call_fn, jni_static_call_fn,
        jni_static_field_get_fn, macro_if, mro, output_trait, output_type, prepare_input, rust_ty,
        setup_class, setup_constructor, setup_inherent_object_method, setup_obj_method,
        setup_op_method, setup_static_field_getter, setup_static_method, view_of_obj, view_of_op,
        setup_java_function,
    };
    pub use jni_sys;
    pub use once_cell;
}
