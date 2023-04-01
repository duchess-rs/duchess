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

pub trait LoggerExt: Sized {
    fn logInt<D>(self, data: D) -> LoggerLog<Self, D>
    where
        D: JvmOp,
        for<'jvm> D::Output<'jvm>: Into<i32>;
}

impl<T> LoggerExt for T
where
    T: JvmOp,
    for<'jvm> T::Output<'jvm>: AsRef<Logger>,
{
    fn logInt<D>(self, data: D) -> LoggerLog<Self, D>
    where
        D: JvmOp,
        for<'jvm> D::Output<'jvm>: Into<i32>,
    {
        LoggerLog { this: self, data }
    }
}

pub struct LoggerConstructor {
    _private: (),
}

impl JvmOp for LoggerConstructor {
    type Output<'jvm> = Local<'jvm, Logger>;

    fn execute<'jvm>(self, jvm: &mut Jvm<'jvm>) -> jni::errors::Result<Self::Output<'jvm>> {
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

pub struct LoggerLog<J, S> {
    this: J,
    data: S,
}

impl<J, S> JvmOp for LoggerLog<J, S>
where
    J: JvmOp,
    for<'jvm> J::Output<'jvm>: AsRef<Logger>,
    S: JvmOp,
    for<'jvm> S::Output<'jvm>: Into<i32>,
{
    type Output<'jvm> = ();

    fn execute<'jvm>(self, jvm: &mut Jvm<'jvm>) -> jni::errors::Result<Self::Output<'jvm>> {
        use duchess::plumbing::JavaObjectExt;

        let this = self.this.execute(jvm)?;
        let this: &Logger = this.as_ref();

        let data = self.data.execute(jvm)?;
        let data: i32 = data.into();

        let env = jvm.to_env();
        match env.call_method(this.as_jobject(), "logInt", "(I)V", &[JValue::from(data)])? {
            JValueGen::Void => Ok(()),
            _ => panic!("class file out of sync"),
        }
    }
}

fn main() -> jni::errors::Result<()> {
    Jvm::with(|jvm| Logger::new().logInt(22).execute(jvm))
}
