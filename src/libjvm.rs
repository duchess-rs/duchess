use crate::GlobalResult;

/// Virtual table for top-level libjvm functions that create or get JVMs.
#[allow(non_snake_case)]
pub(crate) struct Libjvm {
    pub JNI_CreateJavaVM: unsafe extern "system" fn(
        pvm: *mut *mut jni_sys::JavaVM,
        penv: *mut *mut std::ffi::c_void,
        args: *mut std::ffi::c_void,
    ) -> jni_sys::jint,
    pub JNI_GetCreatedJavaVMs: unsafe extern "system" fn(
        vmBuf: *mut *mut jni_sys::JavaVM,
        bufLen: jni_sys::jsize,
        nVMs: *mut jni_sys::jsize,
    ) -> jni_sys::jint,
}

#[cfg(feature = "dylibjvm")]
mod dynlib {
    use std::path::{Path, PathBuf};

    use libloading::Library;
    use once_cell::sync::OnceCell;

    use super::*;
    use crate::Error;

    static LIBJVM: OnceCell<Libjvm> = OnceCell::new();

    #[allow(non_snake_case)]
    fn load_libjvm_at(path: &Path) -> GlobalResult<Libjvm> {
        (|| {
            let lib = unsafe { Library::new(path) }?;
            let JNI_CreateJavaVM = *unsafe { lib.get(b"JNI_CreateJavaVM\0") }?;
            let JNI_GetCreatedJavaVMs = *unsafe { lib.get(b"JNI_GetCreatedJavaVMs\0") }?;
            Ok(Libjvm {
                JNI_CreateJavaVM,
                JNI_GetCreatedJavaVMs,
            })
        })()
        .map_err(|e: libloading::Error| Error::UnableToLoadLibjvm(Box::new(e)))
    }

    pub(crate) fn libjvm_or_load() -> GlobalResult<&'static Libjvm> {
        LIBJVM.get_or_try_init(|| {
            let path: PathBuf = [
                &java_locator::locate_jvm_dyn_library()
                    .map_err(|e| Error::UnableToLoadLibjvm(Box::new(e)))?,
                java_locator::get_jvm_dyn_lib_file_name(),
            ]
            .into_iter()
            .collect();

            load_libjvm_at(&path)
        })
    }

    pub(crate) fn libjvm_or_load_at(path: &Path) -> GlobalResult<&'static Libjvm> {
        LIBJVM.get_or_try_init(|| load_libjvm_at(path))
    }
}

#[cfg(feature = "dylibjvm")]
pub(crate) use dynlib::{libjvm_or_load, libjvm_or_load_at};

#[cfg(not(feature = "dylibjvm"))]
pub(crate) fn libjvm_or_load() -> GlobalResult<&'static Libjvm> {
    static LIBJVM: Libjvm = Libjvm {
        JNI_CreateJavaVM: jni_sys::JNI_CreateJavaVM,
        JNI_GetCreatedJavaVMs: jni_sys::JNI_GetCreatedJavaVMs,
    };

    Ok(&LIBJVM)
}
