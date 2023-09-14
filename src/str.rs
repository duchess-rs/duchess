use std::ffi::{c_char, CString};

use crate::{
    into_rust::IntoRust, java::lang::String as JavaString, jvm::JavaObjectExt, Error, Jvm, JvmOp,
    Local,
};

impl JvmOp for &str {
    type Output<'jvm> = Local<'jvm, JavaString>;

    fn execute_with<'jvm>(
        self,
        jvm: &mut Jvm<'jvm>,
    ) -> crate::Result<'jvm, Local<'jvm, JavaString>> {
        let encoded = cesu8::to_java_cesu8(self);
        // SAFETY: cesu8 encodes interior nul bytes as 0xC080
        let c_string = unsafe { CString::from_vec_unchecked(encoded.into_owned()) };

        let env = jvm.env();
        // SAFETY: c_string is non-null pointer to cesu8-encoded encoded string ending in a trailing nul byte
        let string: Option<Local<JavaString>> = unsafe {
            env.invoke_checked(|env| env.NewStringUTF, |env, f| f(env, c_string.as_ptr()))
        }?;
        string.ok_or_else(|| Error::JvmInternal("JVM faild to create new String".into()))
    }
}

impl JvmOp for &String {
    type Output<'jvm> = Local<'jvm, JavaString>;

    fn execute_with<'jvm>(
        self,
        jvm: &mut Jvm<'jvm>,
    ) -> crate::Result<'jvm, Local<'jvm, JavaString>> {
        <&str as JvmOp>::execute_with(&self, jvm)
    }
}

impl IntoRust<String> for &JavaString {
    fn into_rust<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, String> {
        let str_raw = self.as_raw();

        let env = jvm.env();

        // Note: we need to pull both the Modified UTF-8 length and the UTF-16 length of the string. The indexes to
        // GetStringUTFRegion are UTF-16, while the space we need to allocate for the output pointer is the Modified
        // UTF-8 length!

        // SAFETY: J::Output impls AsRef<JavaString>, so we know str_raw points to a non-null Java String
        let cesu8_len = unsafe {
            env.invoke_unchecked(
                |env| env.GetStringUTFLength,
                |env, f| f(env, str_raw.as_ptr()),
            )
        };
        // Shortcut for common case of empty strings. This also avoids us trying to write
        // to the null ptr of an empty Vec
        if cesu8_len == 0 {
            return Ok(String::new());
        }
        // java uses signed lengths
        assert!(cesu8_len > 0);

        // SAFETY: same as for cesu8_len
        let utf16_len = unsafe {
            env.invoke_unchecked(|env| env.GetStringLength, |env, f| f(env, str_raw.as_ptr()))
        };
        assert!(utf16_len > 0);

        let mut cesu_bytes =
            Vec::<u8>::with_capacity(cesu8_len as usize + 1 /* JNI appends trailing nul */);
        // SAFETY: cesu_bytes is a non-null pointer with enough capacity for the entire string when encoded in Modified
        // UTF-8 (with a trailing nul byte).
        unsafe {
            env.invoke_unchecked(
                |env| env.GetStringUTFRegion,
                |env, f| {
                    f(
                        env,
                        str_raw.as_ptr(),
                        0,
                        utf16_len,
                        cesu_bytes.as_mut_ptr().cast::<c_char>(),
                    )
                },
            );
            cesu_bytes.set_len(cesu8_len as usize); // ignore trailing nul
        };

        // In the common case where there are no surrogate bytes, we can do a (checked) conversion of the Vec into a
        // Rust String. Otherwise, we'll need to use the cesu8 crate to convert properly. Note that this is the same
        // first check done by cesu8, but because the interface takes an &[u8], it would force a copy.
        let decoded = match String::from_utf8(cesu_bytes) {
            Ok(s) => s,
            Err(err) => cesu8::from_java_cesu8(err.as_bytes())
                .map_err(|e| {
                    Error::JvmInternal(format!(
                        "Java String contained invalid Modified UTF-8: {}",
                        e
                    ))
                })?
                .into_owned(),
        };

        Ok(decoded)
    }
}
