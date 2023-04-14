use std::marker::PhantomData;

use jni::objects::{JClass, JObject};

use crate::{
    jvm::{Jail, JavaObjectExt},
    JavaObject, Jvm, JvmOp, Local,
};

/// A trait to represent safe upcast operations for a [`JavaObject`].
///
/// # Safety
///
/// Inherits the rules of [`JavaObject`], but also `S` must be a valid superclass or implemented interface of `Self`.
/// XX: would this actually allow unsafe behavior in a JNI call? or is it already checked/enforced?
///
/// XX: having to impl Upcast<T> for T on each struct is pretty annoying to get AsRef<T> to work without conflicts
pub unsafe trait Upcast<S: JavaObject>: JavaObject {}

pub struct TryDowncast<J, From, To> {
    op: J,
    _marker: PhantomData<(From, To)>,
}

impl<J: Clone, From, To> Clone for TryDowncast<J, From, To> {
    fn clone(&self) -> Self {
        Self {
            op: self.op.clone(),
            _marker: PhantomData,
        }
    }
}

impl<J, From, To> TryDowncast<J, From, To>
where
    J: JvmOp,
    for<'jvm> J::Output<'jvm>: AsRef<From>,
    From: JavaObject,
    To: Upcast<From>,
{
    pub(crate) fn new(op: J) -> Self {
        Self {
            op,
            _marker: PhantomData,
        }
    }

    pub fn execute<'jvm>(
        self,
        jvm: &mut Jvm<'jvm>,
    ) -> crate::Result<'jvm, Result<Local<'jvm, To>, J::Output<'jvm>>>
    where
        J: JvmOp<Input<'jvm> = ()>,
    {
        self.execute_with(jvm, ())
    }
}

impl<J, From, To> JvmOp for TryDowncast<J, From, To>
where
    J: JvmOp,
    for<'jvm> J::Output<'jvm>: AsRef<From>,
    From: JavaObject,
    To: Upcast<From>,
{
    type Input<'jvm> = J::Input<'jvm>;
    type Output<'jvm> = Result<Local<'jvm, To>, J::Output<'jvm>>;

    fn execute_with<'jvm>(
        self,
        jvm: &mut crate::Jvm<'jvm>,
        input: J::Input<'jvm>,
    ) -> crate::Result<'jvm, Self::Output<'jvm>> {
        let instance = self.op.execute_with(jvm, input)?;
        let jobject = instance.as_ref().as_jobject();

        let class = To::class(jvm)?;
        let jclass = class.as_jobject();
        // Safety: XX class is a Local<Class> which can only point to Java class objects
        let jclass = unsafe { std::mem::transmute::<Jail<JObject>, Jail<JClass>>(jclass) };

        let env = jvm.to_env();
        if !env.is_instance_of(&jobject, &*jclass)? {
            return Ok(Err(instance));
        }

        let local = jvm.local(instance.as_ref());
        // Safety: XX repr(transparent) + just checked that instance is an instance of To
        let casted = unsafe { std::mem::transmute::<Local<From>, Local<To>>(local) };
        Ok(Ok(casted))
    }
}

pub struct AsUpcast<J, From, To> {
    op: J,
    _marker: PhantomData<(From, To)>,
}

impl<J: Clone, From, To> Clone for AsUpcast<J, From, To> {
    fn clone(&self) -> Self {
        Self {
            op: self.op.clone(),
            _marker: PhantomData,
        }
    }
}

impl<J, From, To> AsUpcast<J, From, To>
where
    J: JvmOp,
    for<'jvm> J::Output<'jvm>: AsRef<From>,
    From: Upcast<To>,
    To: JavaObject,
{
    pub(crate) fn new(op: J) -> Self {
        Self {
            op,
            _marker: PhantomData,
        }
    }

    pub fn execute<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, Local<'jvm, To>>
    where
        J: JvmOp<Input<'jvm> = ()>,
    {
        self.execute_with(jvm, ())
    }
}

impl<J, From, To> JvmOp for AsUpcast<J, From, To>
where
    J: JvmOp,
    for<'jvm> J::Output<'jvm>: AsRef<From>,
    From: Upcast<To>,
    To: JavaObject,
{
    type Input<'jvm> = J::Input<'jvm>;
    type Output<'jvm> = Local<'jvm, To>;

    fn execute_with<'jvm>(
        self,
        jvm: &mut crate::Jvm<'jvm>,
        input: J::Input<'jvm>,
    ) -> crate::Result<'jvm, Self::Output<'jvm>> {
        let instance = self.op.execute_with(jvm, input)?;
        let jobject = instance.as_ref().as_jobject();

        if cfg!(debug_assertions) {
            // XX: safety, find ways to avoid repeating?
            let to_jclass =
                unsafe { JClass::from_raw(To::class(jvm)?.as_ref().as_jobject().as_raw()) };

            let env = jvm.to_env();
            assert!(!jobject.is_null());
            let class = env.get_object_class(&jobject).unwrap();
            assert!(env.is_assignable_from(class, to_jclass).unwrap());
        }

        let env = jvm.to_env();
        // Safety: From: Upcast<To>
        unsafe {
            let casted = env.new_local_ref(jobject)?;
            Ok(Local::from_jni(env.auto_local(casted)))
        }
    }
}
