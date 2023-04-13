use std::{result, fmt::Debug};

use jni::objects::{AutoLocal, JObject};
use thiserror::Error;

use crate::{Local, Global, Jvm, java::lang::Throwable};

pub type Result<'jvm, T> = result::Result<T, Error<Local<'jvm, Throwable>>>; 
pub type GlobalResult<T> = result::Result<T, Error<Global<Throwable>>>; 

#[derive(Error)]
pub enum Error<T> {
    #[error("Java invocation threw")]
    Thrown(T),

    #[error(transparent)]
    Jni(#[from] JniError),
}

impl<T> Debug for Error<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Thrown(_t) => f.debug_tuple("Thrown").finish(),
            Self::Jni(e) => e.fmt(f)
        }
    }
}

impl<'jvm> Error<Local<'jvm, Throwable>> {
    pub(crate) fn extract_thrown(self, jvm: &mut Jvm<'jvm>) -> Self {
        match &self {
            Self::Jni(JniError(JniErrorInternal::CheckFailure(jni::errors::Error::JavaException))) => {
                let env = jvm.to_env();
                let exception = match env.exception_occurred() {
                    Ok(e) => e,
                    Err(e) => return e.into(),
                };
                assert!(!exception.is_null());
                if let Err(e) = env.exception_clear() {
                    return e.into();
                }
                Self::Thrown(unsafe { Local::from_jni(AutoLocal::new(JObject::from(exception), &env)) })
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

#[derive(Error, Debug)]
#[error(transparent)]
pub struct JniError(#[from] pub(crate) JniErrorInternal);

#[derive(Error, Debug)]
pub(crate) enum JniErrorInternal {
    #[error(transparent)]
    CheckFailure(#[from] jni::errors::Error),
    #[error(transparent)]
    Jni(#[from] jni::errors::JniError)
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
