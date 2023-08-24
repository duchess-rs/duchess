package log;

import java.util.Date;

public interface TimeStep<S> {
    S withTime(Date eventTime);
}
