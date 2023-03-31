use jni::{
    objects::{JValue, JValueGen},
    strings::JNIString,
};

use crate::jvm::{Anchor, JavaObject, JdkOp, Jvm, Local};

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
            .map(|o| unsafe { Local::from_jobject(o) })
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
    J: for<'jvm> JdkOp<Output<'jvm> = &'jvm Logger>,
    S: for<'jvm> JdkOp<Output<'jvm> = JNIString>,
{
    type Output<'jvm> = ();

    fn execute<'jvm>(self, jvm: &'jvm Jvm) -> jni::errors::Result<Self::Output<'jvm>> {
        let this = self.this.execute(jvm)?;
        let mut env = jvm.to_env();

        let data = self.data.execute(jvm)?;

        let this = Anchor::from(&*this);
        match env.call_method(
            &this,
            "log",
            "(Ljava/lang/String;)V",
            &[JValue::from(&data)],
        )? {
            JValueGen::Void => Ok(()),
            _ => panic!("class file out of sync"),
        }
    }
}

impl Logger {
    // static methods require the `jvm` argument to limit the lifetime of the returned local ref
    pub fn global_logger<'jvm>(jvm: &'jvm Jvm) -> jni::errors::Result<Local<'jvm, Self>> {
        let mut env = jvm.to_env();

        // FIXME: how do we cache this
        let class = env.find_class("me/ferris/Logger")?;

        match env.call_static_method(class, "globalLogger", "()Lme/ferris/Logger;", &[])? {
            JValueGen::Object(o) => unsafe { Ok(Local::from_jobject(o)) },
            _ => panic!("class file out of sync"),
        }
    }

    // but normal methods can use the lifetime of `&self`
    pub fn log(&self, s: impl Into<JNIString>) -> jni::errors::Result<()> {
        // FIXME: can we do better than this `impl Into` business, it feels inefficient
        Jvm::with(|jvm| {
            let mut env = jvm.to_env();
            let this = Anchor::from(self);
            let js = env.new_string(s)?;
            match env.call_method(&this, "log", "(Ljava/lang/String;)V", &[JValue::from(&js)])? {
                JValueGen::Void => Ok(()),
                _ => panic!("class file out of sync"),
            }
        })
    }
}

impl LogMessage {
    // But normal methods can use the lifetime of `&self`; this means that we can't support
    // the "push frame" methods of the jvm, but do we need those? When you return a `Local`,
    // it will get freed regardless.
    pub fn level<'jvm>(
        &'jvm self,
        level: impl Into<i32>,
    ) -> jni::errors::Result<Option<Local<'jvm, LogMessage>>> {
        let level: i32 = level.into();

        Jvm::with(|jvm| {
            let mut env = jvm.to_env();
            let this = Anchor::from(self);
            match env.call_method(&this, "level", "(I)Lme/ferris/LogMessage;", &[level.into()])? {
                JValueGen::Object(o) => {
                    if o.is_null() {
                        Ok(None)
                    } else {
                        unsafe { Ok(Some(Local::from_jobject(o))) }
                    }
                }
                _ => panic!("class file out of sync"),
            }
        })
    }
}
