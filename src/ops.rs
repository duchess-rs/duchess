use crate::jvm::Jvm;
use crate::jvm::JvmOp;
use crate::Global;
use crate::JavaObject;
use crate::Local;

macro_rules! identity_jvm_op {
    ($([$($param:tt)*] $t:ty,)*) => {
        $(
            impl<$($param)*> JvmOp for $t {
                type Input<'jvm> = ();
                type Output<'jvm> = Self;

                fn execute_with<'jvm>(self, _jvm: &mut Jvm<'jvm>, (): ()) -> crate::Result<Self::Output<'jvm>> {
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

    [] &str,
    [] &String,
}

/// Types that are able to be used as a Java `T`, either because they will produce a Java `T` (e.g. [`JvmOp`]s that
/// produce a `T`) or because we can convert into them via a JNI call.
///
/// See [`crate::str::IntoJavaString`].
///
/// This is intended to be used as a shorthand trait alias in Duchess fn definitions, like
/// ```ignore
/// fn my_java_call(a: impl IntoJava<JavaString>, b: impl IntoJava<JavaArray<i8>>) -> impl JvmOp {
///    // ...
/// }
///
/// let a = some_java_op_that_produces_a_string();
/// let b = [1, 2, 3].as_slice();
/// my_java_call(a, b).execute(&jvm)?;
/// ```
pub trait IntoJava<T: JavaObject>: Clone {
    type Output<'jvm>: AsRef<T>;

    fn into_java<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::Result<Self::Output<'jvm>>;
}

impl<J, T> IntoJava<T> for J
where
    T: JavaObject,
    for<'jvm> J: JvmOp<Input<'jvm> = ()>,
    for<'jvm> J::Output<'jvm>: AsRef<T>,
{
    type Output<'jvm> = <J as JvmOp>::Output<'jvm>;

    fn into_java<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::Result<Self::Output<'jvm>> {
        self.execute(jvm)
    }
}

/// Types that are able to be converted back into a Rust `T`, either because they will produce a Rust primitive `T` or
/// or because we can convert into them via a JNI call.
///
/// This is intended to be used to explicitly bring a value back to Rust at the end of a JVM session or operation.
pub trait IntoRust<T> {
    type Op: for<'jvm> JvmOp<Output<'jvm> = T>;

    fn into_rust(self) -> Self::Op;
}
