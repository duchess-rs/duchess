use duchess::{java, Java, JvmOp, ToJava};

macro_rules! test_array {
    ($type: ty, $item: expr) => {
        for test_array in [vec![$item], vec![], vec![$item, $item, $item]] {
            let java: Java<java::Array<$type>> =
                test_array.to_java().assert_not_null().execute().unwrap();
            let and_back: Vec<_> = (&*java).execute().unwrap();
            assert_eq!(test_array, and_back);
        }
    };
}

#[test]
fn test_array_types() {
    test_array!(bool, true);
    test_array!(i8, 5_i8);
    test_array!(u16, 5_u16);
    test_array!(i16, 5_i16);
    test_array!(i32, 5_i32);
    test_array!(i64, 5_i64);
    test_array!(f32, 5_f32);
    test_array!(f64, 5_f64);
}
