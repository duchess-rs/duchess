use duchess::{java, prelude::*};

duchess::java_package! {
    package log;

    class BaseEvent { * }

    public class log.Builder implements log.TimeStep<log.NameStep>, log.NameStep, log.BuildStep {
        java.lang.String n;
        java.util.Date d;
        public log.Builder();
        public log.Event build();
        public log.BuildStep withName(java.lang.String);
        public log.NameStep withTime(java.util.Date);

        // FIXME: java generates two `withTime` methods, so we have to specify this
        // fully and comment out one of them. Not sure if this is avoidable
        // given the way Java does things right now. Note the distinct return types!
        //
        // public java.lang.Object withTime(java.util.Date);
    }

    class Event { * }
    class Logger { * }
    class NameStep { * }
    class TimeStep { * }
    class BuildStep { * }
}

#[test]
fn one_big_call() -> duchess::GlobalResult<()> {
    // FIXME: conflict between interface trait (LoggerExt) and class trait (BuilderExt)

    log::Logger::new()
        .add_event(
            log::Event::builder()
                .with_time(java::util::Date::new())
                .with_name("foo")
                .build(),
        )
        .execute()?;

    Ok(())
}

#[test]
fn local_ref_and_two_calls() -> duchess::GlobalResult<()> {
    // FIXME: conflict between interface trait (LoggerExt) and class trait (BuilderExt)

    duchess::Jvm::with(|jvm| {
        let logger = log::Logger::new().execute_with(jvm)?;
        let event = log::Event::builder()
            .with_time(java::util::Date::new())
            .with_name("foo")
            .build()
            .execute_with(jvm)?;
        logger.add_event(&event).execute_with(jvm)?;
        logger.add_event(&event).execute_with(jvm)?;
        Ok(())
    })
}

#[test]
fn global_ref_and_two_calls() -> duchess::GlobalResult<()> {
    // FIXME: conflict between interface trait (LoggerExt) and class trait (BuilderExt)

    let logger = log::Logger::new().global().execute()?;

    duchess::Jvm::with(|jvm| {
        let event = log::Event::builder()
            .with_time(java::util::Date::new())
            .with_name("foo")
            .build()
            .execute_with(jvm)?;
        logger.add_event(&event).execute_with(jvm)?;
        logger.add_event(&event).execute_with(jvm)?;
        Ok(())
    })
}

#[test]
fn global_ref_and_chained_calls() -> duchess::GlobalResult<()> {
    // FIXME: conflict between interface trait (LoggerExt) and class trait (BuilderExt)

    let logger = log::Logger::new().global().execute()?;

    logger
        .add_event(
            log::Event::builder()
                .with_time(java::util::Date::new())
                .with_name("foo")
                .build(),
        )
        .execute()?;

    Ok(())
}

fn main() -> duchess::GlobalResult<()> {
    let logger = log::Logger::new().global().execute()?;

    logger
        .add_event(
            log::Event::builder()
                .with_time(java::util::Date::new())
                .with_name("foo")
                .build(),
        )
        .execute()?;

    Ok(())
}
