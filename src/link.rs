use std::{ffi::CString, ptr::NonNull};

use crate::{java::lang::Class, Jvm, Local};

pub struct JavaFunction {
    pub(crate) name: CString,
    pub(crate) signature: CString,
    pub(crate) pointer: NonNull<()>,
    pub(crate) class_fn: ClassFn,
}

pub type ClassFn =
    for<'jvm> fn(jvm: &mut Jvm<'jvm>) -> crate::LocalResult<'jvm, Local<'jvm, Class>>;

impl JavaFunction {
    /// Create a new `JavaFunction` value with an appropriate name, signature, and function pointer.
    /// Don't call this directly. Instead, use [the `#[java_function]` decorator][java_fn].
    ///
    /// [java_fn]: https://duchess-rs.github.io/duchess/java_function.html
    ///
    /// # Panic
    ///
    /// Panics if `name` or `signature` contain nul values or cannot be converted into C strings.
    ///
    /// # Unsafe
    ///
    /// This function is unsafe because these values will be supplied to the JVM's
    /// [`RegisterNatives`](https://docs.oracle.com/en/java/javase/12/docs/specs/jni/functions.html#registernatives)
    /// function. If they are incorrect, undefined behavior will occur.
    pub unsafe fn new(
        name: &str,
        signature: &str,
        pointer: NonNull<()>,
        class_fn: ClassFn,
    ) -> Self {
        Self {
            name: CString::new(name).unwrap(),
            signature: CString::new(signature).unwrap(),
            pointer,
            class_fn,
        }
    }
}

/// Create a `JavaFunction` that can be linked into the JVM.
/// Implemented by [the `#[java_function]` decorator][java_fn].
///
/// [java_fn]: https://duchess-rs.github.io/duchess/java_function.html
pub trait JavaFn {
    fn java_fn() -> JavaFunction;
}

pub trait IntoJavaFns {
    fn into_java_fns(self) -> Vec<JavaFunction>;
}

impl IntoJavaFns for JavaFunction {
    fn into_java_fns(self) -> Vec<JavaFunction> {
        vec![self]
    }
}

impl IntoJavaFns for Vec<JavaFunction> {
    fn into_java_fns(self) -> Vec<JavaFunction> {
        self
    }
}
