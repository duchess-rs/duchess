//@check-pass

package java_rust_java_exception;

public class JavaRustJavaException {
    native String rustFunction();

    public String javaFunction() {
        throw new RuntimeException("Exception from `javaFunction`");
    }

    public static void main(String[] args) {
        System.loadLibrary("java_rust_java_exception");
        JavaRustJavaException test = new JavaRustJavaException();
        String message = "no exception thrown";
        try {
            test.rustFunction();
        } catch (RuntimeException e) {
            message = e.getMessage();
        }

        if (message != "Exception from `javaFunction`") {
            throw new RuntimeException("Caught no exception or the wrong exception: " + message);
        }
    }

}
