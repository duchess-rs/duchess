use crate::{JavaObject, JvmOp, Local, TryJDeref};

#[derive_where::derive_where(Clone)]
#[derive_where(Copy; J: Copy)]
pub struct NotNull<J: JvmOp> {
    j: J,
}

impl<J, T> NotNull<J>
where
    for<'jvm> J: JvmOp<Output<'jvm>: TryJDeref<Java = T>>,
    T: JavaObject,
{
    pub(crate) fn new(j: J) -> NotNull<J> {
        NotNull { j }
    }
}

impl<J, T> JvmOp for NotNull<J>
where
    for<'jvm> J: JvmOp<Output<'jvm>: TryJDeref<Java = T>>,
    T: JavaObject,
{
    type Output<'jvm> = Local<'jvm, T>;

    fn do_jni<'jvm>(
        self,
        jvm: &mut crate::Jvm<'jvm>,
    ) -> crate::LocalResult<'jvm, Self::Output<'jvm>> {
        let deref = self.j.do_jni(jvm)?;
        let j = deref.try_jderef()?;
        Ok(jvm.local(j))
    }
}
