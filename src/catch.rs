use std::marker::PhantomData;

use jni::{
    objects::{AutoLocal, JObject},
    JNIEnv,
};

use crate::{cast::Upcast, error::Error, java::lang::Throwable, JvmOp, Local};

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
        Err(crate::Error::Thrown(unsafe {
            Local::from_jni(AutoLocal::new(obj, &env))
        }))
    }
}

#[derive(Clone)]
pub struct Catch<J, K> {
    op: J,
    catch: K,
}

impl<J, T, K> Catch<J, K>
where
    J: JvmOp,
    T: Upcast<Throwable>,
    K: JvmOp,
    for<'jvm> K: JvmOp<Input<'jvm> = Local<'jvm, T>>,
    for<'jvm> K::Output<'jvm>: Into<J::Output<'jvm>>,
{
    pub(crate) fn new(op: J, catch: impl FnOnce(ThrownOp<T>) -> K) -> Catch<J, K> {
        let catch = catch(ThrownOp {
            _marker: PhantomData,
        });
        Catch { op, catch }
    }
}

impl<J, T, F> JvmOp for Catch<J, F>
where
    J: JvmOp,
    T: Upcast<Throwable>,
    F: JvmOp,
    for<'jvm> F: JvmOp<Input<'jvm> = Local<'jvm, T>>,
    for<'jvm> F::Output<'jvm>: Into<J::Output<'jvm>>,
{
    type Input<'jvm> = J::Input<'jvm>;
    type Output<'jvm> = J::Output<'jvm>;

    fn execute_with<'jvm>(
        self,
        jvm: &mut crate::Jvm<'jvm>,
        arg: Self::Input<'jvm>,
    ) -> crate::Result<'jvm, Self::Output<'jvm>> {
        self.op.execute_with(jvm, arg).or_else(|e| {
            let e = e.extract_thrown(jvm);
            if let Error::Thrown(thrown) = &e {
                if let Ok(caught) = thrown.try_downcast::<_, T>().execute(jvm)? {
                    return self.catch.execute_with(jvm, caught).map(|v| v.into());
                }
            }
            Err(e)
        })
    }
}

#[derive(Clone)]
pub struct ThrownOp<T> {
    _marker: PhantomData<T>,
}

impl<T: Upcast<Throwable>> JvmOp for ThrownOp<T> {
    type Input<'jvm> = Local<'jvm, T>;
    type Output<'jvm> = Local<'jvm, T>;

    fn execute_with<'jvm>(
        self,
        _jvm: &mut crate::Jvm<'jvm>,
        thrown: Local<'jvm, T>,
    ) -> crate::Result<'jvm, Self::Output<'jvm>> {
        Ok(thrown)
    }
}
