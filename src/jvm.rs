use crate::inspect::{ArgOp, Inspect};

use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use jni::{
    objects::{AutoLocal, GlobalRef, JObject, JValueOwned},
    sys, InitArgsBuilder, JNIEnv, JavaVM,
};
use once_cell::sync::Lazy;

/// A "jdk op" is a suspended operation that, when executed, will run
/// on the jvm, producing a value of type `Output`. These ops typically
/// represent constructor or method calls, and they can be chained
/// together.
///
/// *Eventual goal:* Each call to `execute` represents a single crossing
/// over into the JVM, so the more you can chain together your jvm-ops,
/// the better.
pub trait JvmOp: Clone {
    type Input<'jvm>;
    type Output<'jvm>;

    /// "Inspect" executes an operation on this value that results in unit
    /// and then yields up this value again for further use. If the result
    /// of `self` is the java value `x`, then `self.inspect(|x| x.foo()).bar()`
    /// is equivalent to the Java code `x.foo(); return x.bar()`.
    fn inspect<K>(self, op: impl FnOnce(ArgOp<Self>) -> K) -> Inspect<Self, K>
    where
        for<'jvm> Self::Output<'jvm>: CloneIn<'jvm>,
        K: JvmOp,
        for<'jvm> K: JvmOp<Input<'jvm> = Self::Output<'jvm>, Output<'jvm> = ()>,
    {
        Inspect::new(self, op)
    }

    fn execute_with<'jvm>(
        self,
        jvm: &mut Jvm<'jvm>,
        arg: Self::Input<'jvm>,
    ) -> crate::Result<Self::Output<'jvm>>;
}

/// This trait is only implemented for `()`; it allows the `JvmOp::execute` method to only
/// be used for `()`.
pub trait IsVoid: Default {}
impl IsVoid for () {}

static GLOBAL_JVM: Lazy<JavaVM> = Lazy::new(|| {
    let jvm_args = InitArgsBuilder::new()
        .version(jni::JNIVersion::V8)
        .option("-Xcheck:jni")
        .option("-Djava.class.path=java")
        .build()
        .unwrap();

    JavaVM::new(jvm_args).unwrap()
});

#[repr(transparent)]
pub struct Jvm<'jvm> {
    env: JNIEnv<'jvm>,
}

impl<'jvm> Jvm<'jvm> {
    pub fn with<R>(op: impl FnOnce(&mut Jvm<'_>) -> crate::Result<R>) -> crate::Result<R> {
        let guard = GLOBAL_JVM.attach_current_thread()?;

        // Safety condition: must not be used to create new references
        // unless they are contained by `guard`. In this case, the
        // cloned env is fully contained within the lifetime of `guard`
        // and basically takes its place. The only purpose here is to
        // avoid having two lifetime parameters on `Jvm`; trying to
        // keep the interface simpler.
        let env = unsafe { guard.unsafe_clone() };

        op(&mut Jvm { env })
    }

    pub fn to_env(&mut self) -> &mut JNIEnv<'jvm> {
        &mut self.env
    }

    pub fn local<R>(&mut self, r: &R) -> Local<'jvm, R>
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

    pub fn global<R>(&mut self, _r: &R) -> Global<R>
    where
        R: JavaObject,
    {
        todo!()
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
/// # use duchess::JavaObject;
/// pub struct BigDecimal {
///     _private: (), // prevent construction
/// }
/// unsafe impl JavaObject for BigDecimal {}
/// ```
pub unsafe trait JavaObject: Sized {}

/// Extension trait for [JavaObject].
pub trait JavaObjectExt: Sized {
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

pub trait JavaScalar {}

impl JavaScalar for i8 {}
impl JavaScalar for i16 {}
impl JavaScalar for i32 {}
impl JavaScalar for i64 {}
impl JavaScalar for bool {}

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
#[repr(transparent)]
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

/// A trait to represent safe upcast operations for a [`JavaObject`].
///
/// # Safety
///
/// Inherits the rules of [`JavaObject`], but also `S` must be a valid superclass or implemented interface of `Self`.
/// XX: would this actually allow unsafe behavior in a JNI call? or is it already checked/enforced?
///
/// XX: having to impl Upcast<T> for T on each struct is pretty annoying to get AsRef<T> to work without conflicts
pub unsafe trait Upcast<S: JavaObject>: JavaObject {}

impl<'a, R, S> AsRef<S> for Local<'a, R>
where
    R: Upcast<S>,
    S: JavaObject + 'a,
{
    fn as_ref(&self) -> &S {
        // XX: Safety: repr(transparent) so OwnedRef<J, X> and OwnedRef<J, Y> have the same repr
        let upcast: &Local<'a, S> = unsafe { std::mem::transmute(self) };
        upcast
    }
}

impl<'a, R> Local<'a, R> {
    /// XX: Trying to map onto Into causes impl conflits
    pub fn upcast<S>(self) -> Local<'a, S>
    where
        R: Upcast<S>,
        S: JavaObject + 'a,
    {
        // XX: Safety: repr(transparent) so OwnedRef<J, X> and OwnedRef<J, Y> have the same repr
        let upcast: Local<'a, S> = unsafe { std::mem::transmute(self) };
        upcast
    }
}

impl<R, S> AsRef<S> for Global<R>
where
    R: Upcast<S>,
    S: JavaObject + 'static,
{
    fn as_ref(&self) -> &S {
        // XX: Safety: repr(transparent) so OwnedRef<J, X> and OwnedRef<J, Y> have the same repr
        let upcast: &Global<S> = unsafe { std::mem::transmute(self) };
        upcast
    }
}

impl<R> Global<R> {
    /// XX: Trying to map onto Into causes impl conflits
    pub fn upcast<S>(self) -> Global<S>
    where
        R: Upcast<S>,
        S: JavaObject + 'static,
    {
        // XX: Safety: repr(transparent) so OwnedRef<J, X> and OwnedRef<J, Y> have the same repr
        let upcast: Global<S> = unsafe { std::mem::transmute(self) };
        upcast
    }
}

pub trait CloneIn<'jvm> {
    fn clone_in(&self, jvm: &mut Jvm<'jvm>) -> Self;
}

impl<T> CloneIn<'_> for T
where
    T: Clone,
{
    fn clone_in(&self, _jvm: &mut Jvm<'_>) -> Self {
        self.clone()
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

pub trait FromJValue<'jvm> {
    fn from_jvalue(jvm: &mut Jvm<'jvm>, value: JValueOwned<'jvm>) -> Self;
}

impl<'jvm, T> FromJValue<'jvm> for Option<Local<'jvm, T>>
where
    T: JavaObject,
{
    fn from_jvalue(jvm: &mut Jvm<'jvm>, value: JValueOwned<'jvm>) -> Self {
        match value {
            jni::objects::JValueGen::Object(o) => {
                if o.is_null() {
                    None
                } else {
                    let env = jvm.to_env();
                    Some(unsafe { Local::from_jni(AutoLocal::new(o, &env)) })
                }
            }
            _ => panic!("expected object, found {value:?})"),
        }
    }
}

impl<'jvm> FromJValue<'jvm> for () {
    fn from_jvalue(_jvm: &mut Jvm<'jvm>, value: JValueOwned<'jvm>) -> Self {
        match value {
            jni::objects::JValueGen::Void => (),
            _ => panic!("expected void, found {value:?})"),
        }
    }
}

impl<'jvm> FromJValue<'jvm> for i32 {
    fn from_jvalue(_jvm: &mut Jvm<'jvm>, value: JValueOwned<'jvm>) -> Self {
        match value {
            jni::objects::JValueGen::Int(i) => i,
            _ => panic!("expected void, found {value:?})"),
        }
    }
}

impl<'jvm> FromJValue<'jvm> for i64 {
    fn from_jvalue(_jvm: &mut Jvm<'jvm>, value: JValueOwned<'jvm>) -> Self {
        match value {
            jni::objects::JValueGen::Long(i) => i,
            _ => panic!("expected void, found {value:?})"),
        }
    }
}

impl<'jvm> FromJValue<'jvm> for i8 {
    fn from_jvalue(_jvm: &mut Jvm<'jvm>, value: JValueOwned<'jvm>) -> Self {
        match value {
            jni::objects::JValueGen::Byte(i) => i,
            _ => panic!("expected void, found {value:?})"),
        }
    }
}

impl<'jvm> FromJValue<'jvm> for i16 {
    fn from_jvalue(_jvm: &mut Jvm<'jvm>, value: JValueOwned<'jvm>) -> Self {
        match value {
            jni::objects::JValueGen::Short(i) => i,
            _ => panic!("expected void, found {value:?})"),
        }
    }
}

impl<'jvm> FromJValue<'jvm> for bool {
    fn from_jvalue(_jvm: &mut Jvm<'jvm>, value: JValueOwned<'jvm>) -> Self {
        match value {
            jni::objects::JValueGen::Bool(i) => i != 0,
            _ => panic!("expected void, found {value:?})"),
        }
    }
}

impl<'jvm> FromJValue<'jvm> for f32 {
    fn from_jvalue(_jvm: &mut Jvm<'jvm>, value: JValueOwned<'jvm>) -> Self {
        match value {
            jni::objects::JValueGen::Float(i) => i,
            _ => panic!("expected void, found {value:?})"),
        }
    }
}

impl<'jvm> FromJValue<'jvm> for f64 {
    fn from_jvalue(_jvm: &mut Jvm<'jvm>, value: JValueOwned<'jvm>) -> Self {
        match value {
            jni::objects::JValueGen::Double(i) => i,
            _ => panic!("expected void, found {value:?})"),
        }
    }
}
