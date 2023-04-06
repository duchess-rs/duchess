use duchess::prelude::*;

duchess::java_package! {
    package me.ferris;

    class Logger { * }
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
