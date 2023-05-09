use std::ffi::CStr;

use crate::{
    java,
    jvm::JavaObjectExt,
    plumbing::{check_exception, HasEnvPtr},
    raw::{MethodPtr, ObjectPtr},
    Jvm, Local, Result,
};

pub fn find_class<'jvm>(
    jvm: &mut Jvm<'jvm>,
    jni_name: &CStr,
) -> Result<'jvm, Local<'jvm, java::lang::Class>> {
    let env = jvm.env();
    let class = unsafe { env.invoke(|env| env.FindClass, |env, f| f(env, jni_name.as_ptr())) };
    if let Some(class) = ObjectPtr::new(class) {
        Ok(unsafe { Local::from_raw(env, class) })
    } else {
        check_exception(jvm)?;
        // Class not existing should've triggered NoClassDefFoundError so something strange is now happening
        Err(crate::Error::JvmInternal(format!(
            "failed to find class `{}`",
            jni_name.to_string_lossy()
        )))
    }
}

pub fn find_method<'jvm>(
    jvm: &mut Jvm<'jvm>,
    class: impl AsRef<java::lang::Class>,
    jni_name: &CStr,
    jni_descriptor: &CStr,
) -> Result<'jvm, MethodPtr> {
    let class = class.as_ref().as_raw();

    let env = jvm.env();
    let method = unsafe {
        env.invoke(
            |env| env.GetMethodID,
            |env, f| {
                f(
                    env,
                    class.as_ptr(),
                    jni_name.as_ptr(),
                    jni_descriptor.as_ptr(),
                )
            },
        )
    };
    if let Some(method) = MethodPtr::new(method) {
        Ok(method)
    } else {
        check_exception(jvm)?;
        // Method not existing should've triggered NoSuchMethodError so something strange is now happening
        Err(crate::Error::JvmInternal(format!(
            "failed to find method `{}` with signature `{}`",
            jni_name.to_string_lossy(),
            jni_descriptor.to_string_lossy(),
        )))
    }
}

pub fn find_constructor<'jvm>(
    jvm: &mut Jvm<'jvm>,
    class: impl AsRef<java::lang::Class>,
    jni_descriptor: &CStr,
) -> Result<'jvm, MethodPtr> {
    const METHOD_NAME: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"<init>\0") };
    find_method(jvm, class, METHOD_NAME, jni_descriptor)
}
