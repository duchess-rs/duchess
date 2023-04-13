use duchess::prelude::*;

duchess::java_package! {
    package me.ferris;

    class Logger { * }
}

fn main() -> duchess::GlobalResult<()> {
    use crate::me::ferris::LoggerExt;
    use duchess::java::lang::ThrowableExt;
    duchess::Jvm::with(|jvm| {
        let l = me::ferris::Logger::new().execute(jvm)?;
        l.log_int(22).execute(jvm)?;
        l.log_string("Hello, Duchess!").execute(jvm)?;

        me::ferris::Logger::new()
            .inspect(|l| l.log_int(23))
            .inspect(|l| l.log_string("Hello again, Duchess!"))
            .execute(jvm)?;

        l.throw_something().catch(|t| t.print_stack_trace()).execute(jvm)?;
        println!("all good, though!");

        let res = me::ferris::Logger::new().try_downcast::<_, me::ferris::Logger>().execute(jvm)?;
        assert!(res.is_ok());

        Ok(())
    })
}
