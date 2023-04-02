use std::marker::PhantomData;

use crate::{jvm::CloneIn, JvmOp};

#[derive(Clone)]
pub struct Inspect<J: JvmOp, K: JvmOp> {
    j: J,
    k: K,
}

impl<J, K> Inspect<J, K>
where
    J: JvmOp,
    for<'jvm> J::Output<'jvm>: CloneIn<'jvm>,
    K: JvmOp,
    for<'jvm> K: JvmOp<Input<'jvm> = J::Output<'jvm>, Output<'jvm> = ()>,
{
    pub(crate) fn new(j: J, op: impl FnOnce(ArgOp<J>) -> K) -> Inspect<J, K> {
        let k = op(ArgOp {
            phantom: PhantomData,
        });
        Inspect { j, k }
    }
}

impl<J, K> JvmOp for Inspect<J, K>
where
    J: JvmOp,
    for<'jvm> J::Output<'jvm>: CloneIn<'jvm>,
    K: JvmOp,
    for<'jvm> K: JvmOp<Input<'jvm> = J::Output<'jvm>, Output<'jvm> = ()>,
{
    type Input<'jvm> = J::Input<'jvm>;
    type Output<'jvm> = J::Output<'jvm>;

    fn execute_with<'jvm>(
        self,
        jvm: &mut crate::Jvm<'jvm>,
        input: J::Input<'jvm>,
    ) -> crate::Result<Self::Output<'jvm>> {
        let j = self.j.execute_with(jvm, input)?;

        let j1 = j.clone_in(jvm);
        let () = self.k.execute_with(jvm, j1)?;

        Ok(j)
    }
}

#[derive(Clone)]
pub struct ArgOp<J: JvmOp> {
    phantom: PhantomData<J>,
}

impl<J> JvmOp for ArgOp<J>
where
    J: JvmOp,
{
    type Input<'jvm> = J::Output<'jvm>;
    type Output<'jvm> = J::Output<'jvm>;

    fn execute_with<'jvm>(
        self,
        _jvm: &mut crate::Jvm<'jvm>,
        arg: Self::Input<'jvm>,
    ) -> crate::Result<Self::Output<'jvm>> {
        Ok(arg)
    }
}
