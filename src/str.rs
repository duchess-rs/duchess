use jni::{
    objects::{AutoLocal, JObject},
    strings::JNIString,
};

use crate::{JavaObject, Jvm, JvmOp, Local};

pub struct JavaString {
    _private: (),
}

unsafe impl JavaObject for JavaString {}

pub trait ToJavaStringOp: JvmOp + Sized {
    fn to_java_string(self) -> JavaStringOp<Self>;
}

impl<J: JvmOp> ToJavaStringOp for J
where
    for<'jvm> J::Output<'jvm>: Into<JNIString>,
{
    fn to_java_string(self) -> JavaStringOp<Self> {
        JavaStringOp { op: self }
    }
}

pub struct JavaStringOp<J: JvmOp> {
    op: J,
}

impl<J: JvmOp> JvmOp for JavaStringOp<J>
where
    for<'jvm> J::Output<'jvm>: Into<JNIString>,
{
    type Output<'jvm> = Local<'jvm, JavaString>;

    fn execute<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::Result<Self::Output<'jvm>> {
        let data = self.op.execute(jvm)?;
        let data: JNIString = data.into();

        let env = jvm.to_env();
        let o = env.new_string(data)?;
        let o: JObject = o.into();
        unsafe { Ok(Local::from_jni(AutoLocal::new(o, &env))) }
    }
}
