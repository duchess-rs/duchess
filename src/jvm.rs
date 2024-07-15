use crate::{
    cast::{AsUpcast, TryDowncast, Upcast},
    find::find_class,
    into_rust::ToRustOp,
    java::lang::{Class, Throwable},
    link::{IntoJavaFns, JavaFunction},
    not_null::NotNull,
    plumbing::{FromRef, ToJavaImpl},
    raw::{self, EnvPtr, JvmPtr, ObjectPtr},
    thread,
    try_catch::TryCatch,
    AsJRef, Error, IntoRust, Java, Local, Result, ToJava, TryJDeref,
};

use std::{
    any::Any,
    collections::HashMap,
    ffi::{c_char, c_void, CStr, CString},
    fmt::Display,
    panic::AssertUnwindSafe,
};

use once_cell::sync::OnceCell;

#[cfg(test)]
mod test;

/// A "jdk op" is a suspended operation that, when executed, will run
/// on the jvm, producing a value of type `Output`. These ops typically
/// represent constructor or method calls, and they can be chained
/// together.
///
/// *Eventual goal:* Each call to `execute` represents a single crossing
/// over into the JVM, so the more you can chain together your jvm-ops,
/// the better.
#[must_use = "JvmOps do nothing unless you call `.execute()"]
pub trait JvmOp: Clone {
    type Output<'jvm>;

    fn assert_not_null<T>(self) -> NotNull<Self>
    where
        T: JavaObject,
        for<'jvm> Self: JvmOp<Output<'jvm> = Option<Local<'jvm, T>>>,
    {
        NotNull::new(self)
    }

    /// Tries to downcast output of this operation to `To`, otherwise returning
    /// the output as is. Equivalent to
    /// ```java
    /// From x;
    /// if (x instanceof To) {
    ///    return Ok((To) x);
    /// } else {
    ///    return Err(x);
    /// }
    /// ```
    fn try_downcast<To>(self) -> TryDowncast<Self, To>
    where
        for<'jvm> Self::Output<'jvm>: TryJDeref,
        To: for<'jvm> Upcast<<Self::Output<'jvm> as TryJDeref>::Java>,
    {
        TryDowncast::new(self)
    }

    /// Most duchess-wrapped Java objects will automatically be able to call all
    /// methods defined on any of its super classes or interfaces it implements,
    /// but this can be used to "force" the output of the operation to be typed
    /// as an explicit super type `To`.
    fn upcast<To>(self) -> AsUpcast<Self, To>
    where
        for<'jvm> Self::Output<'jvm>: AsJRef<To>,
        To: JavaObject,
    {
        AsUpcast::new(self)
    }

    fn catch<J>(self) -> TryCatch<Self, J>
    where
        J: Upcast<Throwable>,
    {
        TryCatch::new(self)
    }

    /// Execute on the JVM, starting a JVM instance if necessary.
    ///
    /// Depending on the type parameter `R`,
    /// this method can either return a handle to a Java object
    /// or a Rust type:
    ///
    /// * When `R` is something like [`Java<java::lang::String>`][`Java`],
    ///   this method will return a handle to a Java object.
    ///   Note that to account for possible null return values
    ///   you may need to either invoke `assert_not_null` or else
    ///   use a result type with an `Option`, e.g.,
    ///   `Option<Java<java::lang::String>>`.
    /// * When `R` is a Rust type like [`String`][],
    ///   this method will convert the Java value into a Rust type.
    ///   You may need to derive `ToRust` for your Rust type
    ///   to indicate how the Java object is to be converted.
    fn execute<R>(self) -> crate::Result<R>
    where
        for<'jvm> Self::Output<'jvm>: IntoRust<R>,
    {
        Jvm::with(|jvm| self.execute_with(jvm))
    }

    /// Internal method
    fn execute_with<'jvm, R>(self, jvm: &mut Jvm<'jvm>) -> crate::LocalResult<'jvm, R>
    where
        for<'j> Self::Output<'j>: IntoRust<R>,
    {
        ToRustOp::new(self).do_jni(jvm)
    }
    /// Internal method
    fn do_jni<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::LocalResult<'jvm, Self::Output<'jvm>>;
}

/// A (pseudo) alias for a`JvmOp` that provides "something converted to a Java `T`".
/// Don't implement this yourself, just implement `JvmOp`.
///
/// # Implementation note
///
/// Ideally this would be a "trait alias" for `JvmOp<Output<'_>: JvmRefOp<T>>`, but
/// adding a where-clause to that effect did not seem to work in all cases, so we define
/// a distinct associated type.
pub trait JvmRefOp<T: JavaObject>: Clone {
    // nikomatsakis:
    type Output<'jvm>: AsJRef<T>;

    fn into_as_jref<'jvm>(
        self,
        jvm: &mut Jvm<'jvm>,
    ) -> crate::LocalResult<'jvm, Self::Output<'jvm>>;
}

impl<J, T> JvmRefOp<T> for J
where
    T: JavaObject,
    J: JvmOp,
    for<'jvm> <J as JvmOp>::Output<'jvm>: AsJRef<T>,
{
    type Output<'jvm> = <J as JvmOp>::Output<'jvm>;

    fn into_as_jref<'jvm>(
        self,
        jvm: &mut Jvm<'jvm>,
    ) -> crate::LocalResult<'jvm, <Self as JvmRefOp<T>>::Output<'jvm>> {
        JvmOp::do_jni(self, jvm)
    }
}

/// A [`JvmOp`] that produces a scalar value, like `i8` or `i32`.
pub trait JvmScalarOp<T: JavaScalar>: for<'jvm> JvmOp<Output<'jvm> = T> {}

impl<J, T> JvmScalarOp<T> for J
where
    T: JavaScalar,
    J: for<'jvm> JvmOp<Output<'jvm> = T>,
{
}

static GLOBAL_JVM: OnceCell<JvmPtr> = OnceCell::new();

fn get_or_default_init_jvm() -> crate::Result<JvmPtr> {
    match GLOBAL_JVM.get() {
        Some(jvm) => Ok(*jvm),
        None => {
            Jvm::builder().launch_or_use_existing()?;
            Ok(*GLOBAL_JVM
                .get()
                .expect("launch_or_use_existing must set GLOBAL_JVM"))
        }
    }
}

fn throw_java_runtime_exception(env: EnvPtr<'_>, message: &str) {
    let mut jvm = Jvm(env);

    let runtime_exception_clazz = crate::java::lang::RuntimeException::class(&mut jvm)
        .expect("java/lang/RuntimeException not found");

    let runtime_exception_clazz_ptr = runtime_exception_clazz.as_raw().as_ptr();

    let encoded = cesu8::to_java_cesu8(message);
    // SAFETY: cesu8 encodes interior nul bytes as 0xC080
    let c_string = unsafe { CString::from_vec_unchecked(encoded.into_owned()) };
    let c_string_ptr = c_string.as_ptr();

    unsafe {
        env.invoke_unchecked(
            |env| env.ThrowNew,
            |jni, f| f(jni, runtime_exception_clazz_ptr, c_string_ptr),
        );
    };
}

fn error_to_java_exception(env: EnvPtr<'_>, error: Error<Local<'_, Throwable>>) {
    // SAFETY: invoke_unchecked is used here to raise an exception. The exception is not
    // cleared to force the caller to handle the exception
    let _ = match error {
        Error::Thrown(t) => unsafe {
            env.invoke_unchecked(|env| env.Throw, |env, f| f(env, t.as_raw().as_ptr()));
        },
        Error::JvmInternal(s) => {
            throw_java_runtime_exception(env, &s);
        }
        Error::NullDeref => {
            let mut jvm = Jvm(env);
            let npe_clazz = crate::java::lang::NullPointerException::class(&mut jvm)
                .expect("java/lang/NullPointerException not found");
            let npe_clazz_ptr = npe_clazz.as_raw().as_ptr();

            unsafe {
                env.invoke_unchecked(
                    |env| env.ThrowNew,
                    |jni, f| f(jni, npe_clazz_ptr, std::ptr::null()),
                );
            }
        }
        e => {
            throw_java_runtime_exception(env, &format!("{}", e));
        }
    };
}

/// Invoked as the body from a JNI native function when it is called by the JVM.
/// Initializes the environment and invokes `op`. Converts the result into a java
/// object and returns it. Caller should then return this to the JVM.
///
/// # Safety condition
///
/// Must be invoked as the entire body of a JNI native function, with
/// `env` being the `EnvPtr` argument provided.
pub unsafe fn native_function_returning_object<J, R>(
    env: EnvPtr<'_>,
    op: impl FnOnce() -> R,
) -> jni_sys::jobject
where
    J: Upcast<crate::java::lang::Object> + Upcast<J>,
    R: ToJavaImpl<J>,
{
    init_jvm_from_native_function(env);
    let _callback_guard = thread::attach_from_jni_callback(env);

    let result = match std::panic::catch_unwind(AssertUnwindSafe(|| op())) {
        Ok(result) => {
            let mut jvm = Jvm(env);
            let obj = result.to_java().do_jni(&mut jvm);
            match obj {
                Ok(Some(p)) => p.into_raw().as_ptr(),
                Ok(None) => std::ptr::null_mut(),
                Err(e) => {
                    error_to_java_exception(env, e);
                    std::ptr::null_mut()
                }
            }
        }

        Err(e) => {
            let () = rust_panic_to_java_exception(env, e);
            std::ptr::null_mut()
        }
    };

    result
}

/// Invoked as the body from a JNI native function when it is called by the JVM.
/// Initializes the environment and invokes `op`, returning the result, which should
/// then be returned to the JVM.
///
/// # Safety condition
///
/// Must be invoked as the entire body of a JNI native function, with
/// `env` being the `EnvPtr` argument provided.
pub unsafe fn native_function_returning_scalar<J, R>(env: EnvPtr<'_>, op: impl FnOnce() -> R) -> R
where
    J: Upcast<crate::java::lang::Object> + Upcast<J>,
    R: JavaScalar,
{
    init_jvm_from_native_function(env);
    let _callback_guard = thread::attach_from_jni_callback(env);

    let result = match std::panic::catch_unwind(AssertUnwindSafe(|| op())) {
        Ok(result) => result,
        Err(e) => {
            rust_panic_to_java_exception(env, e);
            R::default()
        }
    };

    result
}

/// Invoked from inside a JNI native function when it is called by the JVM.
/// If `GLOBAL_JVM` is not yet set, initializes it to use the provided `jvm`.
/// Otherwise, does nothing.
///
/// # Safety condition
///
/// Must be invoked as the first thing from inside a JNI native function.
unsafe fn init_jvm_from_native_function(env: EnvPtr<'_>) -> Jvm<'_> {
    // If the JVM is the master process and it invokes Rust code,
    // the global JVM environment may not yet have been initialized.
    //
    // If the Rust code is the master process, the JVM should already have
    // been created and should be the same.

    let jvm = env.jvm_ptr().unwrap();
    let global_jvm = GLOBAL_JVM.get_or_init(|| jvm);
    assert_eq!(jvm, *global_jvm, "multiple JVM pointers in active use");
    Jvm(env)
}

fn rust_panic_to_java_exception(env: EnvPtr<'_>, panic: Box<dyn Any + Send + 'static>) {
    // The documentation suggests that it will *usually* be a str or String.
    let message = if let Some(s) = panic.downcast_ref::<&'static str>() {
        (*s).to_string()
    } else if let Some(s) = panic.downcast_ref::<String>() {
        s.clone()
    } else {
        "Unknown panic!".to_string()
    };

    throw_java_runtime_exception(env, &message);
}

/// Get the global [`JvmPtr`] assuming that the JVM has already been initialized. Expected to be used with values
/// that only can have been derived from an existing JVM.
///
/// # Panics
///
/// Panics if the JVM wasn't initialized.
pub(crate) fn unwrap_global_jvm() -> JvmPtr {
    *GLOBAL_JVM.get().expect("JVM can't be unset")
}

/// Represents a handle to a running JVM.
/// You rarely access this explicitly as a duchess user.
pub struct Jvm<'jvm>(EnvPtr<'jvm>);

impl<'jvm> Jvm<'jvm> {
    /// Construct
    pub fn builder() -> JvmBuilder {
        JvmBuilder::new()
    }

    pub fn attach_thread_permanently() -> crate::Result<()> {
        thread::attach_permanently(get_or_default_init_jvm()?)?;
        Ok(())
    }

    /// Call the callback with access to a `Jvm`.
    /// This cannot be invoked recursively.
    /// It is crate-local because it is only usd from within
    /// the `execute` method on [`JvmOp`][].
    pub(crate) fn with<R>(
        op: impl for<'a> FnOnce(&mut Jvm<'a>) -> crate::LocalResult<'a, R>,
    ) -> crate::Result<R> {
        let jvm = get_or_default_init_jvm()?;
        // SAFTEY: we won't deinitialize the JVM while the guard is live
        let mut guard = unsafe { thread::attach(jvm)? };

        let mut jvm = Jvm(guard.env());
        op(&mut jvm).map_err(|e| e.into_global(&mut jvm))
    }

    pub fn local<R>(&mut self, r: &R) -> Local<'jvm, R>
    where
        R: JavaObject,
    {
        Local::new(self.0, r)
    }

    pub fn global<R>(&mut self, r: &R) -> Java<R>
    where
        R: JavaObject,
    {
        Java::new(self.0, r)
    }

    /// Plumbing method that should only be used by generated and internal code.
    #[doc(hidden)]
    pub fn env(&self) -> EnvPtr<'jvm> {
        self.0
    }

    fn register_native_methods(
        &mut self,
        java_functions: &[JavaFunction],
    ) -> crate::LocalResult<'jvm, ()> {
        let mut sorted_by_class: HashMap<Local<'_, Class>, Vec<jni_sys::JNINativeMethod>> =
            HashMap::default();

        for java_function in java_functions {
            let class = (java_function.class_fn)(self)?;
            sorted_by_class
                .entry(class)
                .or_insert(vec![])
                .push(jni_sys::JNINativeMethod {
                    name: java_function.name.as_ptr() as *mut c_char,
                    signature: java_function.signature.as_ptr() as *mut c_char,
                    fnPtr: java_function.pointer.as_ptr() as *mut c_void,
                });
        }

        for (class, native_methods) in &sorted_by_class {
            unsafe {
                self.0
                    .register_native_methods(class.as_raw(), native_methods)?;
            }
        }

        Ok(())
    }
}

pub struct JvmBuilder {
    options: Vec<String>,
    #[cfg(feature = "dylibjvm")]
    libjvm_path: Option<std::path::PathBuf>,
    java_functions: Vec<JavaFunction>,
}

impl JvmBuilder {
    fn new() -> Self {
        let mut this = Self {
            options: vec![],
            #[cfg(feature = "dylibjvm")]
            libjvm_path: None,
            java_functions: vec![],
        };

        if cfg!(debug_assertions) {
            this = this.custom("-Xcheck:jni");
        }
        if let Ok(classpath) = std::env::var("CLASSPATH") {
            this = this.add_classpath(classpath);
        }

        this
    }

    pub fn add_classpath(self, classpath: impl Display) -> Self {
        self.custom(format!("-Djava.class.path={classpath}"))
    }

    pub fn custom(mut self, opt_string: impl Into<String>) -> Self {
        self.options.push(opt_string.into());
        self
    }

    pub fn link(mut self, fns: impl IntoJavaFns) -> Self {
        self.java_functions.extend(fns.into_java_fns());
        self
    }

    #[cfg(feature = "dylibjvm")]
    pub fn load_libjvm_at(mut self, path: impl AsRef<std::path::Path>) -> Self {
        self.libjvm_path = Some(path.as_ref().into());
        self
    }

    /// Launch a new JVM, returning [`Error::JvmAlreadyExists`] if one already exists.
    pub fn try_launch(self) -> Result<()> {
        #[cfg(feature = "dylibjvm")]
        if let Some(path) = self.libjvm_path {
            crate::libjvm::libjvm_or_load_at(&path)?;
        }

        let mut already_exists = true;
        GLOBAL_JVM.get_or_try_init(|| {
            // SAFETY: we're behind the GLOBAL_JVM lock and we won't race with other threads creating or finding an
            // existing JVM.
            let jvm = unsafe { raw::try_create_jvm(self.options.into_iter()) }?;
            already_exists = false;
            Result::Ok(jvm)
        })?;

        if already_exists {
            Err(Error::JvmAlreadyExists)
        } else {
            if !self.java_functions.is_empty() {
                Jvm::with(|jvm| jvm.register_native_methods(&self.java_functions))?;
            }

            Ok(())
        }
    }

    pub fn launch_or_use_existing(self) -> Result<()> {
        match self.try_launch() {
            Err(Error::JvmAlreadyExists) => {
                // Two cases: (1) another thread successfully invoked try_launch() and we'll now get the pointer out of
                // GLOBAL_JVM, or (2) the JVM was created by some non-duchess code and we'll now need to look it up with
                // the existing_jvm() call.
                GLOBAL_JVM.get_or_try_init(|| {
                    // SAFETY: we're behind the GLOBAL_JVM lock and we won't race with other threads creating or finding
                    // an existing JVM.
                    Result::Ok(unsafe { raw::existing_jvm() }?.expect("JVM should already exist"))
                })?;
                Ok(())
            }
            result => result,
        }
    }
}

/// A trait for zero-sized dummy types that represent Java object types.
///
/// # Safety
///
/// A type `T` that implements this trait must satisfy the following contract:
///
/// 1. `T` must be a zero-sized type.
/// 2. It must not be possible to construct a value of type `T`.
/// 3. The alignment of `T` must *not* be greater than the alignment of [jni::sys::_jobject]. (I
///    *think* this is always true for zero-sized types, so would be implied by rule #1, but I'm not
///    sure.)
/// 4. The conditions on the trait methods.
///
/// # Example
///
/// ```ignore
/// # use duchess::JavaObject;
/// pub struct BigDecimal {
///     _private: (), // prevent construction
/// }
/// unsafe impl JavaObject for BigDecimal {}
/// ```
pub unsafe trait JavaObject: 'static + Sized + JavaType + JavaView {
    /// Returns Java Class object for this type.
    ///
    /// # Implementation safety conditions
    ///
    /// Implementations of `JavaObject` must ensure that the `class` object
    /// resulting from this call is permanently cached in a JVM global reference.
    /// This is needed so that we can cache field and method IDs derived from
    /// reference, as those IDs are only guaranteed to remain stable so long as
    /// the Java class is not collected.
    fn class<'jvm>(jvm: &mut Jvm<'jvm>) -> crate::LocalResult<'jvm, Local<'jvm, Class>>;
}

pub trait JavaView {
    /// The [op struct] for this java object.
    /// This is an internal plumbing detail.
    /// [op struct]: https://duchess-rs.github.io/duchess/methods.html#op-structs
    type OfOp<J>: FromRef<J>;

    /// The [op struct] for this java object with the given Method Resolution Order (`N`).
    /// This is an internal plumbing detail.
    /// [op struct]: https://duchess-rs.github.io/duchess/methods.html#op-structs
    type OfOpWith<J, N>: FromRef<J>
    where
        N: FromRef<J>;

    /// The [object struct] for this java object.
    /// This is an internal plumbing detail.
    /// [object struct]: https://duchess-rs.github.io/duchess/methods.html#obj-structs
    type OfObj<J>: FromRef<J>;

    /// The [object struct] for this java object with the given Method Resolution Order (`N`).
    /// This is an internal plumbing detail.
    /// [object struct]: https://duchess-rs.github.io/duchess/methods.html#obj-structs
    type OfObjWith<J, N>: FromRef<J>
    where
        N: FromRef<J>;
}

/// Extension trait for [JavaObject].
pub trait JavaObjectExt: Sized {
    // We use an extension trait, instead of just declaring these functions on the main JavaObject
    // trait, to prevent trait implementors from overriding the implementation of these functions.

    unsafe fn from_raw<'a>(ptr: ObjectPtr) -> &'a Self;
    fn as_raw(&self) -> ObjectPtr;
}

impl<T: JavaObject> JavaObjectExt for T {
    /// # Safety
    ///
    /// The caller must ensure that the pointed-to object remains live through `'a` and is an instance of `T` (or its
    /// subclasses).
    unsafe fn from_raw<'a>(ptr: ObjectPtr) -> &'a Self {
        // SAFETY: The cast is sound because:
        //
        // 1. A pointer to a suitably aligned `sys::_jobject` should also satisfy Self's alignment
        //    requirement (trait rule #3)
        // 2. Self is a zero-sized type (trait rule #1), so there are no invalid bit patterns to
        //    worry about.
        // 3. Self is a zero-sized type (trait rule #1), so there's no actual memory region that is
        //    subject to the aliasing rules.
        unsafe { ptr.as_ref() }
    }

    fn as_raw(&self) -> ObjectPtr {
        let ptr: *const Self = self;
        let jobj = ptr.cast_mut().cast::<jni_sys::_jobject>();
        // SAFETY: From JavaObject trait contract, T is a non-constructable, zero-sized type. The only way to obtain a
        // &T is through from_raw() given a valid JNI object pointer. from_raw() callers are responsible for ensuring
        // the lifetime of the borrow isn't longer than the lifetime of the java local or global it points to.
        unsafe { ObjectPtr::new(jobj).unwrap_unchecked() }
    }
}

pub unsafe trait JavaType: 'static {
    /// Returns the Java Class object for a Java array containing elements of
    /// `Self`. All Java types, even scalars can be elements of an array object.
    fn array_class<'jvm>(jvm: &mut Jvm<'jvm>) -> crate::LocalResult<'jvm, Local<'jvm, Class>>;
}

unsafe impl<T: JavaObject> JavaType for T {
    fn array_class<'jvm>(jvm: &mut Jvm<'jvm>) -> crate::LocalResult<'jvm, Local<'jvm, Class>> {
        T::class(jvm)?.array_type().assert_not_null().do_jni(jvm)
    }
}

pub trait JavaScalar: JavaType + Default {}

macro_rules! scalar {
    ($($rust:ty: $array_class:literal,)*) => {
        $(
            unsafe impl JavaType for $rust {
                fn array_class<'jvm>(jvm: &mut Jvm<'jvm>) -> crate::LocalResult<'jvm, Local<'jvm, Class>> {
                    // XX: Safety
                    const CLASS_NAME: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked($array_class) };
                    static CLASS: OnceCell<Java<crate::java::lang::Class>> = OnceCell::new();

                    let global = CLASS.get_or_try_init::<_, crate::Error<Local<Throwable>>>(|| {
                        let class = find_class(jvm, CLASS_NAME)?;
                        Ok(jvm.global(&class))
                    })?;
                    Ok(jvm.local(global))
                }
            }

            impl JavaScalar for $rust {}
        )*
    };
}

scalar! {
    bool: b"[Z\0",
    i8:   b"[B\0",
    i16:  b"[S\0",
    u16:  b"[C\0",
    i32:  b"[I\0",
    i64:  b"[J\0",
    f32:  b"[F\0",
    f64:  b"[D\0",
}
