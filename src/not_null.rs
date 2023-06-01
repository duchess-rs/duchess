use crate::{Error, JavaObject, JvmOp, Local};

#[derive_where::derive_where(Copy, Clone)]
pub struct NotNull<J: JvmOp> {
    j: J,
}

impl<J, T> NotNull<J>
where
    J: for<'jvm> JvmOp<Output<'jvm> = Option<Local<'jvm, T>>>,
    T: JavaObject,
{
    pub(crate) fn new(j: J) -> NotNull<J> {
        NotNull { j }
    }
}

impl<J, T> JvmOp for NotNull<J>
where
    J: for<'jvm> JvmOp<Output<'jvm> = Option<Local<'jvm, T>>>,
    T: JavaObject,
{
    type Output<'jvm> = Local<'jvm, T>;

    fn execute_with<'jvm>(
        self,
        jvm: &mut crate::Jvm<'jvm>,
    ) -> crate::Result<'jvm, Self::Output<'jvm>> {
        let j = self.j.execute_with(jvm)?;
        j.ok_or(Error::NullDeref)
    }
}
