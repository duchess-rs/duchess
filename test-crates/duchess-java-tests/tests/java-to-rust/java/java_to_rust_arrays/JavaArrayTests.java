//@check-pass
package java_to_rust_arrays;

import java.util.Arrays;

public class JavaArrayTests {
    public static native long combine_bytes(byte[] bytes);
    public static native byte[] break_bytes(long num);

    public static void main(String[] args) {
        System.loadLibrary("native_fn_arrays");
        byte[] bytes = {1, 1, 1, 1, 1, 1, 1, 1};
        long combo = JavaArrayTests.combine_bytes(bytes);
        if (combo != 72340172838076673L) {
            throw new RuntimeException("expected: 72340172838076673 got: " + combo);
        }
        byte[] broken_combo = JavaArrayTests.break_bytes(combo);
        if (!Arrays.equals(broken_combo, bytes)) {
            throw new RuntimeException("expected: " + Arrays.toString(bytes) + " got: " + Arrays.toString(broken_combo));
        }
    }
}
