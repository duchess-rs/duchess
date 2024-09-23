//! These macros take in a "java type description" and generate various Rust types to reflect it.
//! These java type descriptions are each a token tree and they have the following format:
//!
//! Java scalar types are a single identifier
//! * `int`
//! * `short`
//! * etc
//!
//! Java reference types are a `()`-token tree like:
//! * `(class[$path] $javaty*)`, e.g., `(class[java::util::Vector] (class[java::lang::String))` for `Vector<String>`
//! * `(array $javaty)`, e.g., `(array[(class[java::lang::String])])` for `String[]`
//! * `(generic $name)` to reference a generic (possible captured) type, e.g., `(generic[T])`

/// Generates an `impl Trait` expression that is used as the type of a method argument
/// in a Rust function reflecting a Java method.
#[macro_export]
macro_rules! argument_impl_trait {
    ($scalar:ident) => {
        impl IntoScalar< duchess::plumbing::rust_ty!($scalar) >
    };

    ($r:tt) => {
        impl IntoJava< duchess::plumbing::rust_ty!($r) >
    };
}

/// Generates an `impl Trait` expression that is used as the type of a method argument
/// in a Rust function reflecting a Java method.
#[macro_export]
macro_rules! argument_op {
    ($scalar:ident) => {
        impl JvmScalarOp< duchess::plumbing::rust_ty!($scalar) >
    };

    ($r:tt) => {
        impl JvmRefOp< duchess::plumbing::rust_ty!($r) >
    };
}

#[macro_export]
macro_rules! rust_ty {
    // Scalar types

    (byte) => {
        i8
    };
    (short) => {
        i16
    };
    (int) => {
        i32
    };
    (long) => {
        i64
    };
    (float) => {
        f32
    };
    (double) => {
        f64
    };
    (char) => {
        char
    };
    (boolean) => {
        bool
    };

    // Reference types

    ((class[$($path:tt)*] $($args:tt)*)) => {
        ($($path)* < $(duchess::plumbing::rust_ty!($args),)* >)
    };
    ((array $elem:tt)) => {
        java::Array<duchess::plumbing::rust_ty!($elem)>
    };
    ((generic $name:ident)) => {
        $name
    };
}
