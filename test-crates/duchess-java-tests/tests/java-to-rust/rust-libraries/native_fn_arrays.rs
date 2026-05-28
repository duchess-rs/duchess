//@check-pass

use duchess::java::ArrayModificationExt;
use duchess::{java, Java, JvmOp, ToJava};

duchess::java_package! {
    package java_to_rust_arrays;

    public class JavaArrayTests {
        public static native long combine_bytes(byte[]);
        public static native byte[] break_bytes(long);
        public static native long fillWithOnes(byte[], int);
        public static native long fillWithTrue(boolean[], int);
    }
}

#[duchess::java_function(java_to_rust_arrays.JavaArrayTests::combine_bytes)]
fn combine_bytes(bytes: Option<&duchess::java::Array<i8>>) -> i64 {
    let signed_bytes: &[i8] = &*bytes.assert_not_null().execute::<Vec<i8>>().unwrap();
    let unsigned_bytes = signed_bytes.iter().map(|x| *x as u8).collect::<Vec<u8>>();
    return i64::from_le_bytes(unsigned_bytes.try_into().unwrap());
}

#[duchess::java_function(java_to_rust_arrays.JavaArrayTests::break_bytes)]
fn break_bytes(num: i64) -> duchess::Result<Java<java::Array<i8>>> {
    let unsigned_bytes = num.to_le_bytes();
    let signed_bytes = unsigned_bytes.iter().map(|x| *x as i8).collect::<Vec<i8>>();
    let java_array: Java<java::Array<i8>> = signed_bytes.to_java().execute()?.unwrap();

    return Ok(java_array);
}

#[duchess::java_function(java_to_rust_arrays.JavaArrayTests::fillWithOnes)]
fn fill_with_ones(arr: Option<&mut duchess::java::Array<i8>>, len: i32) -> i64 {
    let region: Vec<i8> = vec![1; len as usize];
    arr.unwrap().set_array_region(0, &region).execute();
    0
}

#[duchess::java_function(java_to_rust_arrays.JavaArrayTests::fillWithTrue)]
fn fill_with_true(arr: Option<&mut duchess::java::Array<bool>>, len: i32) -> i64 {
    let region: Vec<bool> = vec![true; len as usize];
    arr.unwrap().set_array_region(0, &region).execute();
    0
}
