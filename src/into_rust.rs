use std::marker::PhantomData;

use crate::{Java, JavaObject, Jvm, JvmOp, Local};

/// Types that are able to be converted back into a Rust `T`, either because they will produce a Rust primitive `T` or
/// or because we can convert into them via a JNI call.
///
/// This is intended to be used to explicitly bring a value back to Rust at the end of a JVM session or operation.
pub trait IntoRust<R> {
    fn into_rust<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::LocalResult<'jvm, R>;
}

macro_rules! identity_rust_op {
    ($($t:ty,)*) => {
        $(
            impl IntoRust<$t> for $t {
                fn into_rust<'jvm>(self, _jvm: &mut Jvm<'jvm>) -> crate::LocalResult<'jvm, $t> {
                    Ok(self)
                }
            }
        )*
    }
}

identity_rust_op! {
    (),
    bool,
    u16, // java char
    i8,
    i16,
    i32,
    i64,
}

impl<O, E, JO, JE> IntoRust<Result<O, E>> for Result<JO, JE>
where
    JO: IntoRust<O>,
    JE: IntoRust<E>,
{
    fn into_rust<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::LocalResult<'jvm, Result<O, E>> {
        match self {
            Ok(jo) => Ok(Ok(jo.into_rust(jvm)?)),
            Err(je) => Ok(Err(je.into_rust(jvm)?)),
        }
    }
}

impl<O, JO> IntoRust<Option<O>> for Option<JO>
where
    JO: IntoRust<O>,
{
    fn into_rust<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::LocalResult<'jvm, Option<O>> {
        match self {
            Some(jo) => Ok(Some(jo.into_rust(jvm)?)),
            None => Ok(None),
        }
    }
}

impl<J> IntoRust<Java<J>> for &J
where
    J: JavaObject,
{
    fn into_rust<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::LocalResult<'jvm, Java<J>> {
        Ok(jvm.global(self))
    }
}

impl<R, J> IntoRust<R> for Local<'_, J>
where
    J: JavaObject,
    for<'a> &'a J: IntoRust<R>,
{
    fn into_rust<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::LocalResult<'jvm, R> {
        <&J as IntoRust<R>>::into_rust(&self, jvm)
    }
}

impl<R, J> IntoRust<R> for &Local<'_, J>
where
    J: JavaObject,
    for<'a> &'a J: IntoRust<R>,
{
    fn into_rust<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::LocalResult<'jvm, R> {
        <&J as IntoRust<R>>::into_rust(self, jvm)
    }
}

impl<R, J> IntoRust<R> for Java<J>
where
    J: JavaObject,
    for<'a> &'a J: IntoRust<R>,
{
    fn into_rust<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::LocalResult<'jvm, R> {
        <&J as IntoRust<R>>::into_rust(&self, jvm)
    }
}

impl<R, J> IntoRust<R> for &Java<J>
where
    J: JavaObject,
    for<'a> &'a J: IntoRust<R>,
{
    fn into_rust<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::LocalResult<'jvm, R> {
        <&J as IntoRust<R>>::into_rust(self, jvm)
    }
}

#[derive_where::derive_where(Copy, Clone)]
pub struct ToRustOp<This, R>
where
    This: JvmOp,
{
    this: This,
    phantom: PhantomData<R>,
}

impl<This, R> ToRustOp<This, R>
where
    This: JvmOp,
    for<'jvm> This::Output<'jvm>: IntoRust<R>,
{
    pub(crate) fn new(this: This) -> Self {
        ToRustOp {
            this,
            phantom: PhantomData,
        }
    }
}

impl<This, R> JvmOp for ToRustOp<This, R>
where
    This: JvmOp,
    for<'jvm> This::Output<'jvm>: IntoRust<R>,
{
    type Output<'jvm> = R;

    fn execute_with<'jvm>(
        self,
        jvm: &mut Jvm<'jvm>,
    ) -> crate::LocalResult<'jvm, Self::Output<'jvm>> {
        let java = self.this.execute_with(jvm)?;
        let rust = IntoRust::into_rust(java, jvm)?;
        Ok(rust)
    }
}
