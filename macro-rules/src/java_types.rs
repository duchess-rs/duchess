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
    ($scalar:ident $(+ $lt:lifetime)?) => {
        impl duchess::IntoScalar< duchess::plumbing::rust_ty!($scalar) > $(+ $lt)?
    };

    ($r:tt $(+ $lt:lifetime)?) => {
        impl duchess::IntoJava< duchess::plumbing::rust_ty!($r) > $(+ $lt)?
    };
}

/// Returns an appropriate trait for a method that
/// returns `ty`. Assumes objects are nullable.
#[macro_export]
macro_rules! output_type {
    ($lt:lifetime, void) => {
        ()
    };

    ($lt:lifetime, $scalar:ident) => {
        duchess::plumbing::rust_ty!($scalar)
    };

    ($lt:lifetime, $r:tt) => {
        Option<duchess::Local<$lt, duchess::plumbing::rust_ty!($r)>>
    };
}

/// Returns an appropriate trait for a method that
/// returns `ty`. Assumes objects are nullable.
#[macro_export]
macro_rules! output_trait {
    (void $(+ $lt:lifetime)?) => {
        impl duchess::VoidMethod $(+ $lt)?
    };

    ($scalar:ident $(+ $lt:lifetime)?) => {
        impl duchess::ScalarMethod< duchess::plumbing::rust_ty!($scalar) > $(+ $lt)?
    };

    ($r:tt $(+ $lt:lifetime)?) => {
        impl duchess::JavaMethod< duchess::plumbing::rust_ty!($r) > $(+ $lt)?
    };
}

/// Returns an appropriate trait for a method that
/// returns `ty`. Assumes objects are nullable.
#[macro_export]
macro_rules! field_output_trait {
    ($scalar:ident) => {
        impl duchess::ScalarField< duchess::plumbing::rust_ty!($scalar) >
    };

    ($r:tt) => {
        impl duchess::JavaField< duchess::plumbing::rust_ty!($r) >
    };
}

#[macro_export]
macro_rules! view_of_op {
    ($r:tt) => {
        <duchess::plumbing::rust_ty!($r) as duchess::plumbing::JavaView>::OfOp<Self>
    };
}

#[macro_export]
macro_rules! view_of_obj {
    ($r:tt) => {
        <duchess::plumbing::rust_ty!($r) as duchess::plumbing::JavaView>::OfObj<Self>
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
        u16
    };
    (boolean) => {
        bool
    };

    // Reference types

    ((class[$($path:tt)*])) => {
        $($path)*
    };
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

#[macro_export]
macro_rules! jni_static_field_get_fn {
    (byte) => {
        |env| env.GetStaticByteField
    };
    (short) => {
        |env| env.GetStaticShortField
    };
    (int) => {
        |env| env.GetStaticIntField
    };
    (long) => {
        |env| env.GetStaticLongField
    };
    (float) => {
        |env| env.GetStaticFloatField
    };
    (double) => {
        |env| env.GetStaticDoubleField
    };
    (char) => {
        |env| env.GetStaticCharField
    };
    (boolean) => {
        |env| env.GetStaticBooleanField
    };

    // Reference types
    ($r:tt) => {
        |env| env.GetStaticObjectField
    };
}
