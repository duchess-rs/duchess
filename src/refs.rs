use crate::{cast::Upcast, java::lang::Throwable, Error, Global, JavaObject, Local};

/// Possibly null reference to a Java object.
pub trait AsJRef<U> {
    fn as_jref(&self) -> Nullable<&U>;
}

/// Marker type used to indicate an attempt to dereference a null java reference.
/// See [`TryJDeref`][] trait.
pub struct NullJRef;

pub type Nullable<T> = Result<T, NullJRef>;

impl<'jvm, T, U> AsJRef<U> for T
where
    T: TryJDeref,
    T::Java: Upcast<U>,
    U: JavaObject,
{
    fn as_jref(&self) -> Nullable<&U> {
        let this = self.try_jderef()?;
        Ok(unsafe { std::mem::transmute(this) })
    }
}

/// Reference to a Java object that may or may not be null.
/// Implemented both by non-null references like `Global<java::lang::Object>`
/// or `&java::lang::Object` and by maybe-null references like `Option<Global<java::lang::Object>>`.
pub trait TryJDeref {
    /// The Java type (e.g., [`java::lang::Object`][`crate::java::lang::Object`]).
    type Java: JavaObject;

    /// Dereference to a plain reference to the java object, or `Err` if it is null.
    fn try_jderef(&self) -> Nullable<&Self::Java>;
}

/// Reference to a Java object that cannot be null (e.g., `Global<java::lang::Object>`).
pub trait JDeref: TryJDeref {
    /// Dereference to a plain reference to the java object.
    fn jderef(&self) -> &Self::Java;
}

impl<T> TryJDeref for &T
where
    T: TryJDeref,
{
    type Java = T::Java;

    fn try_jderef(&self) -> Nullable<&T::Java> {
        T::try_jderef(self)
    }
}

impl<T> JDeref for &T
where
    T: JDeref,
{
    fn jderef(&self) -> &T::Java {
        T::jderef(self)
    }
}

impl<T> TryJDeref for Local<'_, T>
where
    T: JavaObject,
{
    type Java = T;

    fn try_jderef(&self) -> Nullable<&T> {
        Ok(self)
    }
}

impl<T> JDeref for Local<'_, T>
where
    T: JavaObject,
{
    fn jderef(&self) -> &T {
        self
    }
}

impl<T> TryJDeref for Global<T>
where
    T: JavaObject,
{
    type Java = T;

    fn try_jderef(&self) -> Nullable<&T> {
        Ok(self)
    }
}

impl<T> JDeref for Global<T>
where
    T: JavaObject,
{
    fn jderef(&self) -> &T {
        self
    }
}

impl<T> TryJDeref for Option<T>
where
    T: TryJDeref,
{
    type Java = T::Java;

    fn try_jderef(&self) -> Result<&T::Java, NullJRef> {
        match self {
            Some(r) => r.try_jderef(),
            None => Err(NullJRef),
        }
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
