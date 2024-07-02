//@run
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

fn main() -> duchess::Result<()> {
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
