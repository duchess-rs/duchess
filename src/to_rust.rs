use std::marker::PhantomData;

use crate::{JDeref, Jvm, JvmOp, TryJDeref};

/// Types that are able to be converted back into a Rust `T`, either because they will produce a Rust primitive `T` or
/// or because we can convert into them via a JNI call.
///
/// This is intended to be used to explicitly bring a value back to Rust at the end of a JVM session or operation.
pub trait ToRust<R> {
    fn to_rust<'jvm>(&self, jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, R>;
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
    for<'jvm> This::Output<'jvm>: JDeref,
    for<'jvm> <This::Output<'jvm> as TryJDeref>::Java: ToRust<R>,
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
    for<'jvm> This::Output<'jvm>: JDeref,
    for<'jvm> <This::Output<'jvm> as TryJDeref>::Java: ToRust<R>,
{
    type Output<'jvm> = R;

    fn execute<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, Self::Output<'jvm>> {
        let java = self.this.execute(jvm)?;
        let java = java.jderef();
        let rust = ToRust::to_rust(java, jvm)?;
        Ok(rust)
    }
}
