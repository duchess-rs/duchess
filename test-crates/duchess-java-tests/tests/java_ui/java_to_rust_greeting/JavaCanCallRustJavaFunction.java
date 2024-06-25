//@check-pass
package java_to_rust_greeting;

public class JavaCanCallRustJavaFunction {
    native String baseGreeting(String name);

    public static void main(String[] args) {
        System.loadLibrary("native_fn_callable_from_java");
        JavaCanCallRustJavaFunction sut = new JavaCanCallRustJavaFunction();
        sut.baseGreeting("duchess");
    }
}
