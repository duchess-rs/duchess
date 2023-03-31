use duchess::jvm::{JavaObject, JavaObjectExt, JdkOp, Jvm, Local};
use jni::objects::{AutoLocal, JValue, JValueGen};

pub struct Logger {
    _dummy: (),
}

unsafe impl JavaObject for Logger {}

// class Logger {
//    public Logger();
// }

impl Logger {
    pub fn new() -> LoggerConstructor {
        LoggerConstructor { private: () }
    }
}

pub struct LoggerConstructor {
    private: (),
}

impl JdkOp for LoggerConstructor {
    type Output<'jvm> = Local<'jvm, Logger>;

    fn execute<'jvm>(self, jvm: &'jvm Jvm) -> jni::errors::Result<Self::Output<'jvm>> {
        let mut env = jvm.to_env();

        // FIXME: how do we cache this
        let class = env.find_class("me/ferris/Logger")?;

        env.new_object(class, "()", &[])
            .map(|o| unsafe { Local::from_jni(AutoLocal::new(o, &env)) })
    }
}

// class Logger {
//     public void log(String data);
// }

struct LoggerLog<J, S> {
    this: J,
    data: S,
}

impl<J, S> JdkOp for LoggerLog<J, S>
where
    J: JdkOp,
    for<'jvm> J::Output<'jvm>: AsRef<Logger>,
    S: JdkOp,
    for<'jvm> S::Output<'jvm>: Into<i32>,
{
    type Output<'jvm> = ();

    fn execute<'jvm>(self, jvm: &'jvm Jvm) -> jni::errors::Result<Self::Output<'jvm>> {
        let this = self.this.execute(jvm)?;
        let this: &Logger = this.as_ref();

        let mut env = jvm.to_env();

        let data = self.data.execute(jvm)?;
        let data: i32 = data.into();

        match env.call_method(
            this.as_jobject(),
            "log",
            "(Ljava/lang/String;)V",
            &[JValue::from(data)],
        )? {
            JValueGen::Void => Ok(()),
            _ => panic!("class file out of sync"),
        }
    }
}

fn main() {
    todo!()
}
