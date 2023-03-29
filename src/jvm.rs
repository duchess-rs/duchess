use jni::sys::JNINativeInterface_;
use std::{marker::PhantomData, ops::Deref};

#[repr(transparent)]
pub struct Jvm {
    env: *mut *const JNINativeInterface_,
    data: PhantomData<*mut ()>, // Disable send, sync, etc
}

thread_local! {
    static JVM_REF: Option<Jvm> = None
}

impl Jvm {
    pub fn with(f: impl FnOnce(&Jvm)) {
        todo!()
    }

    unsafe fn get() -> Jvm {
        JVM_REF.with(|r| match r {
            &Some(Jvm { env, data }) => Jvm { env, data },
            None => panic!("not form an attached thread, this shouldn't happen"),
        })
    }

    unsafe fn delete_local_ref(&self, r: &impl JavaObject) {
        let delete_local_ref = (**self.env).DeleteLocalRef.unwrap();
        delete_local_ref(self.env, to_jobject(r))
    }
}

/// Only safe to be implemented by the Java types we create.
///
/// The contract is that `X: JavaObject` is every `&X` is guaranteed
/// to be a JVM local reference in the currently active JVM.
pub unsafe trait JavaObject {}

fn to_jobject(r: &impl JavaObject) -> *mut jni::sys::_jobject {
    r as *const _ as *mut jni::sys::_jobject
}

/// Indicates an owned Java object.
pub struct J<'jvm, R>
where
    R: JavaObject,
{
    data: *mut R,
    phantom: PhantomData<&'jvm R>,
}

impl<R> Drop for J<'_, R>
where
    R: JavaObject,
{
    fn drop(&mut self) {
        unsafe { Jvm::get().delete_local_ref(&*self.data) }
    }
}

impl<R> Deref for J<'_, R>
where
    R: JavaObject,
{
    type Target = R;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}
