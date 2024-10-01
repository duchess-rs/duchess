/// Creates the "object type" for `$r`, which should be a reference type.
/// See the [method resolution order][mro] docs for background on the "object type".
///
/// # Examples
///
/// * `(class[java::lang::Object])` expands to `<java::lang::Object as JavaView>::OfObj<Self>`
///
/// [mro]: https://duchess-rs.github.io/duchess/methods.html
#[macro_export]
macro_rules! view_of_obj {
    ($r:tt) => {
        <duchess::semver_unstable::rust_ty!($r) as duchess::semver_unstable::JavaView>::OfObj<Self>
    };
}
