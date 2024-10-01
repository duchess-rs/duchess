/// Returns an appropriate trait for a method that
/// returns `ty`. Assumes objects are nullable.
///
/// # Examples
///
/// * `int` expands to `impl ScalarField<i32>`
/// * `(class[java::lang::Object])` expands to `impl JavaField<java::lang::Object>`
#[macro_export]
macro_rules! field_output_trait {
    ($scalar:ident) => {
        impl duchess::ScalarField< duchess::plumbing::rust_ty!($scalar) >
    };

    ($r:tt) => {
        impl duchess::JavaField< duchess::plumbing::rust_ty!($r) >
    };
}
