use jni::{objects::{JObject, AutoLocal}, JNIEnv};

use crate::{JvmOp, Local, java::lang::Throwable, error::Error};

/// Plumbing utility to check the exception state of the JVM thread convert it into a [`crate::Result`].
// XX: many of the jni crate checked methods do this automatically. We only need this if/when we invoke
// unchecked methods.
pub fn try_catch<'jvm>(env: &mut JNIEnv<'jvm>) -> crate::Result<'jvm, ()> {
    let exception = env.exception_occurred()?;
    if exception.is_null() {
        Ok(())
    } else {
        env.exception_clear()?;
        let obj: JObject = exception.into();
        Err(crate::Error::Thrown(unsafe { Local::from_jni(AutoLocal::new(obj, &env)) }))
    }
}

#[derive(Clone)]
pub struct Catch<J, K> {
    op: J,
    catch: K,
}

impl<J, K> Catch<J, K>
where
    J: JvmOp,
    K: JvmOp,
    for<'jvm> K: JvmOp<Input<'jvm> = Local<'jvm, Throwable>>,
    for<'jvm> K::Output<'jvm>: Into<J::Output<'jvm>>,
{
    pub(crate) fn new(op: J, catch: impl FnOnce(ThrownOp) -> K) -> Catch<J, K> {
        let catch = catch(ThrownOp { _private: () });
        Catch { op, catch }
    }
}

impl<J, F> JvmOp for Catch<J, F> 
where
    J: JvmOp,
    F: JvmOp,
    for<'jvm> F: JvmOp<Input<'jvm> = Local<'jvm, Throwable>>,
    for<'jvm> F::Output<'jvm>: Into<J::Output<'jvm>>,
{
    type Input<'jvm> = J::Input<'jvm>;
    type Output<'jvm> = J::Output<'jvm>;

    fn execute_with<'jvm>(
        self,
        jvm: &mut crate::Jvm<'jvm>,
        arg: Self::Input<'jvm>,
    ) -> crate::Result<'jvm, Self::Output<'jvm>> {
        self.op.execute_with(jvm, arg).or_else(|e| match e.extract_thrown(jvm) {
            Error::Thrown(t) => self.catch.execute_with(jvm, t).map(|v| v.into()),
            e => Err(e),
        })
    }
}

#[derive(Clone)]
pub struct ThrownOp {
    _private: (),
}

impl JvmOp for ThrownOp {
    type Input<'jvm> = Local<'jvm, Throwable>;
    type Output<'jvm> = Local<'jvm, Throwable>;

    fn execute_with<'jvm>(
        self,
        _jvm: &mut crate::Jvm<'jvm>,
        thrown: Local<'jvm, Throwable>,
    ) -> crate::Result<'jvm, Self::Output<'jvm>> {
        Ok(thrown)
    }
}

