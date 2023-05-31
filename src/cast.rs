use std::marker::PhantomData;

use crate::Jvm;
use crate::{
    jvm::JavaObjectExt, raw::HasEnvPtr, refs::AsJRef, JavaObject, JvmOp, Local, TryJDeref,
};

/// A trait to represent safe upcast operations for a [`JavaObject`].
///
/// # Safety
///
/// Inherits the rules of [`JavaObject`], but also `S` must be a valid superclass or implemented interface of `Self`.
/// XX: would this actually allow unsafe behavior in a JNI call? or is it already checked/enforced?
///
/// XX: having to impl `Upcast<T>` for T on each struct is pretty annoying to get `AsJRef<T>` to work without conflicts
pub unsafe trait Upcast<S: JavaObject>: JavaObject {}

pub struct TryDowncast<J, To> {
    op: J,
    _marker: PhantomData<To>,
}

impl<J: Clone, To> Clone for TryDowncast<J, To> {
    fn clone(&self) -> Self {
        Self {
            op: self.op.clone(),
            _marker: PhantomData,
        }
    }
}

impl<J, To> TryDowncast<J, To>
where
    J: JvmOp,
    for<'jvm> J::Output<'jvm>: TryJDeref,
    To: for<'jvm> Upcast<<J::Output<'jvm> as TryJDeref>::Java>,
{
    pub(crate) fn new(op: J) -> Self {
        Self {
            op,
            _marker: PhantomData,
        }
    }
}

impl<J, To> JvmOp for TryDowncast<J, To>
where
    J: JvmOp,
    for<'jvm> J::Output<'jvm>: TryJDeref,
    To: for<'jvm> Upcast<<J::Output<'jvm> as TryJDeref>::Java>,
{
    type Output<'jvm> = Result<Local<'jvm, To>, J::Output<'jvm>>;

    fn execute_with<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, Self::Output<'jvm>> {
        let instance = self.op.execute_with(jvm)?;
        let instance_raw = instance.try_jderef()?.as_raw();

        let class = To::class(jvm)?;
        let class_raw = class.as_raw();

        let env = jvm.env();
        let is_inst = unsafe {
            env.invoke(
                |env| env.IsInstanceOf,
                |env, f| f(env, instance_raw.as_ptr(), class_raw.as_ptr()),
            ) == jni_sys::JNI_TRUE
        };

        if is_inst {
            // SAFETY: just shown that jobject instanceof To::class
            let casted = unsafe { std::mem::transmute::<&_, &To>(instance.try_jderef()?) };
            Ok(Ok(jvm.local(casted)))
        } else {
            Ok(Err(instance))
        }
    }
}

pub struct AsUpcast<J, To> {
    op: J,
    _marker: PhantomData<To>,
}

impl<J: Clone, To> Clone for AsUpcast<J, To> {
    fn clone(&self) -> Self {
        Self {
            op: self.op.clone(),
            _marker: PhantomData,
        }
    }
}

impl<J, To> AsUpcast<J, To>
where
    J: JvmOp,
    for<'jvm> J::Output<'jvm>: AsJRef<To>,
    To: JavaObject,
{
    pub(crate) fn new(op: J) -> Self {
        Self {
            op,
            _marker: PhantomData,
        }
    }
}

impl<J, To> JvmOp for AsUpcast<J, To>
where
    J: JvmOp,
    for<'jvm> J::Output<'jvm>: AsJRef<To>,
    To: JavaObject,
{
    type Output<'jvm> = Local<'jvm, To>;

    fn execute_with<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, Self::Output<'jvm>> {
        let instance = self.op.execute_with(jvm)?;

        if cfg!(debug_assertions) {
            let class = To::class(jvm)?;
            let class_raw = class.as_raw();

            let instance_raw = instance.as_jref()?.as_raw();
            assert!(unsafe {
                jvm.env().invoke(
                    |env| env.IsInstanceOf,
                    |env, f| f(env, instance_raw.as_ptr(), class_raw.as_ptr()),
                ) == jni_sys::JNI_TRUE
            });
        }

        // Safety: From: Upcast<To>
        Ok(jvm.local(instance.as_jref()?))
    }
}
