use std::marker::PhantomData;

use crate::{
    cast::Upcast, error::Error, java::lang::Throwable, jvm::CloneIn, IntoVoid, Jvm, JvmOp, Local,
};

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

#[derive(Clone)]
pub struct Catching<J, C> {
    op: J,
    catch: C,
}

#[derive(Clone)]
pub struct CatchNone {
    _private: (),
}

pub struct CatchSome<P, T, J> {
    prev: P,
    _thrown: PhantomData<T>,
    op: J,
}

#[derive(Clone)]
pub struct Finally<C, J> {
    catcher: C,
    op: J,
}

impl<P: Clone, T, J: Clone> Clone for CatchSome<P, T, J> {
    fn clone(&self) -> Self {
        Self {
            prev: self.prev.clone(),
            _thrown: PhantomData,
            op: self.op.clone(),
        }
    }
}

impl<J: JvmOp> Catching<J, CatchNone> {
    pub(crate) fn new(op: J) -> Self {
        Self {
            op,
            catch: CatchNone { _private: () },
        }
    }
}

impl<J: JvmOp, C> Catching<J, C> {
    /// Catch any unhandled exception thrown by this operation as long as its of
    /// type `T`. Use [`Throwable`] as `T` to catch any exception.
    pub fn catch<T, K>(self, op: impl FnOnce(ThrownOp<T>) -> K) -> Catching<J, CatchSome<C, T, K>>
    where
        T: Upcast<Throwable>,
        K: JvmOp,
        for<'jvm> K: JvmOp<Input<'jvm> = Local<'jvm, T>>,
        for<'jvm> K::Output<'jvm>: Into<J::Output<'jvm>>,
    {
        Catching {
            op: self.op,
            catch: CatchSome {
                prev: self.catch,
                _thrown: PhantomData,
                op: op(ThrownOp {
                    _marker: PhantomData,
                }),
            },
        }
    }

    /// Execute `op` regardless of the current operation succeeding, catching an exception, or "bubbling up" an
    /// unhandled exception. If `op` itself throws an exception that exception will be bubbled up instead.
    pub fn finally<K>(self, op: K) -> Finally<Self, K>
    where
        for<'jvm> K: JvmOp<Input<'jvm> = (), Output<'jvm> = ()>,
    {
        Finally { catcher: self, op }
    }
}

trait CatchArm<J: JvmOp> {
    fn try_handle<'jvm>(
        self,
        jvm: &mut Jvm<'jvm>,
        thrown: Local<'jvm, Throwable>,
    ) -> crate::Result<'jvm, Result<J::Output<'jvm>, Local<'jvm, Throwable>>>;
}

impl<J: JvmOp> CatchArm<J> for CatchNone {
    fn try_handle<'jvm>(
        self,
        _jvm: &mut Jvm<'jvm>,
        thrown: Local<'jvm, Throwable>,
    ) -> crate::Result<'jvm, Result<J::Output<'jvm>, Local<'jvm, Throwable>>> {
        Ok(Err(thrown))
    }
}

impl<J, P, T, K> CatchArm<J> for CatchSome<P, T, K>
where
    J: JvmOp,
    P: CatchArm<J>,
    T: Upcast<Throwable>,
    for<'jvm> K: JvmOp<Input<'jvm> = Local<'jvm, T>>,
    for<'jvm> K::Output<'jvm>: Into<J::Output<'jvm>>,
{
    fn try_handle<'jvm>(
        self,
        jvm: &mut Jvm<'jvm>,
        thrown: Local<'jvm, Throwable>,
    ) -> crate::Result<'jvm, Result<J::Output<'jvm>, Local<'jvm, Throwable>>> {
        match self.prev.try_handle(jvm, thrown)? {
            Ok(x) => Ok(Ok(x)),
            Err(thrown) => match thrown.try_downcast::<Throwable, T>().execute(jvm)? {
                Ok(can_catch) => Ok(Ok(self.op.execute_with(jvm, can_catch)?.into())),
                Err(other) => Ok(Err(other.clone_in(jvm))),
            },
        }
    }
}

impl<J, C: CatchArm<J>> JvmOp for Catching<J, C>
where
    J: JvmOp,
{
    type Input<'jvm> = J::Input<'jvm>;
    type Output<'jvm> = J::Output<'jvm>;

    fn execute_with<'jvm>(
        self,
        jvm: &mut Jvm<'jvm>,
        arg: Self::Input<'jvm>,
    ) -> crate::Result<'jvm, Self::Output<'jvm>> {
        self.op.execute_with(jvm, arg).or_else(|e| match e {
            Error::Thrown(thrown) => self.catch.try_handle(jvm, thrown)?.map_err(Error::Thrown),
            e => Err(e),
        })
    }
}

impl<J, K> JvmOp for Finally<J, K>
where
    J: JvmOp,
    for<'jvm> K: JvmOp<Input<'jvm> = (), Output<'jvm> = ()>,
{
    type Input<'jvm> = J::Input<'jvm>;
    type Output<'jvm> = J::Output<'jvm>;

    fn execute_with<'jvm>(
        self,
        jvm: &mut Jvm<'jvm>,
        arg: Self::Input<'jvm>,
    ) -> crate::Result<'jvm, Self::Output<'jvm>> {
        let result = self.catcher.execute_with(jvm, arg);
        if matches!(result, Ok(_) | Err(Error::Thrown(_))) {
            self.op.execute(jvm)?;
        }
        result
    }
}
