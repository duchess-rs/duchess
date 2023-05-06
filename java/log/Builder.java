package log;

import java.util.Date;

public class Builder
        implements TimeStep<NameStep>, NameStep, BuildStep {

    String n;
    Date d;

    // FIXME: support static methods
    Builder() {
    }

    @Override
    public Event build() {
        return new Event(n, d);
    }

    @Override
    public BuildStep withName(String name) {
        n = name;
        return this;
    }

    @Override
    public NameStep withTime(Date eventTime) {
        d = eventTime;
        return this;
    }

}
