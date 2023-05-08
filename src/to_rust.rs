use std::marker::PhantomData;

use crate::{Global, JavaObject, Jvm, JvmOp, Local};

/// Types that are able to be converted back into a Rust `T`, either because they will produce a Rust primitive `T` or
/// or because we can convert into them via a JNI call.
///
/// This is intended to be used to explicitly bring a value back to Rust at the end of a JVM session or operation.
pub trait ToRust<R> {
    fn to_rust<'jvm>(&self, jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, R>;
}

impl<O, E, JO, JE> ToRust<Result<O, E>> for Result<JO, JE>
where
    JO: ToRust<O>,
    JE: ToRust<E>,
{
    fn to_rust<'jvm>(&self, jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, Result<O, E>> {
        match self {
            Ok(jo) => Ok(Ok(jo.to_rust(jvm)?)),
            Err(je) => Ok(Err(je.to_rust(jvm)?)),
        }
    }
}

impl<O, JO> ToRust<Option<O>> for Option<JO>
where
    JO: ToRust<O>,
{
    fn to_rust<'jvm>(&self, jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, Option<O>> {
        match self {
            Some(jo) => Ok(Some(jo.to_rust(jvm)?)),
            None => Ok(None),
        }
    }
}

impl<R, J> ToRust<R> for Local<'_, J>
where
    J: JavaObject + ToRust<R>,
{
    fn to_rust<'jvm>(&self, jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, R> {
        J::to_rust(self, jvm)
    }
}

impl<R, J> ToRust<R> for Global<J>
where
    J: JavaObject + ToRust<R>,
{
    fn to_rust<'jvm>(&self, jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, R> {
        J::to_rust(&**self, jvm)
    }
}

impl<R, J> ToRust<R> for &J
where
    J: ToRust<R>,
{
    fn to_rust<'jvm>(&self, jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, R> {
        J::to_rust(self, jvm)
    }
}

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
    for<'jvm> This::Output<'jvm>: ToRust<R>,
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
    for<'jvm> This::Output<'jvm>: ToRust<R>,
{
    type Output<'jvm> = R;

    fn execute<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, Self::Output<'jvm>> {
        let java = self.this.execute(jvm)?;
        let rust = ToRust::to_rust(&java, jvm)?;
        Ok(rust)
    }
}
