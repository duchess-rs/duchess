use std::{fmt::Debug, result};

use jni::{
    objects::{AutoLocal, JObject},
    JNIEnv,
};
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
    pub fn into_global(self, jvm: &mut Jvm<'jvm>) -> Error<Global<Throwable>> {
        match self {
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

impl<T> From<jni::errors::JniError> for Error<T> {
    fn from(value: jni::errors::JniError) -> Self {
        Self::from(JniError::from(JniErrorInternal::from(value)))
    }
}

/// Plumbing utility to wrap an operation using the [`jni`] crate [`JNIEnv`] that will check for, and materialize,
/// thrown exceptions. This should be the default way to convert [`jni::errors::Result`]s into Duchess [`Result`]s in
/// generated code.
///
/// # Motivation
///
/// Why always check for and materialize exceptions? A Duchess user can execute most [`crate::JvmOp`]s at any time
/// inside of a [`crate::Jvm::with()`] call. The returned [`Result`] should correctly contain an [`Error::Thrown`] if
/// a Java exception was thrown. If we waited to materialize errors until exiting `with` or in a `catching()` block,
/// the execute call would return an internal [`Error::Jni`] instead and the exception would be buried.
///
pub fn with_jni_env<'jvm, T>(
    env: &mut JNIEnv<'jvm>,
    f: impl FnOnce(&mut JNIEnv<'jvm>) -> jni::errors::Result<T>,
) -> Result<'jvm, T> {
    let result = f(env);
    result.map_err(|e| match e {
        jni::errors::Error::JavaException => {
            let exception = match env.exception_occurred() {
                Ok(ex) => ex,
                Err(e) => return convert_non_throw_jni_error(e),
            };
            assert!(!exception.is_null());
            if let Err(e) = env.exception_clear() {
                return convert_non_throw_jni_error(e);
            }
            Error::Thrown(unsafe {
                Local::from_jni(AutoLocal::new(JObject::from(exception), &env))
            })
        }
        error => convert_non_throw_jni_error(error),
    })
}

/// Plumbing utility to convert any [`jni::errors::Error`] that *isn't* from a thrown exception. This doesn't require
/// a [`JNIEnv`] borrow and can be used in more contexts. However, it isn't implemented as an [`Into`] to avoid masking
/// scenarios that do require an explicit exception check via [`with_jni_env()`].
///
/// # Panics
///
/// Panics if the error indicates a thrown Java exception.
pub fn convert_non_throw_jni_error<T>(error: jni::errors::Error) -> Error<T> {
    assert!(!matches!(error, jni::errors::Error::JavaException));
    Error::from(JniError::from(JniErrorInternal::from(error)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        java::{self, lang::ThrowableExt, util::ArrayListExt},
        prelude::*,
    };

    #[test]
    fn with_jni_env_materializes_thrown_exceptions() {
        Jvm::with(|jvm| {
            let env = jvm.to_env();
            // set exception state
            env.throw_new("java/lang/Throwable", "some thrown exception")
                .unwrap();
            let err = with_jni_env(env, |_env| Err::<(), _>(jni::errors::Error::JavaException))
                .unwrap_err();
            let Error::Thrown(t) = err else { panic!("expected materialized exception"); };
            assert_eq!(
                "some thrown exception",
                t.get_message().assert_not_null().into_rust(jvm).unwrap()
            );
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn with_jni_env_maps_everything_else_to_jni() {
        Jvm::with(|jvm| {
            let err = with_jni_env(jvm.to_env(), |_env| {
                Err::<(), _>(jni::errors::Error::TryLock)
            })
            .unwrap_err();
            assert!(matches!(err, Error::Jni(_)));
            Ok(())
        })
        .unwrap();
    }

    // XX this should likely move to an integration test suite
    #[test]
    fn exceptions_from_duchess_generated_types_are_materialized_without_a_catch() {
        Jvm::with(|jvm| {
            let list = java::util::ArrayList::<java::lang::Object>::new()
                .execute(jvm)
                .unwrap();
            // -2 is an illegal from index, throws
            let err = list.sub_list(-2, 0).execute(jvm).err().unwrap();
            assert!(matches!(err, Error::Thrown(_)));
            Ok(())
        })
        .unwrap();
    }
}
