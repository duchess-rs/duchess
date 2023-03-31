use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use jni::{
    objects::{AutoLocal, GlobalRef, JObject},
    sys, JNIEnv,
};

pub trait JdkOp {
    type Output<'jvm>;

    fn execute<'jvm>(self, jvm: &'jvm Jvm) -> jni::errors::Result<Self::Output<'jvm>>;
}

#[repr(transparent)]
pub struct Jvm {
    env: *mut sys::JNIEnv,
    data: PhantomData<*mut ()>, // Disable send, sync, etc
}

thread_local! {
    static JVM_REF: Option<Jvm> = None
}

impl Jvm {
    pub fn with<R>(f: impl FnOnce(&Jvm) -> R) -> R {
        unsafe { f(&Jvm::get()) }
    }

    unsafe fn get() -> Jvm {
        JVM_REF.with(|r| match r {
            &Some(Jvm { env, data }) => Jvm { env, data },
            None => panic!("not form an attached thread, this shouldn't happen"),
        })
    }

    pub fn to_env(&self) -> JNIEnv<'_> {
        unsafe { JNIEnv::from_raw(self.env).unwrap() }
    }

    pub fn local<'r, R>(&'r self, r: &'r R) -> Local<'r, R>
    where
        R: JavaObject,
    {
        let env = self.to_env();
        unsafe {
            let raw = r.as_jobject();
            assert!(!raw.is_null());
            let new_local_ref = env.new_local_ref(&*raw).unwrap();
            assert!(!new_local_ref.is_null());
            Local::from_jni(AutoLocal::new(new_local_ref, &env))
        }
    }
}

/// A trait for zero-sized dummy types that represent Java object types.
///
/// # Safety
///
/// A type `T` that implements this trait must satisfy the following contract:
///
/// 1. `T` must be a zero-sized type.
/// 2. It must not be possible to construct a value of type `T`.
/// 3. The alignment of `T` must *not* be greater than the alignment of [jni::sys::_jobject]. (I
///    *think* this is always true for zero-sized types, so would be implied by rule #1, but I'm not
///    sure.)
///
/// # Example
///
/// ```
/// # use duchess::jvm::JavaObject;
/// pub struct BigDecimal {
///     _private: (), // prevent construction
/// }
/// unsafe impl JavaObject for BigDecimal {}
/// ```
pub unsafe trait JavaObject: Sized {}

/// Extension trait for [JavaObject].
pub trait JavaObjectExt {
    // We use an extension trait, instead of just declaring these functions on the main JavaObject
    // trait, to prevent trait implementors from overriding the implementation of these functions.

    fn from_jobject<'a>(obj: &'a JObject<'a>) -> Option<&'a Self>;
    fn as_jobject(&self) -> BorrowedJObject<'_>;
}
impl<T: JavaObject> JavaObjectExt for T {
    fn from_jobject<'a>(obj: &'a JObject<'a>) -> Option<&'a Self> {
        // SAFETY: I *think* the cast is sound, because:
        //
        // 1. A pointer to a suitably aligned `sys::_jobject` should also satisfy Self's alignment
        //    requirement (trait rule #3)
        // 2. Self is a zero-sized type (trait rule #1), so there are no invalid bit patterns to
        //    worry about.
        // 3. Self is a zero-sized type (trait rule #1), so there's no actual memory region that is
        //    subject to the aliasing rules.
        //
        // XXX: Please check my homework.
        Some(unsafe { NonNull::new(obj.as_raw())?.cast().as_ref() })
    }

    fn as_jobject(&self) -> BorrowedJObject<'_> {
        let raw = (self as *const Self).cast_mut().cast::<sys::_jobject>();

        // SAFETY: the only way to get a `&Self` is by calling `Self::from_jobject` (trait rule #1),
        // so reconstructing the original JObject passed to `from_jni` should also be safe.
        let obj = unsafe { JObject::from_raw(raw) };

        // We must wrap the JObject to prevent anyone from calling `delete_local_ref` on it;
        // otherwise, `self` could become dangling
        BorrowedJObject::new(obj)
    }
}

/// A wrapper for a [JObject] that only allows access by reference. This prevents passing the
/// wrapped `JObject` to `JNIEnv::delete_local_ref`.
pub type BorrowedJObject<'a> = Jail<JObject<'a>>;

/// A wrapper for a value that prevents the value from being moved out, while still allowing access
/// by reference.
pub struct Jail<T>(T);

impl<T> Jail<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }
}
impl<T> Deref for Jail<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> DerefMut for Jail<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<T> AsRef<T> for Jail<T> {
    fn as_ref(&self) -> &T {
        &*self
    }
}
impl<T> AsMut<T> for Jail<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut *self
    }
}

// I suspect we'll need to change these from type aliases to newtypes, for ergonomics sake, but for
// now, they just work.

/// An owned local reference to a non-null Java object of type `T`. The reference will be freed when
/// dropped.
pub type Local<'a, T> = OwnedRef<'a, AutoLocal<'a, JObject<'a>>, T>;

/// An owned global reference to a non-null Java object of type `T`. The reference will be freed
/// when dropped.
pub type Global<T> = OwnedRef<'static, GlobalRef, T>;

/// An *owned* JNI reference, either local or global, to a non-null Java object of type `T`. The
/// underlying JNI reference is represented by type `J`, which is responsible for freeing the
/// reference when dropped.
///
/// Typically, instead of using `OwnedRef` directly, you would use one of the type aliases, [Local]
/// or [Global]. If you need a borrowed reference instead of an owned one, just use `&T`.
pub struct OwnedRef<'a, J, T> {
    inner: J,
    phantom: PhantomData<&'a T>,
}

impl<J, T> OwnedRef<'_, J, T> {
    /// Converts an underlying JNI reference into an [OwnedRef].
    ///
    /// # Safety
    ///
    /// `inner` must refer to a non-null Java object whose type is `T`.
    pub unsafe fn from_jni(inner: J) -> Self {
        Self {
            inner,
            phantom: PhantomData,
        }
    }
}

impl<'a, J, T> Deref for OwnedRef<'a, J, T>
where
    J: Deref<Target = JObject<'a>>,
    T: JavaObject,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        T::from_jobject(&*self.inner).expect("inner reference is null")
    }
}

// pub(crate) trait IntoRawJObject {
//     fn into_raw(self) -> sys::jobject;
// }

// impl IntoRawJObject for JObject<'_> {
//     fn into_raw(self) -> sys::jobject {
//         self.into_raw()
//     }
// }

// impl IntoRawJObject for sys::jobject {
//     fn into_raw(self) -> sys::jobject {
//         self
//     }
// }

// impl<R> IntoRawJObject for &R
// where
//     R: JavaObject,
// {
//     fn into_raw(self) -> sys::jobject {
//         self.to_raw()
//     }
// }
