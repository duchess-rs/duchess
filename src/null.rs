use std::marker::PhantomData;

use crate::{AsJRef, JavaObject, NullJRef, TryJDeref};

#[derive(Copy, Clone)]
pub struct Null;

impl<J: JavaObject> AsJRef<J> for Null {
    fn as_jref(&self) -> crate::Nullable<&J> {
        Err(NullJRef)
    }
}
