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

fn run_logger() -> duchess::GlobalResult<()> {
    // FIXME: conflict between interface trait (LoggerExt) and class trait (BuilderExt)
    use crate::log::BuildStepExt;
    use crate::log::LoggerExt;
    use crate::log::NameStepExt;
    use crate::log::TimeStepExt;

    duchess::Jvm::with(|jvm| {
        let logger = log::Logger::new().execute(jvm)?;
        let event = log::Builder::new()
            .with_time(java::util::Date::new())
            .assert_not_null()
            .with_name("foo")
            .assert_not_null()
            .build()
            .assert_not_null()
            .execute(jvm)?;
        logger.add_event(&event).execute(jvm)?;
        Ok(())
    })
}

fn main() -> duchess::GlobalResult<()> {
    run_logger()
}

#[test]
fn test() -> duchess::GlobalResult<()> {
    run_logger()
}
