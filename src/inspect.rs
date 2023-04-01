use crate::JvmOp;

#[derive(Clone)]
pub struct Inspect<J: JvmOp, K: JvmOp> {
    j: J,
    k: K,
}

impl<J, K> JvmOp for Inspect<J, K>
where
    J: JvmOp,
    K: JvmOp,
    for<'jvm> K: JvmOp<Input<'jvm> = J::Input<'jvm>>,
{
    type Input<'jvm> = J::Input<'jvm>;
    type Output<'jvm> = J::Output<'jvm>;

    fn execute_with<'jvm>(
        self,
        jvm: &mut crate::Jvm<'jvm>,
        input: J::Input<'jvm>,
    ) -> crate::Result<Self::Output<'jvm>> {
        let j = self.j.execute_with(jvm, input)?;

        // FIXME

        Ok(j)
    }
}
