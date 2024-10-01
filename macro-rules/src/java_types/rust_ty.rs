/// Convert a Java type to its corresponding Rust type.
///
/// # Examples
///
/// * `byte` expands to `i8`
/// * `(class[java::lang::Object])` expands to `java::lang::Object`
/// * `(class[java::util::List] (class[java::lang::Object]))` expands to `java::util::List<java::lang::Object>`
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
