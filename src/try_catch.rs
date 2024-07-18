use std::marker::PhantomData;

use crate::{cast::Upcast, java::lang::Throwable, Jvm, JvmOp, Local};

#[derive_where::derive_where(Clone)]
#[derive_where(Copy; This: Copy)]
pub struct TryCatch<This, J>
where
    This: JvmOp,
    J: Upcast<Throwable>,
{
    this: This,
    phantom: PhantomData<J>,
}

impl<This, J> TryCatch<This, J>
where
    This: JvmOp,
    J: Upcast<Throwable>,
{
    pub(crate) fn new(this: This) -> Self {
        Self {
            this,
            phantom: PhantomData,
        }
    }
}

impl<This, J> JvmOp for TryCatch<This, J>
where
    This: JvmOp,
    J: Upcast<Throwable>,
{
    type Output<'jvm> = Result<This::Output<'jvm>, Local<'jvm, J>>;

    fn do_jni<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::LocalResult<'jvm, Self::Output<'jvm>> {
        match self.this.do_jni(jvm) {
            Ok(v) => Ok(Ok(v)),
            Err(e) => match e {
                crate::Error::Thrown(exception) => {
                    if let Ok(exception) = exception.try_downcast::<J>().do_jni(jvm)? {
                        Ok(Err(exception))
                    } else {
                        Err(crate::Error::Thrown(exception))
                    }
                }
                _ => Err(e),
            },
        }
    }
}
