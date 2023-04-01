use crate::jvm::Jvm;
use crate::jvm::JvmOp;
use crate::Global;
use crate::JavaObject;
use crate::Local;
use jni::errors::Result as JniResult;

macro_rules! identity_jvm_op {
    ($([$($param:tt)*] $t:ty,)*) => {
        $(
            impl<$($param)*> JvmOp for $t {
                type Output<'jvm> = Self;

                fn execute<'jvm>(self, _jvm: &mut Jvm<'jvm>) -> JniResult<Self::Output<'jvm>> {
                    Ok(self)
                }
            }
        )*
    };
}

identity_jvm_op! {
    [] i8,  // byte
    [] i16, // short
    [] i32, // int
    [] i64, // long

    [] char, // char

    [] (),  // void

    [] f32, // float
    [] f64, // double

    [R: JavaObject] &R,
    [R: JavaObject] Local<'_, R>,
    [R: JavaObject] &Local<'_, R>,
    [R: JavaObject] Global<R>,
    [R: JavaObject] &Global<R>,

    [] &str,
    [] &String,
    [] String,
}
