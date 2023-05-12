use crate::jvm::JavaScalar;
use crate::jvm::Jvm;
use crate::jvm::JvmOp;
use crate::AsJRef;
use crate::Global;
use crate::JavaObject;
use crate::Local;

macro_rules! identity_jvm_op {
    ($([$($param:tt)*] $t:ty,)*) => {
        $(
            impl<$($param)*> JvmOp for $t {
                type Output<'jvm> = Self;

                fn execute_with<'jvm>(self, _jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, Self::Output<'jvm>> {
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
    [R: JavaObject] &Global<R>,
    [R: JavaObject] &Option<Local<'_, R>>,
    [R: JavaObject] &Option<Global<R>>,
}

/// Types that are able to be used as a Java `T`, either because they will produce a Java `T` (e.g. [`JvmOp`]s that
/// produce a `T`) or because we can convert into them via a JNI call.
///
/// **Don't implement this directly.** Instead, implement `JvmOp`. This trait is intended
/// to be used as a shorthand trait alias in Duchess fn definitions, like
///
/// ```ignore
/// fn my_java_call(a: impl IntoJava<JavaString>, b: impl IntoJava<JavaArray<i8>>) -> impl JvmOp {
///    // ...
/// }
///
/// let a = some_java_op_that_produces_a_string();
/// let b = [1, 2, 3].as_slice();
/// my_java_call(a, b).execute(&jvm)?;
/// ```
pub trait IntoJava<T: JavaObject> {
    type Output<'jvm>: AsJRef<T>;

    fn into_java<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, Self::Output<'jvm>>;
}

impl<J, T> IntoJava<T> for J
where
    T: JavaObject,
    for<'jvm> J: JvmOp,
    for<'jvm> J::Output<'jvm>: AsJRef<T>,
{
    type Output<'jvm> = <J as JvmOp>::Output<'jvm>;

    fn into_java<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, Self::Output<'jvm>> {
        self.execute_with(jvm)
    }
}

/// A [`JvmOp`] that produces a [`Local`] reference to a `T` object.
/// Local references are values that are only valid in this JNI call.
/// They can be converted to [`Global`] references.
pub trait IntoLocal<T: JavaObject>: for<'jvm> JvmOp<Output<'jvm> = Local<'jvm, T>> {}

impl<J, T> IntoLocal<T> for J
where
    T: JavaObject,
    J: for<'jvm> JvmOp<Output<'jvm> = Local<'jvm, T>>,
{
}

/// A [`JvmOp`] that produces an optional [`Local`] reference to a `T`;
/// None will be used if the result is `null`.
/// Local references are values that are only valid in this JNI call.
/// They can be converted to [`Global`] references.
pub trait IntoOptLocal<T: JavaObject>:
    for<'jvm> JvmOp<Output<'jvm> = Option<Local<'jvm, T>>>
{
}

impl<J, T> IntoOptLocal<T> for J
where
    T: JavaObject,
    J: for<'jvm> JvmOp<Output<'jvm> = Option<Local<'jvm, T>>>,
{
}

/// A [`JvmOp`] that produces a scalar value, like `i8` or `i32`.
pub trait IntoScalar<T: JavaScalar>: for<'jvm> JvmOp<Output<'jvm> = T> {}

impl<J, T> IntoScalar<T> for J
where
    T: JavaScalar,
    J: for<'jvm> JvmOp<Output<'jvm> = T>,
{
}

/// A [`JvmOp`] that produces a void (`()`)
pub trait IntoVoid: for<'jvm> JvmOp<Output<'jvm> = ()> {}

impl<J> IntoVoid for J where J: for<'jvm> JvmOp<Output<'jvm> = ()> {}

/// A java method that returns a `T` object (when executed).
pub trait JavaMethod<T>
where
    T: JavaObject,
    for<'jvm> Self: JvmOp<Output<'jvm> = Option<Local<'jvm, T>>>,
{
}

impl<J, T> JavaMethod<T> for J
where
    T: JavaObject,
    for<'jvm> Self: JvmOp<Output<'jvm> = Option<Local<'jvm, T>>>,
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
