use crate::{cast::Upcast, java::lang::Throwable, Error, Global, JavaObject, Local};

/// Possibly null reference to a Java object.
pub trait AsJRef<U> {
    fn as_jref(&self) -> Result<&U, NullJRef>;
}

pub struct NullJRef;

impl<'jvm, T, U> AsJRef<U> for T
where
    T: BaseJRef,
    T::Java: Upcast<U>,
    U: JavaObject,
{
    fn as_jref(&self) -> Result<&U, NullJRef> {
        let this = self.base_jref()?;
        Ok(unsafe { std::mem::transmute(this) })
    }
}

/// Possibly null reference to a Java object.
pub trait BaseJRef {
    type Java: JavaObject;
    fn base_jref(&self) -> Result<&Self::Java, NullJRef>;
}

impl<T> BaseJRef for &T
where
    T: BaseJRef,
{
    type Java = T::Java;

    fn base_jref(&self) -> Result<&T::Java, NullJRef> {
        T::base_jref(self)
    }
}

impl<T> BaseJRef for Local<'_, T>
where
    T: JavaObject,
{
    type Java = T;

    fn base_jref(&self) -> Result<&T, NullJRef> {
        Ok(self)
    }
}

impl<T> BaseJRef for Global<T>
where
    T: JavaObject,
{
    type Java = T;

    fn base_jref(&self) -> Result<&T, NullJRef> {
        Ok(self)
    }
}

impl<T> From<NullJRef> for Error<T>
where
    T: AsJRef<Throwable>,
{
    fn from(NullJRef: NullJRef) -> Self {
        Error::NullDeref
    }
}
