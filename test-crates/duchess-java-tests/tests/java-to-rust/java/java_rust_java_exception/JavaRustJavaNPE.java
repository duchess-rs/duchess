//@check-pass

package java_rust_java_exception;

public class JavaRustJavaNPE {
    native String rustFunction();

    public String javaFunction() {
        throw new NullPointerException();
    }

    public static void main(String[] args) {
        System.loadLibrary("java_rust_java_npe");
        JavaRustJavaNPE test = new JavaRustJavaNPE();
        try {
            test.rustFunction();
        } catch (NullPointerException e) {
            return;
        }

        throw new RuntimeException("Did not catch NullPointerException");
    }

}
