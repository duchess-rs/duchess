/// Returns an appropriate trait for a method that
/// returns `ty`. Assumes objects are nullable.
///
/// # Examples
///
/// * `void` expands to `impl VoidMethod`
/// * `void + 'a` expands to `impl VoidMethod + 'a`
/// * `int` expands to `impl ScalarMethod<i32>`
/// * `(object[java::lang::Object])` expands to `impl JavaMethod<java::lang::Object>`
/// * `(object[java::lang::Object]) + 'a` expands to `impl JavaMethod<java::lang::Object> + 'a`
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
