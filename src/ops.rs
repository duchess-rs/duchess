use crate::jvm::Jvm;
use crate::jvm::JvmOp;
use crate::Global;
use crate::JavaObject;
use crate::Local;
use jni::errors::Result as JniResult;

macro_rules! scalar_jvm_op {
    ($($t:ty,)*) => {
        $(
            impl JvmOp for $t {
                type Output<'jvm> = Self;

                fn execute<'jvm>(self, _jvm: &mut Jvm<'jvm>) -> JniResult<Self::Output<'jvm>> {
                    Ok(self)
                }
            }
        )*
    };
}

scalar_jvm_op! {
    i8,  // byte
    i16, // short
    i32, // int
    i64, // long

    char, // char

    (),  // void

    f32, // float
    f64, // double
}

macro_rules! obj_op {
    ($R:ident => { $($t:ty,)* }) => {
        $(
            impl<$R> JvmOp for $t
            where
                $R: JavaObject,
            {
                type Output<'jvm> = Self;

                fn execute<'jvm>(self, _jvm: &mut Jvm<'jvm>) -> JniResult<Self> {
                    Ok(self)
                }
            }
        )*
    }
}

obj_op! {
    R => {
        &R,
        Local<'_, R>,
        &Local<'_, R>,
        Global<R>,
        &Global<R>,
    }
}
