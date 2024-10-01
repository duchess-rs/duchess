/// Prepares an input from a JVM method call to be passed to JNI.
///
/// Expected to be used in the `JvmOp` impl for some struct that was
/// created to represent the method call; each method input is expected
/// to be stored in a field whose type implements either `JvmRefOp`
/// or `JvmScalarOp`, appropriately.
///
/// Used like `prepare_input!(let $O = ($self.$I: $I_ty) in $jvm)`, the parameters are
///
/// * `$O`: name of the local variable we will define (usually same as `$I`)
/// * `$self`: the struct representing the method call
/// * `$I`: the field in the struct that holds the input
/// * `$I_ty`: the (Java) type of the input
/// * `$jvm`: the `Jvm` instance that will be used to prepare the input
#[macro_export]
macro_rules! prepare_input {
    (let $O:ident = ($self:ident.$I:ident: $I_scalar_ty:ident) in $jvm:ident) => {
        let $O = $self.$I.do_jni($jvm)?;
    };

    (let $O:ident = ($self:ident.$I:ident: $I_ref_ty:tt) in $jvm:ident) => {
        let $O = $self.$I.into_as_jref($jvm)?;
        let $O = match duchess::prelude::AsJRef::as_jref(&$I) {
            Ok(v) => Some(v),
            Err(duchess::NullJRef) => None,
        };
    };
}
