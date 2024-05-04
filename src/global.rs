use crate::{Global, JavaObject, Jvm, JvmOp, Local};

/// [`JvmOp`][] that converts a local result into a global one.
#[derive_where::derive_where(Copy, Clone)]
pub struct GlobalOp<J: JvmOp> {
    j: J,
}

impl<J: JvmOp> GlobalOp<J>
where
    J: JvmOp,
    for<'jvm> <J as JvmOp>::Output<'jvm>: IntoGlobal<'jvm>,
{
    pub(crate) fn new(j: J) -> Self {
        Self { j }
    }
}

impl<J> JvmOp for GlobalOp<J>
where
    J: JvmOp,
    for<'jvm> J::Output<'jvm>: IntoGlobal<'jvm>,
{
    type Output<'jvm> = GlobalVersionOf<'jvm, J::Output<'jvm>>;

    fn execute_with<'jvm>(
        self,
        jvm: &mut crate::Jvm<'jvm>,
    ) -> crate::LocalResult<'jvm, Self::Output<'jvm>> {
        let local = self.j.execute_with(jvm)?;
        local.into_global(jvm)
    }
}

pub type GlobalVersionOf<'jvm, T> = <T as IntoGlobal<'jvm>>::Output;

pub trait IntoGlobal<'jvm> {
    type Output;

    fn into_global(self, jvm: &mut Jvm<'jvm>) -> crate::LocalResult<'jvm, Self::Output>;
}

impl<'jvm, T> IntoGlobal<'jvm> for Local<'jvm, T>
where
    T: JavaObject,
{
    type Output = Global<T>;

    fn into_global(self, jvm: &mut Jvm<'jvm>) -> crate::LocalResult<'jvm, Self::Output> {
        Ok(jvm.global::<T>(&self))
    }
}

impl<'jvm, T> IntoGlobal<'jvm> for &T
where
    T: JavaObject,
{
    type Output = Global<T>;

    fn into_global(self, jvm: &mut Jvm<'jvm>) -> crate::LocalResult<'jvm, Self::Output> {
        Ok(jvm.global::<T>(self))
    }
}

impl<'jvm, T> IntoGlobal<'jvm> for Option<Local<'jvm, T>>
where
    T: JavaObject,
{
    type Output = Option<Global<T>>;

    fn into_global(self, jvm: &mut Jvm<'jvm>) -> crate::LocalResult<'jvm, Self::Output> {
        Ok(self.map(|p| jvm.global::<T>(&p)))
    }
}
