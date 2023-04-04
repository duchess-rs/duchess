use duchess::{java, prelude::*, Jvm, JvmOp};
use jni::objects::JValue;

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
