package native_void;

public class NativeVoid {
    public void fire(String name) {
        record(name);
    }

    native void record(String name);

    public static void fireStatic(String name) {
        recordStatic(name);
    }

    static native void recordStatic(String name);
}
