/// Convert a Java type to its corresponding Rust type.
///
/// # Examples
///
/// * `byte` expands to `i8`
/// * `(class[duchess::java::lang::Object])` expands to `duchess::java::lang::Object`
/// * `(class[duchess::java::util::List] (class[duchess::java::lang::Object]))` expands to `duchess::java::util::List<duchess::java::lang::Object>`
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
        ($($path)* < $(duchess::semver_unstable::rust_ty!($args),)* >)
    };
    ((array $elem:tt)) => {
        duchess::java::Array<duchess::semver_unstable::rust_ty!($elem)>
    };
    ((generic $name:ident)) => {
        $name
    };
}
