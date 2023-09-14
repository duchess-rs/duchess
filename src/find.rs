use std::ffi::CStr;

use crate::{
    java,
    jvm::JavaObjectExt,
    raw::{FieldPtr, MethodPtr},
    Jvm, Local, Result,
};

pub fn find_class<'jvm>(
    jvm: &mut Jvm<'jvm>,
    jni_name: &CStr,
) -> Result<'jvm, Local<'jvm, java::lang::Class>> {
    let class: Option<Local<java::lang::Class>> = unsafe {
        // SAFETY: jni_name is a valid pointer to a nul-terminated byte string
        jvm.env()
            .invoke_checked(|env| env.FindClass, |env, f| f(env, jni_name.as_ptr()))
    }?;
    class.ok_or_else(|| {
        // Class not existing should've triggered NoClassDefFoundError so something strange is now happening
        crate::Error::JvmInternal(format!(
            "failed to find class `{}`",
            jni_name.to_string_lossy()
        ))
    })
}

pub fn find_method<'jvm>(
    jvm: &mut Jvm<'jvm>,
    class: impl AsRef<java::lang::Class>,
    jni_name: &CStr,
    jni_descriptor: &CStr,
    is_static: bool,
) -> Result<'jvm, MethodPtr> {
    let class = class.as_ref().as_raw();

    let env = jvm.env();
    let method = unsafe {
        env.invoke_unchecked(
            |env| {
                if is_static {
                    env.GetStaticMethodID
                } else {
                    env.GetMethodID
                }
            },
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
    // JVM guarantees that valid method IDs are non-null, so the null check here suffices
    if let Some(method) = MethodPtr::new(method) {
        Ok(method)
    } else {
        jvm.env().check_exception()?;
        // Method not existing should've triggered NoSuchMethodError so something strange is now happening
        Err(crate::Error::JvmInternal(format!(
            "failed to find method `{}` with signature `{}`",
            jni_name.to_string_lossy(),
            jni_descriptor.to_string_lossy(),
        )))
    }
}

pub fn find_field<'jvm>(
    jvm: &mut Jvm<'jvm>,
    class: impl AsRef<java::lang::Class>,
    jni_name: &CStr,
    jni_descriptor: &CStr,
    is_static: bool,
) -> Result<'jvm, FieldPtr> {
    let class = class.as_ref().as_raw();

    let env = jvm.env();
    let field = unsafe {
        env.invoke_unchecked(
            |env| {
                if is_static {
                    env.GetStaticFieldID
                } else {
                    env.GetFieldID
                }
            },
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
    // JVM guarantees that valid field IDs are non-null, so the null check here suffices
    if let Some(field) = FieldPtr::new(field) {
        Ok(field)
    } else {
        jvm.env().check_exception()?;
        // Field not existing should've triggered NoSuchFieldError so something strange is now happening
        Err(crate::Error::JvmInternal(format!(
            "failed to find field `{}` with signature `{}`",
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
    find_method(jvm, class, METHOD_NAME, jni_descriptor, false)
}
