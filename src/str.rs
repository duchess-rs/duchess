use jni::{
    objects::{AutoLocal, JObject, JString},
    strings::JNIString,
};

use crate::{
    java::lang::String as JavaString,
    jvm::JavaObjectExt,
    ops::{IntoJava, IntoRust},
    plumbing::with_jni_env,
    Jvm, JvmOp, Local,
};

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

#[derive(Clone)]
pub struct JavaStringOp<J: JvmOp> {
    op: J,
}

impl<J: JvmOp> JvmOp for JavaStringOp<J>
where
    for<'jvm> J::Output<'jvm>: Into<JNIString>,
{
    type Input<'jvm> = J::Input<'jvm>;
    type Output<'jvm> = Local<'jvm, JavaString>;

    fn execute_with<'jvm>(
        self,
        jvm: &mut Jvm<'jvm>,
        input: Self::Input<'jvm>,
    ) -> crate::Result<'jvm, Self::Output<'jvm>> {
        let data = self.op.execute_with(jvm, input)?;
        let env = jvm.to_env();
        let o = with_jni_env(env, |env| env.new_string(data))?;
        let o: JObject = o.into();
        unsafe { Ok(Local::from_jni(AutoLocal::new(o, &env))) }
    }
}

impl IntoJava<JavaString> for &str {
    type Output<'jvm> = Local<'jvm, JavaString>;

    fn into_java<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, Local<'jvm, JavaString>> {
        let env = jvm.to_env();
        let string = with_jni_env(env, |env| env.new_string(self))?;
        unsafe { Ok(Local::from_jni(AutoLocal::new(JObject::from(string), &env))) }
    }
}

impl IntoJava<JavaString> for String {
    type Output<'jvm> = Local<'jvm, JavaString>;

    fn into_java<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, Local<'jvm, JavaString>> {
        <&str as IntoJava<JavaString>>::into_java(&self, jvm)
    }
}

impl<J> IntoRust<String> for J
where
    for<'jvm> J: JvmOp<Input<'jvm> = ()>,
    for<'jvm> J::Output<'jvm>: AsRef<JavaString>,
{
    fn into_rust<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, String> {
        let object = self.execute_with(jvm, ())?;
        let env = jvm.to_env();
        // XX: safety? is this the right way to do this cast?
        let string_object = unsafe { JString::from_raw(object.as_ref().as_jobject().as_raw()) };
        let string = with_jni_env(env, |env| unsafe {
            env.get_string_unchecked(&string_object)
        })?;
        Ok(string.into())
    }
}
