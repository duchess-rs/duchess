/// Creates the "operation type" for `$r`, which should be a reference type.
/// See the [method resolution order][mro] docs for background on the "operation type".
///
/// # Examples
///
/// * `(class[java::lang::Object])` expands to `<java::lang::Object as JavaView>::OfOp<Self>`
///
/// [mro]: https://duchess-rs.github.io/duchess/methods.html
#[macro_export]
macro_rules! view_of_op {
    ($r:tt) => {
        <duchess::plumbing::rust_ty!($r) as duchess::plumbing::JavaView>::OfOp<Self>
    };
}
