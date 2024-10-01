/// Generates a closure that selects the appropriate JNI method
/// to call to get a static field based on the field type.
///
/// # Examples
///
/// * `byte` expands to `|env| env.GetStaticByteField`
#[macro_export]
macro_rules! jni_static_field_get_fn {
    (byte) => {
        |env| env.GetStaticByteField
    };
    (short) => {
        |env| env.GetStaticShortField
    };
    (int) => {
        |env| env.GetStaticIntField
    };
    (long) => {
        |env| env.GetStaticLongField
    };
    (float) => {
        |env| env.GetStaticFloatField
    };
    (double) => {
        |env| env.GetStaticDoubleField
    };
    (char) => {
        |env| env.GetStaticCharField
    };
    (boolean) => {
        |env| env.GetStaticBooleanField
    };

    // Reference types
    ($r:tt) => {
        |env| env.GetStaticObjectField
    };
}
