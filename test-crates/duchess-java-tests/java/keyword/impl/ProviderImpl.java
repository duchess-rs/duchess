package keyword.impl;

import keyword.api.Provider;

public class ProviderImpl implements Provider {
    private static int started = 0;

    public ProviderImpl() {}

    public void start() {
        started++;
    }

    public static int count() {
        return started;
    }
}
