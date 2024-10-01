/// Returns an appropriate trait for a method that
/// returns `ty`. Assumes objects are nullable.
///
/// # Examples
///
/// * `'a, void` expands to `()`
/// * `'a, int` expands to `i32`
/// * `'a, (object[java::lang::Object])` expands to `Option<Local<'a, java::lang::Object>>`
#[macro_export]
macro_rules! output_type {
    ($lt:lifetime, void) => {
        ()
    };

    ($lt:lifetime, $scalar:ident) => {
        duchess::semver_unstable::rust_ty!($scalar)
    };

    ($lt:lifetime, $r:tt) => {
        Option<duchess::Local<$lt, duchess::semver_unstable::rust_ty!($r)>>
    };
}
