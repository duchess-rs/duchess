/// Generates an `impl Trait` expression that is used as the type of a method argument
/// in a Rust function reflecting a Java method.
///
/// # Examples
///
/// * `int` expands to `impl IntoScalar<i32>`
/// * `int + 'a` expands to `impl IntoScalar<i32> + 'a`
/// * `(class[java::lang::Object])` expands to `impl IntoJava<java::lang::Object>`
/// * `(class[java::lang::Object]) + 'a` expands to `impl IntoJava<java::lang::Object> + 'a`
#[macro_export]
macro_rules! argument_impl_trait {
    ($scalar:ident $(+ $lt:lifetime)?) => {
        impl duchess::IntoScalar< duchess::plumbing::rust_ty!($scalar) > $(+ $lt)?
    };

    ($r:tt $(+ $lt:lifetime)?) => {
        impl duchess::IntoJava< duchess::plumbing::rust_ty!($r) > $(+ $lt)?
    };
}
