/// Closure that selects the appropriate JNI method to call based on its return type.
///
/// # Examples
///
/// * `byte` expands to `|env| env.CallByteMethodA`
#[macro_export]
macro_rules! jni_call_fn {
    (byte) => {
        |env| env.CallByteMethodA
    };
    (short) => {
        |env| env.CallShortMethodA
    };
    (int) => {
        |env| env.CallIntMethodA
    };
    (long) => {
        |env| env.CallLongMethodA
    };
    (float) => {
        |env| env.CallFloatMethodA
    };
    (double) => {
        |env| env.CallDoubleMethodA
    };
    (char) => {
        |env| env.CallCharMethodA
    };
    (boolean) => {
        |env| env.CallBooleanMethodA
    };
    (void) => {
        |env| env.CallVoidMethodA
    };

    // Reference types
    ($r:tt) => {
        |env| env.CallObjectMethodA
    };
}
