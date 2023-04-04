use duchess::prelude::*;

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
    use crate::me::ferris::LoggerExt;
    duchess::Jvm::with(|jvm| {
        let l = me::ferris::Logger::new().execute(jvm)?;
        l.log_int(22).execute(jvm)?;
        l.log_string("Hello, Duchess!").execute(jvm)?;

        me::ferris::Logger::new()
            .inspect(|l| l.log_int(23))
            .inspect(|l| l.log_string("Hello again, Duchess!"))
            .execute(jvm)?;

        Ok(())
    })
}
