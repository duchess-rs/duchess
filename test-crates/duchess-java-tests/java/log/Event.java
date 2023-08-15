package log;

import java.util.Date;

public record Event(
        String name,
        Date eventTime) {

    public static TimeStep<NameStep> builder() {
        return new Builder();
    }
}