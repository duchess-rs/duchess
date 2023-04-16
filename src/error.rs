use std::{fmt::Debug, result};

use jni::objects::{AutoLocal, JObject};
use thiserror::Error;

use crate::{java::lang::Throwable, Global, Jvm, Local};

/// Result returned by most Java operations that may contain a local reference
/// to a thrown exception.
pub type Result<'jvm, T> = result::Result<T, Error<Local<'jvm, Throwable>>>;

/// Result returned by [`crate::Jvm::with()`] that will store any uncaught
/// exception as a global reference.
pub type GlobalResult<T> = result::Result<T, Error<Global<Throwable>>>;

#[derive(Error)]
pub enum Error<T> {
    /// A reference to an uncaught Java exception
    #[error("Java invocation threw")]
    Thrown(T),

    /// An internal JNI error occurred
    #[error(transparent)]
    Jni(#[from] JniError),
}

impl<T> Debug for Error<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Thrown(_t) => f.debug_tuple("Thrown").finish(),
            Self::Jni(e) => e.fmt(f),
        }
    }
}

impl<'jvm> Error<Local<'jvm, Throwable>> {
    /// Many checked functions from the [`jni`] crate will check if an exception
    /// has been thrown and return a [`jni::errors::Error::JavaException`]
    /// instead. This is just a flag, so we need to use a JNI method to extract
    /// the Java object that was thrown.
    pub(crate) fn extract_thrown(self, jvm: &mut Jvm<'jvm>) -> Self {
        match &self {
            Self::Jni(JniError(JniErrorInternal::CheckFailure(
                jni::errors::Error::JavaException,
            ))) => {
                let env = jvm.to_env();
                let exception = match env.exception_occurred() {
                    Ok(e) => e,
                    Err(e) => return e.into(),
                };
                assert!(!exception.is_null());
                if let Err(e) = env.exception_clear() {
                    return e.into();
                }
                Self::Thrown(unsafe {
                    Local::from_jni(AutoLocal::new(JObject::from(exception), &env))
                })
            }
            _ => self,
        }
    }

    pub fn into_global(self, jvm: &mut Jvm<'jvm>) -> Error<Global<Throwable>> {
        match self.extract_thrown(jvm) {
            Error::Thrown(t) => Error::Thrown(jvm.global(&t)),
            Error::Jni(e) => Error::Jni(e),
        }
    }
}

/// An error ocurred invoking the JNI bridge.
///
/// XX: can we say that is either a duchess bug or a mismatch between the Java
/// interface the rust code was compiled with and what was run? What other cases
/// are there?
#[derive(Error, Debug)]
#[error(transparent)]
pub struct JniError(#[from] pub(crate) JniErrorInternal);

#[derive(Error, Debug)]
pub(crate) enum JniErrorInternal {
    #[error(transparent)]
    CheckFailure(#[from] jni::errors::Error),
    #[error(transparent)]
    Jni(#[from] jni::errors::JniError),
}

impl<T> From<jni::errors::Error> for Error<T> {
    fn from(value: jni::errors::Error) -> Self {
        Self::from(JniError::from(JniErrorInternal::from(value)))
    }
}

impl<T> From<jni::errors::JniError> for Error<T> {
    fn from(value: jni::errors::JniError) -> Self {
        Self::from(JniError::from(JniErrorInternal::from(value)))
    }
}
