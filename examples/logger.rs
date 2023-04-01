use duchess::{JavaObject, Jvm, JvmOp, Local};
use jni::{
    objects::{AutoLocal, JValue, JValueGen},
    strings::JNIString,
};

pub struct Logger {
    _dummy: (),
}

unsafe impl JavaObject for Logger {}

// class Logger {
//    public Logger();
// }

impl Logger {
    pub fn new() -> LoggerConstructor {
        LoggerConstructor { _private: () }
    }
}

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

#[derive(Clone)]
pub struct LoggerConstructor {
    _private: (),
}

impl JvmOp for LoggerConstructor {
    type Input<'jvm> = ();
    type Output<'jvm> = Local<'jvm, Logger>;

    fn execute_with<'jvm>(
        self,
        jvm: &mut Jvm<'jvm>,
        (): (),
    ) -> jni::errors::Result<Self::Output<'jvm>> {
        let env = jvm.to_env();

        // FIXME: how do we cache this
        let class = env.find_class("me/ferris/Logger")?;

        env.new_object(class, "()V", &[])
            .map(|o| unsafe { Local::from_jni(AutoLocal::new(o, &env)) })
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
    S: JvmOp,
    for<'jvm> S: JvmOp<Input<'jvm> = ()>,
    for<'jvm> S::Output<'jvm>: Into<i32>,
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

        let data = self.data.execute_with(jvm, ())?;
        let data: i32 = data.into();

        let env = jvm.to_env();
        match env.call_method(this.as_jobject(), "logInt", "(I)V", &[JValue::from(data)])? {
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
            .execute(jvm)?
            .inspect(|l| l.log_int(23))
            .inspect(|l| l.log_string("Hello again, Duchess!"))
            .execute(jvm)?;

        Ok(())
    })
}
