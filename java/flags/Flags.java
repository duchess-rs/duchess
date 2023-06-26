package flags;

public class Flags {
    private int privateField;

    private int privateMethod() {
        return privateField;
    }

    public int publicMethod() {
        return privateMethod();
    }
}
