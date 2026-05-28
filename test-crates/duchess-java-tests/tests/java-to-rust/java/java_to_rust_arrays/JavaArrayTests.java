//@check-pass
package java_to_rust_arrays;

import java.util.Arrays;

public class JavaArrayTests {
    public static native long combine_bytes(byte[] bytes);
    public static native byte[] break_bytes(long num);
    public static native long fillWithOnes(byte[] arr, int len);
    public static native long fillWithTrue(boolean[] arr, int len);

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

        byte[] arr = new byte[5];
        for (int i = 0; i < arr.length; i++) {
            if (arr[i] != 0) {
                throw new RuntimeException("Array not initialized to 0 at index " + i);
            }
        }
        fillWithOnes(arr, arr.length);
        for (int i = 0; i < arr.length; i++) {
            if (arr[i] != 1) {
                throw new RuntimeException("Rust did not set all elements to 1 at index " + i);
            }
        }
        boolean[] b_arr = new boolean[5];
        for (int i = 0; i < b_arr.length; i++) {
            if (b_arr[i] != false) {
                throw new RuntimeException("Array not initialized to false at index " + i);
            }
        }
        fillWithTrue(b_arr, arr.length);
        for (int i = 0; i < b_arr.length; i++) {
            if (b_arr[i] != true) {
                throw new RuntimeException("Rust did not set all elements to true at index " + i);
            }
        }
    }
}
