use std::marker::PhantomData;

use crate::{cast::Upcast, java::lang::Throwable, Jvm, JvmOp, ToRust};

pub struct TryCatch<This, C, E>
where
    This: JvmOp,
    C: CatchBlock<E>,
    E: std::error::Error,
{
    this: This,
    catch_block: C,
    phantom: PhantomData<E>,
}

impl<This, E> TryCatch<This, CatchNone, E>
where
    This: JvmOp,
    E: std::error::Error,
{
    pub(crate) fn new(this: This) -> Self {
        Self {
            this,
            catch_block: CatchNone { _private: () },
            phantom: PhantomData,
        }
    }
}

impl<This, C, E> TryCatch<This, C, E>
where
    This: JvmOp,
    C: CatchBlock<E>,
    E: std::error::Error,
{
    pub fn catch<J>(self) -> TryCatch<This, impl CatchBlock<E>, E>
    where
        J: Upcast<Throwable> + ToRust<Rust = E>,
    {
        let TryCatch {
            this,
            catch_block,
            phantom: _,
        } = self;
        TryCatch {
            this,
            catch_block: CatchSome {
                or_else: catch_block,
                phantom: PhantomData::<J>,
            },
            phantom: PhantomData,
        }
    }
}

impl<This, C, E> JvmOp for TryCatch<This, C, E>
where
    This: JvmOp,
    C: CatchBlock<E>,
    E: std::error::Error,
{
    type Output<'jvm> = Result<This::Output<'jvm>, E>;

    fn execute<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, Self::Output<'jvm>> {
        match self.this.execute(jvm) {
            Ok(v) => Ok(Ok(v)),
            Err(e) => match e {
                crate::Error::Thrown(t) => {
                    if let Some(e) = self.catch_block.try_catch(jvm, &t)? {
                        Ok(Err(e))
                    } else {
                        Err(crate::Error::Thrown(t))
                    }
                }
                _ => Err(e),
            },
        }
    }
}
pub trait CatchBlock<E> {
    fn try_catch<'jvm>(
        self,
        jvm: &mut Jvm<'jvm>,
        error: &Throwable,
    ) -> crate::Result<'jvm, Option<E>>;
}

pub struct CatchNone {
    _private: (),
}

impl<E> CatchBlock<E> for CatchNone
where
    E: std::error::Error,
{
    fn try_catch<'jvm>(
        self,
        _jvm: &mut Jvm<'jvm>,
        _error: &Throwable,
    ) -> crate::Result<'jvm, Option<E>> {
        Ok(None)
    }
}

pub struct CatchSome<J, C>
where
    J: Upcast<Throwable> + ToRust,
    C: CatchBlock<J::Rust>,
{
    /// Catch block to try if this one doesn't work.
    or_else: C,

    phantom: PhantomData<J>,
}

impl<J, C> CatchBlock<J::Rust> for CatchSome<J, C>
where
    J: Upcast<Throwable> + ToRust,
    C: CatchBlock<J::Rust>,
{
    fn try_catch<'jvm>(
        self,
        jvm: &mut Jvm<'jvm>,
        error: &Throwable,
    ) -> crate::Result<'jvm, Option<J::Rust>> {
        match error.try_downcast::<J>().execute(jvm)? {
            Ok(error) => {
                let rust_data = J::to_rust(&error, jvm)?;
                Ok(Some(rust_data))
            }
            Err(_) => self.or_else.try_catch(jvm, error),
        }
    }
}
