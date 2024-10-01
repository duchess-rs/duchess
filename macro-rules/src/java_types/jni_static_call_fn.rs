/// Closure that selects the appropriate JNI method to call based on its return type.
///
/// # Examples
///
/// * `byte` expands to `|env| env.CallStaticByteMethodA`
#[macro_export]
macro_rules! jni_static_call_fn {
    (byte) => {
        |env| env.CallStaticByteMethodA
    };
    (short) => {
        |env| env.CallStaticShortMethodA
    };
    (int) => {
        |env| env.CallStaticIntMethodA
    };
    (long) => {
        |env| env.CallStaticLongMethodA
    };
    (float) => {
        |env| env.CallStaticFloatMethodA
    };
    (double) => {
        |env| env.CallStaticDoubleMethodA
    };
    (char) => {
        |env| env.CallStaticCharMethodA
    };
    (boolean) => {
        |env| env.CallStaticBooleanMethodA
    };
    (void) => {
        |env| env.CallStaticVoidMethodA
    };

    // Reference types
    ($r:tt) => {
        |env| env.CallStaticObjectMethodA
    };
}
