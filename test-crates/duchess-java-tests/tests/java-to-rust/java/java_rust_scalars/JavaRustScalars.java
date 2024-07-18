//@check-pass
package java_rust_scalars;

public class JavaRustScalars {
    native int echoInt(int i);
    native long echoLong(long l);
    native double echoDouble(double d);
    native byte echoByte(byte b);
    native short echoShort(short s);
    native float echoFloat(float f);
    native char echoChar(char c);

    public static void main(String[] args) {
        System.loadLibrary("native_fn_echos_int");
        JavaRustScalars sut = new JavaRustScalars();
        int expectedInt = -123456;
        int i = sut.echoInt(expectedInt);
        if (i != expectedInt) {
            throw new RuntimeException("expected: " + expectedInt + " got: " + i);
        }

        long expectedLong = 99L;
        long l = sut.echoLong(expectedLong);
        if (l != expectedLong) {
            throw new RuntimeException("expected: " + expectedLong + " got: " + l);
        }

        double expectedDouble = 123.4;
        double d = sut.echoDouble(expectedDouble);
        if (d != expectedDouble) {
            throw new RuntimeException("expected: " + expectedDouble + " got: " + d);
        }

        byte expectedByte = 5;
        byte b = sut.echoByte(expectedByte);
        if (b != expectedByte) {
            throw new RuntimeException("expected: " + expectedByte + " got: " + b);
        }

        short expectedShort = 5;
        short s = sut.echoShort(expectedShort);
        if (s != expectedShort) {
            throw new RuntimeException("expected: " + expectedShort + " got: " + s);
        }

        float expectedFloat = 5.5f;
        float f = sut.echoFloat(expectedFloat);
        if (f != expectedFloat) {
            throw new RuntimeException("expected: " + expectedFloat + " got: " + f);
        }

        char expectedChar = 'a';
        char c = sut.echoChar(expectedChar);
        if (c != expectedChar) {
            throw new RuntimeException("expected: " + expectedChar + " got: " + c);
        }
    }
}
