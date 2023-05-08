use crate::{JDeref, Jvm, JvmOp};

/// Types that are able to be converted back into a Rust `T`, either because they will produce a Rust primitive `T` or
/// or because we can convert into them via a JNI call.
///
/// This is intended to be used to explicitly bring a value back to Rust at the end of a JVM session or operation.
pub trait ToRust {
    type Rust;

    fn to_rust<'jvm>(&self, jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, Self::Rust>;
}

pub struct ToRustOp<This>
where
    This: JvmOp,
{
    this: This,
}

impl<This, J> ToRustOp<This>
where
    This: JvmOp,
    for<'jvm> This::Output<'jvm>: JDeref<Java = J>,
    J: ToRust,
{
    pub(crate) fn new(this: This) -> Self {
        ToRustOp { this }
    }
}

impl<This, J> JvmOp for ToRustOp<This>
where
    This: JvmOp,
    for<'jvm> This::Output<'jvm>: JDeref<Java = J>,
    J: ToRust,
{
    type Output<'jvm> = J::Rust;

    fn execute<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, Self::Output<'jvm>> {
        let java = self.this.execute(jvm)?;
        let rust = java.jderef().to_rust(jvm)?;
        Ok(rust)
    }
}
