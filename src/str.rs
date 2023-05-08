use std::ffi::CString;

use crate::{
    error::check_exception, java::lang::String as JavaString, jvm::JavaObjectExt,
    plumbing::HasEnvPtr, raw::ObjectPtr, to_rust::ToRust, Error, Jvm, JvmOp, Local,
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
        let string =
            unsafe { env.invoke(|env| env.NewStringUTF, |env, f| f(env, c_string.as_ptr())) };
        if let Some(string) = ObjectPtr::new(string) {
            Ok(unsafe { Local::from_raw(env, string) })
        } else {
            check_exception(jvm)?; // likely threw an OutOfMemoryError
            Err(Error::JvmInternal("JVM failed to create new String".into()))
        }
    }
}

impl JvmOp for String {
    type Output<'jvm> = Local<'jvm, JavaString>;

    fn execute_with<'jvm>(
        self,
        jvm: &mut Jvm<'jvm>,
    ) -> crate::Result<'jvm, Local<'jvm, JavaString>> {
        <&str as JvmOp>::execute_with(&self, jvm)
    }
}

impl ToRust<String> for JavaString {
    fn to_rust<'jvm>(&self, jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, String> {
        let str_raw = self.as_raw();

        let env = jvm.env();

        // Note: we need to pull both the Modified UTF-8 length and the UTF-16 length of the string. The indexes to
        // GetStringUTFRegion are UTF-16, while the space we need to allocate for the output pointer is the Modified
        // UTF-8 length!

        // SAFETY: J::Output impls AsRef<JavaString>, so we know str_raw points to a non-null Java String
        let cesu8_len = unsafe {
            env.invoke(
                |env| env.GetStringUTFLength,
                |env, f| f(env, str_raw.as_ptr()),
            )
        };
        assert!(cesu8_len >= 0);

        // SAFETY: same as for cesu8_len
        let utf16_len =
            unsafe { env.invoke(|env| env.GetStringLength, |env, f| f(env, str_raw.as_ptr())) };
        assert!(utf16_len >= 0);

        let mut cesu_bytes = Vec::<u8>::with_capacity(cesu8_len as usize);
        // SAFETY: cesu_bytes is a non-null pointer with enough capacity for the entire string when encoded in Modified
        // UTF-8 (with no trailing nul byte).
        unsafe {
            env.invoke(
                |env| env.GetStringUTFRegion,
                |env, f| {
                    f(
                        env,
                        str_raw.as_ptr(),
                        0,
                        utf16_len,
                        cesu_bytes.as_mut_ptr().cast::<i8>(),
                    )
                },
            );
            cesu_bytes.set_len(cesu8_len as usize);
        };
        check_exception(jvm)?;

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
