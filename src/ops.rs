use crate::jvm::JavaScalar;
use crate::jvm::Jvm;
use crate::jvm::JvmOp;
use crate::jvm::JvmRefOp;
use crate::jvm::JvmScalarOp;
use crate::Java;
use crate::JavaObject;
use crate::Local;
use crate::Null;

macro_rules! identity_jvm_op {
    ($([$($param:tt)*] $t:ty,)*) => {
        $(
            impl<$($param)*> JvmOp for $t {
                type Output<'jvm> = Self;

                fn do_jni<'jvm>(self, _jvm: &mut Jvm<'jvm>) -> crate::LocalResult<'jvm, Self::Output<'jvm>> {
                    Ok(self)
                }
            }
        )*
    };
}

identity_jvm_op! {
    [] bool, // bool
    [] i8,   // byte
    [] i16,  // short
    [] i32,  // int
    [] i64,  // long

    [] char, // long (Java char is 2 bytes / UTF-16)
    [] u16,  // char

    [] (),  // void

    [] f32, // float
    [] f64, // double

    [R: JavaObject] &R,
    [R: JavaObject] &Local<'_, R>,
    [R: JavaObject] &Java<R>,
    [R: JavaObject] &Option<Local<'_, R>>,
    [R: JavaObject] &Option<Java<R>>,
    [R: JavaObject] &Option<&R>,

    [] Null,
}

/// Value that can be given as argument to a Java method expecting a value of type `T`.
///
/// Typically this is a [`JvmOp`][] that produces a `T` value, but it can also be a
/// Rust value that will be wrapped or given to Java somehow.
///
/// This trait's only method (`into_op`) occurs without any JVM in scope,
/// so it is limited to doing Rust operations.
/// If you need to perform JNI operations, implement [`JvmOp`][] for your type
/// (which will in turn mean that this trait is implemented).
pub trait IntoJava<T: JavaObject> {
    type JvmOp: JvmRefOp<T>;

    fn into_op(self) -> Self::JvmOp;
}

impl<J, T> IntoJava<T> for J
where
    T: JavaObject,
    J: JvmRefOp<T>,
{
    type JvmOp = J;

    fn into_op(self) -> Self::JvmOp {
        self
    }
}

/// Value that can be given as argument to a Java method expecting a scalar value of type `T` (e.g., `i8`).
///
/// Typically this is a [`JvmOp`][] that produces a `T` value, but it can also be a
/// Rust value that will be wrapped or given to Java somehow. See [`IntoJava`][] for more details.
pub trait IntoScalar<T: JavaScalar> {
    type JvmOp: JvmScalarOp<T>;

    fn into_op(self) -> Self::JvmOp;
}

impl<J, T> IntoScalar<T> for J
where
    T: JavaScalar,
    J: JvmScalarOp<T>,
{
    type JvmOp = J;

    fn into_op(self) -> Self::JvmOp {
        self
    }
}

/// A [`JvmOp`] that produces a [`Local`] reference to a `T` object.
/// Local references are values that are only valid in this JNI call.
/// They can be converted to [`Global`] references.
pub trait JavaConstructor<T: JavaObject>
where
    Self: for<'jvm> JvmOp<Output<'jvm> = Local<'jvm, T>>,
    Self: std::ops::Deref<Target = T::OfOp<Self>>,
{
}

impl<J, T> JavaConstructor<T> for J
where
    T: JavaObject,
    J: for<'jvm> JvmOp<Output<'jvm> = Local<'jvm, T>>,
    J: std::ops::Deref<Target = T::OfOp<Self>>,
{
}

/// A [`JvmOp`] that produces a void (`()`)
pub trait IntoVoid: for<'jvm> JvmOp<Output<'jvm> = ()> {}

impl<J> IntoVoid for J where J: for<'jvm> JvmOp<Output<'jvm> = ()> {}

/// A java method that returns a `T` object (when executed).
pub trait JavaMethod<T>
where
    T: JavaObject,
    Self: for<'jvm> JvmOp<Output<'jvm> = Option<Local<'jvm, T>>>,
    Self: std::ops::Deref<Target = T::OfOp<Self>>,
{
}

impl<J, T> JavaMethod<T> for J
where
    T: JavaObject,
    for<'jvm> Self: JvmOp<Output<'jvm> = Option<Local<'jvm, T>>>,
    J: std::ops::Deref<Target = T::OfOp<J>>,
{
}

/// A java method that returns a scalar value of type `T` when executed.
pub trait ScalarMethod<T>
where
    T: JavaScalar,
    for<'jvm> Self: JvmOp<Output<'jvm> = T>,
{
}

impl<J, T> ScalarMethod<T> for J
where
    T: JavaScalar,
    for<'jvm> Self: JvmOp<Output<'jvm> = T>,
{
}

/// A java method that returns void when executed.
pub trait VoidMethod
where
    for<'jvm> Self: JvmOp<Output<'jvm> = ()>,
{
}

impl<J> VoidMethod for J where for<'jvm> Self: JvmOp<Output<'jvm> = ()> {}

/// A java field that returns a `T` object (when executed).
pub trait JavaField<T>
where
    T: JavaObject,
    for<'jvm> Self: JvmOp<Output<'jvm> = Option<Local<'jvm, T>>>,
{
}

impl<J, T> JavaField<T> for J
where
    T: JavaObject,
    for<'jvm> Self: JvmOp<Output<'jvm> = Option<Local<'jvm, T>>>,
{
}

/// A java field that returns a scalar value of type `T` when executed.
pub trait ScalarField<T>
where
    T: JavaScalar,
    for<'jvm> Self: JvmOp<Output<'jvm> = T>,
{
}

impl<J, T> ScalarField<T> for J
where
    T: JavaScalar,
    for<'jvm> Self: JvmOp<Output<'jvm> = T>,
{
}
