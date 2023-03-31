use jni::{objects::JObject, sys, JNIEnv};
use std::{marker::PhantomData, ops::Deref};

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

    pub(crate) fn to_env(&self) -> JNIEnv<'_> {
        unsafe { JNIEnv::from_raw(self.env).unwrap() }
    }

    pub fn local<'r, R>(&self, r: &'r R) -> Local<'r, R>
    where
        R: JavaObject,
    {
        unsafe {
            let raw = r.to_raw();
            assert!(!raw.is_null());
            let internal = self.to_env().get_native_interface();
            let new_local_ref = (**internal).NewLocalRef.unwrap();
            let new_raw = new_local_ref(internal, raw);
            assert!(!new_raw.is_null());
            Local::from_jobject(new_raw)
        }
    }
}

/// Only safe to be implemented by the Java types we create.
///
/// The contract is that `X: JavaObject` is every `&X` is guaranteed
/// to be a JVM (local || global) reference in the currently active JVM.
pub unsafe trait JavaObject {
    fn to_raw(&self) -> sys::jobject {
        self as *const _ as sys::jobject
    }
}

pub(crate) struct Anchor<'jvm> {
    object: JObject<'jvm>,
}

impl<'jvm> Anchor<'jvm> {
    pub fn from(r: &'jvm impl JavaObject) -> Self {
        unsafe {
            Anchor {
                object: JObject::from_raw(r.to_raw()),
            }
        }
    }
}

impl<'jvm> AsRef<JObject<'jvm>> for Anchor<'jvm> {
    fn as_ref(&self) -> &JObject<'jvm> {
        &self.object
    }
}

impl<'jvm> Deref for Anchor<'jvm> {
    type Target = JObject<'jvm>;

    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

/// Indicates a local ref to a JVM object.
/// When this is dropped, the local ref is reclaimed.
/// There are only a limited number of local refs, so this can be important.
pub struct Local<'jvm, R>
where
    R: JavaObject,
{
    data: *mut R,
    phantom: PhantomData<&'jvm R>,
}

impl<'jvm, R> Local<'jvm, R>
where
    R: JavaObject,
{
    /// Unsafety conditions:
    ///
    /// * jobject must be an instance of `R`
    pub(crate) unsafe fn from_jobject(jobject: impl IntoRawJObject) -> Self {
        let jobject = jobject.into_raw();
        Local {
            data: jobject as *mut R,
            phantom: PhantomData,
        }
    }

    pub fn into_global(self) -> Global<R> {
        todo!()
    }

    fn to_raw(&self) -> sys::jobject {
        self.data as sys::jobject
    }

    unsafe fn to_jobject(&self) -> JObject<'_> {
        JObject::from_raw(self.to_raw())
    }
}

impl<R> Drop for Local<'_, R>
where
    R: JavaObject,
{
    fn drop(&mut self) {
        unsafe {
            let jvm = Jvm::get();
            jvm.to_env().delete_local_ref(self.to_jobject()).unwrap();
        }
    }
}

impl<R> Deref for Local<'_, R>
where
    R: JavaObject,
{
    type Target = R;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

/// Indicates a **global** ref to a JVM object.
/// When this is dropped, the global ref is reclaimed.
pub struct Global<R>
where
    R: JavaObject,
{
    data: *mut R,
    phantom: PhantomData<R>,
}

impl<R> Global<R>
where
    R: JavaObject,
{
    fn to_raw(&self) -> sys::jobject {
        self.data as sys::jobject
    }
}

impl<R> Deref for Global<R>
where
    R: JavaObject,
{
    type Target = R;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

impl<R> Drop for Global<R>
where
    R: JavaObject,
{
    fn drop(&mut self) {
        unsafe {
            let jvm = Jvm::get();
            let internal = jvm.to_env().get_native_interface();
            let delete_global_ref = (**internal).DeleteGlobalRef.unwrap();
            delete_global_ref(internal, self.to_raw());
        }
    }
}

pub(crate) trait IntoRawJObject {
    fn into_raw(self) -> sys::jobject;
}

impl IntoRawJObject for JObject<'_> {
    fn into_raw(self) -> sys::jobject {
        self.into_raw()
    }
}

impl IntoRawJObject for sys::jobject {
    fn into_raw(self) -> sys::jobject {
        self
    }
}

impl<R> IntoRawJObject for &R
where
    R: JavaObject,
{
    fn into_raw(self) -> sys::jobject {
        self.to_raw()
    }
}
