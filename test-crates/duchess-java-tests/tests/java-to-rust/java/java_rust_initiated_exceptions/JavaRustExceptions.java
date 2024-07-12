//@check-pass

package java_rust_initiated_exceptions;

public class JavaRustExceptions {
    native String raiseNPE();
    native String raiseSliceTooLong();
    native String raiseJvmInternal();
    native String panic();

    public static void expectNPE(JavaRustExceptions test) {
        try {
            test.raiseNPE();
        } catch (NullPointerException e) {
            return;
        }

        throw new RuntimeException("NullPointerException not caught");
    }

    public static void expectSliceTooLong(JavaRustExceptions test) {
        String message = "no exception thrown";
        try {
            test.raiseSliceTooLong();
        } catch (RuntimeException e) {
            message = e.getMessage();
        }

        if (!message.contains("slice was too long")) {
            throw new RuntimeException("Caught no exception or the wrong exception: " + message);
        }
    }

    public static void expectJvmInternal(JavaRustExceptions test) {
        String message = "no exception thrown";
        try {
            test.raiseJvmInternal();
        } catch (RuntimeException e) {
            message = e.getMessage();
        }

        if (!message.equals("JvmInternal")) {
            throw new RuntimeException("Caught no exception or the wrong exception: " + message);
        }
    }

    public static void expectPanicIsRuntimeException(JavaRustExceptions test) {
        String message = "no exception thrown";
        try {
            test.panic();
        } catch (RuntimeException e) {
            message = e.getMessage();
        }

        if (!message.equals("RUST PANIC!")) {
            throw new RuntimeException("Caught no exception or the wrong exception: " + message);
        }
    }

    public static void main(String[] args) {
        System.loadLibrary("java_rust_initiated_exceptions");
        JavaRustExceptions test = new JavaRustExceptions();

        expectNPE(test);
        expectSliceTooLong(test);
        expectJvmInternal(test);
        expectPanicIsRuntimeException(test);
    }

}
