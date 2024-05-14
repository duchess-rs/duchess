package generics;

public enum EventKey implements MapKey {
    A("abc"),
    B("cde");

    private final String name;

    private EventKey(String name) {
        this.name = name;
    }

}