use jni::{
    objects::{JValue, JValueGen},
    strings::JNIString,
};

use crate::jvm::{Anchor, JavaObject, Jvm, Local};

pub struct Logger {
    dummy: (),
}

unsafe impl JavaObject for Logger {}

pub struct LogMessage {
    dummy: (),
}

unsafe impl JavaObject for LogMessage {}

impl Logger {
    pub fn new<'jvm>(jvm: &'jvm Jvm) -> jni::errors::Result<Local<'jvm, Self>> {
        let mut env = jvm.to_env();

        // FIXME: how do we cache this
        let class = env.find_class("me/ferris/Logger")?;

        env.new_object(class, "()", &[])
            .map(|o| unsafe { Local::from_jobject(jvm, o) })
    }

    pub fn global_logger<'jvm>(jvm: &'jvm Jvm) -> jni::errors::Result<Local<'jvm, Self>> {
        let mut env = jvm.to_env();

        // FIXME: how do we cache this
        let class = env.find_class("me/ferris/Logger")?;

        match env.call_static_method(class, "globalLogger", "()Lme/ferris/Logger;", &[])? {
            JValueGen::Object(o) => unsafe { Ok(Local::from_jobject(jvm, o)) },
            _ => panic!("class file out of sync"),
        }
    }

    pub fn log(&self, s: impl Into<JNIString>) -> jni::errors::Result<()> {
        // FIXME: can we do better than this `impl Into` business, it feels inefficient
        Jvm::with(|jvm| {
            let env = jvm.to_env();
            let this = Anchor::from(self);
            let js:  = env.new_string(s)?;
            match env.call_method(&this, "log", "(Ljava/lang/String;)V", &[JValue::from(&js)])? {
                JValueGen::Void => Ok(()),
                _ => panic!("class file out of sync"),
            }
        })
    }
}
