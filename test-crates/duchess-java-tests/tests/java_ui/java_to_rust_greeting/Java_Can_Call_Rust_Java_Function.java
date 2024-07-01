//@check-pass
package java_to_rust_greeting;

public class Java_Can_Call_Rust_Java_Function {
    native String base_greeting(String name);

    public static void main(String[] args) {
        System.loadLibrary("native_fn_callable_from_java");
        Java_Can_Call_Rust_Java_Function sut = new Java_Can_Call_Rust_Java_Function();
        sut.base_greeting("duchess");
    }
}
