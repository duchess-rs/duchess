use std::{marker::PhantomData, ops::Deref, ptr::NonNull};

use crate::jvm::JavaObjectExt;
use crate::thread;
use crate::{cast::Upcast, jvm::CloneIn, plumbing::ObjectPtr, raw::EnvPtr, JavaObject, Jvm};

/// An owned local reference to a non-null Java object of type `T`. The reference will be freed when
/// dropped. Cannot be shared across threads or [`Jvm::with`] invocations.
#[derive_where::derive_where(PartialEq, Eq, Hash, Debug)]
pub struct Local<'jvm, T: JavaObject> {
    env: EnvPtr<'jvm>,
    obj: ObjectPtr,
    _marker: PhantomData<T>,
}

impl<'jvm, T: JavaObject> Local<'jvm, T> {
    /// Convert an existing local reference pointed to by `obj` into an owned `Local`. This is used by
    /// codegen to wrap the output of most JNI calls to prevent the user from not freeing local refs.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `obj` points to a Java object that is an instance of `T` (or its subclasses), is a
    /// a live, local reference in the current frame, will not later be deleted (including through another call to
    /// `from_raw()`), and will not dereferenced after the returned [`Local`] is dropped.
    #[doc(hidden)]
    pub unsafe fn from_raw(env: EnvPtr<'jvm>, obj: ObjectPtr) -> Self {
        Self {
            obj,
            env,
            _marker: PhantomData,
        }
    }

    /// Creates a *new* local reference to `obj` in the current frame via a `NewLocalRef` JNI call.
    pub(crate) fn new(env: EnvPtr<'jvm>, obj: &T) -> Self {
        // SAFETY: The JavaObject trait contract ensures that &T points to a Java object that is an instance of T.
        unsafe {
            let new_ref = env.invoke_unchecked(
                |jni| jni.NewLocalRef,
                |jni, f| f(jni, obj.as_raw().as_ptr()),
            );
            Self::from_raw(env, NonNull::new(new_ref).unwrap().into())
        }
    }

    /// Convert this `Local` into a raw object pointer *without* running the Local destructor (which would release it from the JVM).
    ///
    /// # Safety
    ///
    /// Caller must ensure that this pointer
    /// does not escape the `'jvm` scope.
    pub unsafe fn into_raw(self) -> ObjectPtr {
        let p = self.as_raw();
        std::mem::forget(self);
        p
    }
}

impl<T: JavaObject> Drop for Local<'_, T> {
    fn drop(&mut self) {
        // SAFETY: Local owns the local ref and it's no longer possible to dereference the object pointer.
        unsafe {
            self.env
                .invoke_unchecked(|jni| jni.DeleteLocalRef, |jni, f| f(jni, self.obj.as_ptr()));
        }
    }
}

impl<T: JavaObject> Deref for Local<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: from the guarantees of Local::from_raw, we know obj is a live local ref to an instance of T
        unsafe { T::from_raw(self.obj) }
    }
}

/// An owned global reference to a non-null Java object of type `T`. The reference will be freed when dropped.
#[derive_where::derive_where(PartialEq, Eq, Hash)]
pub struct Global<T: JavaObject> {
    obj: ObjectPtr,
    _marker: PhantomData<T>,
}

impl<T: JavaObject> Global<T> {
    /// Convert an existing global reference pointed to by `obj` into an owned `Global`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `obj` points to a Java object that is an instance of `T` (or its subclasses), is a
    /// a live, global reference, will not later be deleted (including through another call to `from_raw()`), and will
    /// not dereferenced after the returned [`Global`] is dropped.
    pub(crate) unsafe fn from_raw(obj: ObjectPtr) -> Self {
        Self {
            obj,
            _marker: PhantomData,
        }
    }

    /// Creates a *new* global reference to `obj` in the current frame via a `NewGlobalRef` JNI call.
    pub(crate) fn new(env: EnvPtr<'_>, obj: &T) -> Self {
        // SAFETY: The JavaObject trait contract ensures that &T points to a Java object that is an instance of T.
        unsafe {
            let new_ref =
                env.invoke_unchecked(|e| e.NewGlobalRef, |e, f| f(e, obj.as_raw().as_ptr()));
            Self::from_raw(NonNull::new(new_ref).unwrap().into())
        }
    }
}

impl<T: JavaObject> Drop for Global<T> {
    fn drop(&mut self) {
        let jvm = crate::jvm::unwrap_global_jvm();

        // SAFETY: Global owns the global ref and it's no longer possible to dereference the object pointer.
        let delete = |env: EnvPtr<'_>| unsafe {
            env.invoke_unchecked(
                |jni| jni.DeleteGlobalRef,
                |jni, f| f(jni, self.obj.as_ptr()),
            )
        };

        match unsafe { jvm.env() } {
            Ok(Some(env)) => delete(env),
            Ok(None) => {
                // SAFETY: jvm is a valid pointer since duchess will not deinitialize a JVM once created
                match unsafe { thread::attach(jvm) } {
                    Ok(mut attached) => delete(attached.env()),
                    Err(err) => {
                        tracing::warn!(?err, "unable to attach current thread to delete global ref")
                    }
                }
            }
            Err(err) => tracing::warn!(
                ?err,
                "unable to get JNI interface for local thread to delete global ref"
            ),
        }
    }
}

// SAFETY: The JNI promises only global refs are shareable across threads
unsafe impl<T: JavaObject> Send for Global<T> {}
unsafe impl<T: JavaObject> Sync for Global<T> {}

impl<T: JavaObject> Deref for Global<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: from the guarantees of Global::from_raw, we know obj is a live global ref to an instance of T
        unsafe { T::from_raw(self.obj) }
    }
}

impl<'a, R, S> AsRef<S> for Local<'a, R>
where
    R: Upcast<S>,
    S: JavaObject + 'a,
{
    fn as_ref(&self) -> &S {
        // SAFETY: From the Upcast trait contract, we know R is also an instance of S
        unsafe { S::from_raw(self.obj) }
    }
}

impl<'a, R: JavaObject> Local<'a, R> {
    pub fn upcast<S>(self) -> Local<'a, S>
    where
        R: Upcast<S>,
        S: JavaObject + 'a,
    {
        // SAFETY: From the Upcast trait contract, we know R is also an instance of S
        let upcast = unsafe { Local::<S>::from_raw(self.env, self.obj) };
        upcast
    }
}

impl<R, S> AsRef<S> for Global<R>
where
    R: Upcast<S>,
    S: JavaObject + 'static,
{
    fn as_ref(&self) -> &S {
        // SAFETY: From the Upcast trait contract, we know R is also an instance of S
        unsafe { S::from_raw(self.obj) }
    }
}

impl<R: JavaObject> Global<R> {
    pub fn upcast<S>(self) -> Global<S>
    where
        R: Upcast<S>,
        S: JavaObject + 'static,
    {
        // SAFETY: From the Upcast trait contract, we know R is also an instance of S
        let upcast = unsafe { Global::<S>::from_raw(self.obj) };
        upcast
    }
}

impl<'jvm, T> CloneIn<'jvm> for Local<'jvm, T>
where
    T: JavaObject,
{
    fn clone_in(&self, jvm: &mut Jvm<'jvm>) -> Self {
        jvm.local(self)
    }
}

impl<'jvm, T> CloneIn<'jvm> for Global<T>
where
    T: JavaObject,
{
    fn clone_in(&self, jvm: &mut Jvm<'jvm>) -> Self {
        jvm.global(self)
    }
}
