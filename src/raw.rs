//! Contains newtypes for pointers vended by [`jni_sys`]. The newtypes must correctly impl Send and Sync (or not) and
//! include correct lifetime bounds, but are otherwise "untyped". The class or interface the Java object they point to
//! must be safely tracked elsewhere.
//!
//! The `'static` pointers generally rely on duchess not deinitializing a JVM after it's already initialized.

use std::{
    ffi::{self},
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::{self, NonNull},
};

use jni_sys::jvalue;

use crate::{jvm::JavaObjectExt, Error, GlobalResult, JavaObject, Local};

const VERSION: jni_sys::jint = jni_sys::JNI_VERSION_1_8;

/// Get a [`JvmPtr`] to an already initialized JVM (if one exists).
///
/// If the `dynlibjvm` feature is enabled and `libjvm` isn't already loaded, it will first force it to be loaded.
///
/// # Safety
///
/// Caller must ensure that no two threads race to call this fn or [`try_create_jvm()`].
pub(crate) unsafe fn existing_jvm() -> GlobalResult<Option<JvmPtr>> {
    let libjvm = crate::libjvm::libjvm_or_load()?;

    let mut jvms = [std::ptr::null_mut::<jni_sys::JavaVM>()];
    let mut num_jvms: jni_sys::jsize = 0;

    let code = unsafe {
        (libjvm.JNI_GetCreatedJavaVMs)(
            jvms.as_mut_ptr(),
            jvms.len().try_into().unwrap(),
            &mut num_jvms as *mut _,
        )
    };
    if code != jni_sys::JNI_OK {
        return Err(Error::JvmInternal(format!(
            "GetCreatedJavaVMs failed with code `{code}`"
        )));
    }

    match num_jvms {
        0 => Ok(None),
        1 => JvmPtr::new(jvms[0])
            .ok_or_else(|| Error::JvmInternal("GetCreatedJavaVMs returned null pointer".into()))
            .map(Some),
        _ => Err(Error::JvmInternal(format!(
            "GetCreatedJavaVMs returned more JVMs than expected: `{num_jvms}`"
        ))),
    }
}

/// Try to initialize a new JVM with the provided `options`, returning a [`JvmPtr`] on success or an
/// [`Error::JvmAlreadyExists`] if one already exists.
///
/// If the `dynlibjvm` feature is enabled and `libjvm` isn't already loaded, it will first force it to be loaded.
///
/// # Safety
///
/// Caller must ensure that no two threads race to call this fn or [`jvm()`].
pub(crate) unsafe fn try_create_jvm<'a>(
    options: impl IntoIterator<Item = String>,
) -> GlobalResult<JvmPtr> {
    let libjvm = crate::libjvm::libjvm_or_load()?;

    let options = options
        .into_iter()
        .map(|opt| ffi::CString::new(opt).unwrap())
        .collect::<Vec<_>>();

    let mut option_ptrs = options
        .iter()
        .map(|opt| jni_sys::JavaVMOption {
            optionString: opt.as_ptr().cast_mut(),
            extraInfo: std::ptr::null_mut(),
        })
        .collect::<Vec<_>>();

    let mut args = jni_sys::JavaVMInitArgs {
        version: VERSION,
        nOptions: options.len().try_into().unwrap(),
        options: option_ptrs.as_mut_ptr(),
        ignoreUnrecognized: jni_sys::JNI_FALSE,
    };

    let mut jvm = std::ptr::null_mut::<jni_sys::JavaVM>();
    let mut env = std::ptr::null_mut::<ffi::c_void>();

    // SAFETY: the C strings pointed to be options are valid and non-null through the end of the call. They're not
    // needed once it returns.
    let code = unsafe {
        (libjvm.JNI_CreateJavaVM)(
            &mut jvm as *mut _,
            &mut env as *mut _,
            &mut args as *mut _ as *mut ffi::c_void,
        )
    };

    match code {
        jni_sys::JNI_OK => {
            let Some(jvm) = JvmPtr::new(jvm) else {
                return Err(Error::JvmInternal(
                    "JNI_CreateJavaVM returned null pointer".into(),
                ));
            };
            // Undo default attaching of current thread like the jni crate does
            unsafe { jvm.detach_thread() }?;
            Ok(jvm)
        }
        jni_sys::JNI_EEXIST => Err(Error::JvmAlreadyExists),
        _ => Err(Error::JvmInternal(format!(
            "CreateJavaVM failed with code `{code}`"
        ))),
    }
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JvmPtr(NonNull<jni_sys::JavaVM>);

impl JvmPtr {
    pub(crate) fn new(ptr: *mut jni_sys::JavaVM) -> Option<Self> {
        NonNull::new(ptr).map(Self)
    }

    /// Returns an [`EnvPtr`] which can be used to invoke JNI methods on the current thread. Will return
    /// `None` if the current thread isn't attached to the JVM.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the `'jvm` lifetime will not live past when the current thread is detached from the
    /// JVM.
    pub(crate) unsafe fn env<'jvm>(self) -> GlobalResult<Option<EnvPtr<'jvm>>> {
        let mut env_ptr = std::ptr::null_mut::<ffi::c_void>();
        match fn_table_call(
            self.0,
            |jvm| jvm.GetEnv,
            |jvm, f| f(jvm, &mut env_ptr as *mut _, VERSION),
        ) {
            jni_sys::JNI_OK => Ok(Some(EnvPtr::new(env_ptr.cast()).unwrap())),
            jni_sys::JNI_EDETACHED => Ok(None),
            code => Err(Error::JvmInternal(format!(
                "GetEnv failed with code `{code}`"
            ))),
        }
    }

    /// Attaches the current thread to the JVM and returns an [`EnvPtr`] that can be used to invoke JNI methods.
    /// Multiple calls on the same thread are idempotent.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the `'jvm` lifetime will not live past when the current thread is detached from the
    /// JVM.
    pub(crate) unsafe fn attach_thread<'jvm>(self) -> GlobalResult<EnvPtr<'jvm>> {
        let mut env_ptr = std::ptr::null_mut::<ffi::c_void>();
        match fn_table_call(
            self.0,
            |jvm| jvm.AttachCurrentThread,
            |jvm, f| {
                f(
                    jvm,
                    &mut env_ptr as *mut _,
                    std::ptr::null_mut(), /* args */
                )
            },
        ) {
            jni_sys::JNI_OK => Ok(EnvPtr::new(env_ptr.cast()).unwrap()),
            code => Err(Error::JvmInternal(format!(
                "AttachCurrentThread failed with code `{code}`"
            ))),
        }
    }

    /// Detaches the current thread from the JVM. Multiple calls on the same thread are idempotent.
    ///
    /// # Safety
    ///
    /// The caller must ensure that no local refs from the current thread are accessible.
    pub(crate) unsafe fn detach_thread(self) -> GlobalResult<()> {
        match fn_table_call(self.0, |jvm| jvm.DetachCurrentThread, |jvm, f| f(jvm)) {
            jni_sys::JNI_OK => Ok(()),
            code => Err(Error::JvmInternal(format!(
                "DetachCurrentThread failed with code `{code}`"
            ))),
        }
    }
}

/// Invokes a JNI function through a virtual table interface
unsafe fn fn_table_call<T, F, R>(
    table_ptr: NonNull<*const T>,
    fn_field: impl FnOnce(&T) -> Option<F>,
    call: impl FnOnce(*mut *const T, F) -> R,
) -> R {
    let fn_field = fn_field(&**table_ptr.as_ptr());
    // SAFETY: We specify VERSION when accessing the JNI interfaces and libjvm promises these fn pointers will be
    // non-null
    let fn_field = fn_field.unwrap_unchecked();
    call(table_ptr.as_ptr(), fn_field)
}

// SAFETY: The JVM pointer is safe to be used by any thread
unsafe impl Send for JvmPtr {}
unsafe impl Sync for JvmPtr {}

/// Points to an attached JNI environment interface for the current thread that is valid through `'jvm`.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct EnvPtr<'jvm> {
    ptr: NonNull<jni_sys::JNIEnv>,
    _marker: PhantomData<&'jvm ()>,
}

impl<'jvm> EnvPtr<'jvm> {
    /// # Safety
    ///
    /// The caller must ensure that the JVM remains attached to the current thread throughout `'jvm`.
    pub(crate) unsafe fn new(ptr: *mut jni_sys::JNIEnv) -> Option<Self> {
        let ptr = NonNull::new(ptr)?;
        Some(Self {
            ptr,
            _marker: PhantomData,
        })
    }

    /// Invoke a JNI method dispatched through a virtual table lookup. Used by codegen to make most JNI calls and so
    /// must be public.
    ///
    /// First invokes [`FromJniValue::from_jni_value()`] before checking for a JVM exception. This allows [`Local`] drop
    /// code to run and safely decrement the ref count before exiting early on an exception.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the [`jni_sys::JNIEnv`] raw pointer is only used for this invocation.
    #[doc(hidden)]
    pub unsafe fn invoke_checked<F, T: FromJniValue<'jvm>>(
        self,
        fn_field: impl FnOnce(&jni_sys::JNINativeInterface_) -> Option<F>,
        call: impl FnOnce(*mut jni_sys::JNIEnv, F) -> T::JniValue,
    ) -> crate::Result<'jvm, T> {
        let value = self.invoke_unchecked(fn_field, call);

        // Even if there was an exception thrown, we still need to free any non-null local ref
        let value = T::from_jni_value(self, value);
        self.check_exception()?;

        Ok(value)
    }

    /// Invoke a JNI method dispatched through a virtual table lookup. Does *not* check for an exception and should
    /// only be used when other mechanisms can prove the absence of an exception (e.g. a non-null return value or a
    /// separate call to [`Self::check_exception()`]).
    ///
    /// # Safety
    ///
    /// The caller must ensure that the [`jni_sys::JNIEnv`] raw pointer is only used for this invocation.
    pub(crate) unsafe fn invoke_unchecked<F, T>(
        self,
        fn_field: impl FnOnce(&jni_sys::JNINativeInterface_) -> Option<F>,
        call: impl FnOnce(*mut jni_sys::JNIEnv, F) -> T,
    ) -> T {
        fn_table_call(self.ptr, fn_field, call)
    }

    /// Loads the JVM pointer from this environment.
    /// Returns Err if there is some sort of error.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the [`jni_sys::JNIEnv`] raw pointer is valid for this invocation.
    pub unsafe fn jvm_ptr(self) -> Result<JvmPtr, ()> {
        let env = self.ptr.as_ptr();
        let get_java_vm = unsafe { (**env).GetJavaVM.ok_or(())? };
        let mut jvm_ptr: MaybeUninit<*mut jni_sys::JavaVM> = MaybeUninit::uninit();
        if get_java_vm(env, jvm_ptr.as_mut_ptr()) != 0 {
            return Err(());
        }
        let jvm_ptr = jvm_ptr.assume_init();
        assert!(!jvm_ptr.is_null());
        JvmPtr::new(jvm_ptr).ok_or(())
    }

    /// Registers native methods on the JVM.
    ///
    /// # Safety
    ///
    /// The `class` and `native_methods` arguments must be valid to supply to the JVM.
    pub unsafe fn register_native_methods(
        self,
        class: ObjectPtr,
        native_methods: &[jni_sys::JNINativeMethod],
    ) -> crate::Result<'jvm, ()> {
        let result: jni_sys::jint = self.invoke_checked(
            |f| f.RegisterNatives,
            |env, register_natives| {
                let nm_ptr = native_methods.as_ptr();
                let nm_len: i32 = native_methods.len() as i32;
                register_natives(env, class.as_ptr(), nm_ptr, nm_len)
            },
        )?;

        if result == 0 {
            Ok(())
        } else {
            Err(crate::Error::JvmInternal(format!(
                "register native methods failed"
            )))
        }
    }

    pub fn check_exception(self) -> crate::Result<'jvm, ()> {
        // SAFETY: we don't hold on to the return env ptr
        let thrown = unsafe { self.invoke_unchecked(|env| env.ExceptionOccurred, |env, f| f(env)) };
        if let Some(thrown) = ObjectPtr::new(thrown) {
            unsafe { self.invoke_unchecked(|env| env.ExceptionClear, |env, f| f(env)) };
            // SAFETY: the ptr returned by ExceptionOccurred is already a local ref and must be an instance of Throwable
            Err(Error::Thrown(unsafe { Local::from_raw(self, thrown) }))
        } else {
            Ok(())
        }
    }
}

/// Points to a live Java object through either a local or global ref.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectPtr(NonNull<jni_sys::_jobject>);

impl ObjectPtr {
    /// Used by codegen to wrap JNI object pointers
    #[doc(hidden)]
    pub fn new(ptr: jni_sys::jobject) -> Option<Self> {
        NonNull::new(ptr).map(Self)
    }

    /// Used by codegen to invoke JNI calls on a Java object.
    #[doc(hidden)]
    pub fn as_ptr(self) -> jni_sys::jobject {
        self.0.as_ptr()
    }

    /// # Safety
    ///
    /// The caller must ensure that the pointed-to object remains live through `'a` and is an instance of `T` (or its
    /// subclasses).
    pub(crate) unsafe fn as_ref<'a, T: JavaObject>(self) -> &'a T {
        // SAFETY: The cast is sound because:
        //
        // 1. A pointer to a suitably aligned `sys::_jobject` should also satisfy Self's alignment
        //    requirement (trait rule #3)
        // 2. Self is a zero-sized type (trait rule #1), so there are no invalid bit patterns to
        //    worry about.
        // 3. Self is a zero-sized type (trait rule #1), so there's no actual memory region that is
        //    subject to the aliasing rules.
        unsafe { self.0.cast().as_ref() }
    }
}

impl From<NonNull<jni_sys::_jobject>> for ObjectPtr {
    fn from(ptr: NonNull<jni_sys::_jobject>) -> Self {
        Self(ptr)
    }
}

// Note: ObjectPtrs are generally not safe to use across threads. Only ObjectPtrs that are global refs are.

#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct MethodPtr(NonNull<jni_sys::_jmethodID>);

impl MethodPtr {
    pub(crate) fn new(ptr: jni_sys::jmethodID) -> Option<Self> {
        NonNull::new(ptr).map(Self)
    }

    #[doc(hidden)]
    pub fn as_ptr(self) -> jni_sys::jmethodID {
        self.0.as_ptr()
    }
}

// The JNI promises method pointers remain valid for as long as the class is loaded and can be shared across threads
unsafe impl Send for MethodPtr {}
unsafe impl Sync for MethodPtr {}

#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct FieldPtr(NonNull<jni_sys::_jfieldID>);

impl FieldPtr {
    pub(crate) fn new(ptr: jni_sys::jfieldID) -> Option<Self> {
        NonNull::new(ptr).map(Self)
    }

    #[doc(hidden)]
    pub fn as_ptr(self) -> jni_sys::jfieldID {
        self.0.as_ptr()
    }
}

// The JNI promises field pointers remain valid for as long as the class is loaded and can be shared across threads
unsafe impl Send for FieldPtr {}
unsafe impl Sync for FieldPtr {}

/// Trait used by codegen to convert into [`jni-sys`] unions.
#[doc(hidden)]
pub trait IntoJniValue {
    fn into_jni_value(self) -> jvalue;
}

impl<T: JavaObject> IntoJniValue for &T {
    fn into_jni_value(self) -> jvalue {
        jvalue {
            l: self.as_raw().as_ptr(),
        }
    }
}

impl<T: JavaObject> IntoJniValue for Option<&T> {
    fn into_jni_value(self) -> jvalue {
        self.map(|v| v.into_jni_value())
            .unwrap_or(jvalue { l: ptr::null_mut() })
    }
}

/// Trait used by codegen to extract the return value of a JNI call.
#[doc(hidden)]
pub trait FromJniValue<'jvm> {
    type JniValue;
    unsafe fn from_jni_value(env: EnvPtr<'jvm>, value: Self::JniValue) -> Self;
}

impl<'jvm, T: JavaObject> FromJniValue<'jvm> for Option<Local<'jvm, T>> {
    type JniValue = jni_sys::jobject;

    unsafe fn from_jni_value(env: EnvPtr<'jvm>, value: Self::JniValue) -> Self {
        // SAFETY: objects returned by JNI calls are already local refs
        ObjectPtr::new(value).map(|obj| unsafe { Local::from_raw(env, obj) })
    }
}

// () is Java `void`
impl<'jvm> FromJniValue<'jvm> for () {
    type JniValue = ();

    unsafe fn from_jni_value(_env: EnvPtr<'jvm>, _value: Self::JniValue) -> Self {
        ()
    }
}

macro_rules! scalar_jni_value {
    ($($rust:ty: $field:ident $java:ident,)*) => {
        $(
            impl IntoJniValue for $rust {
                fn into_jni_value(self) -> jvalue {
                    jvalue {
                        $field: self as jni_sys::$java,
                    }
                }
            }

            impl<'jvm> FromJniValue<'jvm> for $rust {
                type JniValue = jni_sys::$java;

                unsafe fn from_jni_value(_env: EnvPtr<'jvm>, value: Self::JniValue) -> Self {
                    value
                }
            }
        )*
    };
}

scalar_jni_value! {
    // jboolean is u8, need to explicitly map to Rust bool
    // bool: z jboolean,
    i8: b jbyte,
    i16: s jshort,
    u16: c jchar,
    i32: i jint,
    i64: j jlong,
    f32: f jfloat,
    f64: d jdouble,
}

impl IntoJniValue for bool {
    fn into_jni_value(self) -> jvalue {
        jvalue {
            z: self as jni_sys::jboolean,
        }
    }
}

impl<'jvm> FromJniValue<'jvm> for bool {
    type JniValue = jni_sys::jboolean;

    unsafe fn from_jni_value(_env: EnvPtr<'jvm>, value: Self::JniValue) -> Self {
        value == jni_sys::JNI_TRUE
    }
}
