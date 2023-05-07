use crate::Jvm;

/// Types that are able to be converted back into a Rust `T`, either because they will produce a Rust primitive `T` or
/// or because we can convert into them via a JNI call.
///
/// This is intended to be used to explicitly bring a value back to Rust at the end of a JVM session or operation.
pub trait IntoRust<T> {
    fn into_rust<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, T>;
}
