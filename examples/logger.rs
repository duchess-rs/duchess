use duchess::{prelude::*, Jvm, JvmOp};
use jni::{
    objects::{JValue, JValueGen},
    strings::JNIString,
};

duchess::plumbing::duchess_javap! {
    r#"
        Compiled from "Logger.java"
        class me.ferris.Logger {
        me.ferris.Logger();
            descriptor: ()V

        void logInt(int);
            descriptor: (I)V

        void logString(java.lang.String);
            descriptor: (Ljava/lang/String;)V
        }
    "#
}

// class Logger {
//    public Logger();
// }

pub trait LoggerExt: JvmOp + Sized {
    fn log_int<D>(self, data: D) -> LoggerLogInt<Self, D>
    where
        D: JvmOp,
        for<'jvm> D::Output<'jvm>: Into<i32>;

    fn log_string<D>(self, data: D) -> LoggerLogString<Self, D>
    where
        D: JvmOp,
        for<'jvm> D::Output<'jvm>: Into<JNIString>;
}

impl<T> LoggerExt for T
where
    T: JvmOp,
    for<'jvm> T::Output<'jvm>: AsRef<Logger>,
{
    fn log_int<D>(self, data: D) -> LoggerLogInt<Self, D>
    where
        D: JvmOp,
        for<'jvm> D::Output<'jvm>: Into<i32>,
    {
        LoggerLogInt { this: self, data }
    }

    fn log_string<D>(self, data: D) -> LoggerLogString<Self, D>
    where
        D: JvmOp,
        for<'jvm> D::Output<'jvm>: Into<JNIString>,
    {
        LoggerLogString { this: self, data }
    }
}

// class Logger {
//     public void logInt(int data);
// }

#[derive(Clone)]
pub struct LoggerLogInt<J: JvmOp, S: JvmOp> {
    this: J,
    data: S,
}

impl<J, S> JvmOp for LoggerLogInt<J, S>
where
    J: JvmOp,
    for<'jvm> J::Output<'jvm>: AsRef<Logger>,
    S: IntoScalar<i32>,
{
    type Input<'jvm> = J::Input<'jvm>;
    type Output<'jvm> = ();

    fn execute_with<'jvm>(
        self,
        jvm: &mut Jvm<'jvm>,
        input: J::Input<'jvm>,
    ) -> jni::errors::Result<Self::Output<'jvm>> {
        use duchess::plumbing::JavaObjectExt;

        let this = self.this.execute_with(jvm, input)?;
        let this: &Logger = this.as_ref();
        let this = this.as_jobject();

        let data = self.data.execute(jvm)?;

        let env = jvm.to_env();
        match env.call_method(this, "logInt", "(I)V", &[JValue::from(data)])? {
            JValueGen::Void => Ok(()),
            _ => panic!("class file out of sync"),
        }
    }
}

// class Logger {
//     public void logString(String data);
// }

#[derive(Clone)]
pub struct LoggerLogString<J: JvmOp, S: JvmOp> {
    this: J,
    data: S,
}

impl<J, S> JvmOp for LoggerLogString<J, S>
where
    J: JvmOp,
    for<'jvm> J::Output<'jvm>: AsRef<Logger>,
    S: JvmOp,
    for<'jvm> S: JvmOp<Input<'jvm> = ()>,
    for<'jvm> S::Output<'jvm>: Into<JNIString>,
{
    type Input<'jvm> = J::Input<'jvm>;
    type Output<'jvm> = ();

    fn execute_with<'jvm>(
        self,
        jvm: &mut Jvm<'jvm>,
        input: J::Input<'jvm>,
    ) -> jni::errors::Result<Self::Output<'jvm>> {
        use duchess::plumbing::{JavaObjectExt, ToJavaStringOp};

        let this = self.this.execute_with(jvm, input)?;
        let this: &Logger = this.as_ref();

        let data = self.data.to_java_string().execute_with(jvm, ())?;

        let env = jvm.to_env();
        let data = data.as_jobject();
        match env.call_method(
            this.as_jobject(),
            "logString",
            "(Ljava/lang/String;)V",
            &[JValue::from(&data)],
        )? {
            JValueGen::Void => Ok(()),
            _ => panic!("class file out of sync"),
        }
    }
}

fn main() -> jni::errors::Result<()> {
    Jvm::with(|jvm| {
        let l = Logger::new().execute(jvm)?;
        l.log_int(22).execute(jvm)?;
        l.log_string("Hello, Duchess!").execute(jvm)?;

        Logger::new()
            .inspect(|l| l.log_int(23))
            .inspect(|l| l.log_string("Hello again, Duchess!"))
            .execute(jvm)?;

        Ok(())
    })
}
