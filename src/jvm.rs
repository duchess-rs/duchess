use crate::{
    cast::{AsUpcast, TryDowncast, Upcast},
    find::find_class,
    global::{GlobalOp, IntoGlobal},
    java::lang::{Class, ClassExt, Throwable},
    not_null::NotNull,
    raw::{self, EnvPtr, HasEnvPtr, JvmPtr, ObjectPtr},
    thread,
    to_rust::ToRustOp,
    try_catch::TryCatch,
    AsJRef, Error, Global, GlobalResult, Local, ToRust, TryJDeref,
};

use std::{ffi::CStr, fmt::Display, ptr::NonNull};

use once_cell::sync::OnceCell;

/// A "jdk op" is a suspended operation that, when executed, will run
/// on the jvm, producing a value of type `Output`. These ops typically
/// represent constructor or method calls, and they can be chained
/// together.
///
/// *Eventual goal:* Each call to `execute` represents a single crossing
/// over into the JVM, so the more you can chain together your jvm-ops,
/// the better.
pub trait JvmOp: Sized {
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

    /// Given a JVM op that creates a local reference, convert the local reference
    /// into a global one. Global JVM references can be held as long as you like
    /// within
    fn global(self) -> GlobalOp<Self>
    where
        for<'jvm> <Self as JvmOp>::Output<'jvm>: IntoGlobal<'jvm>,
    {
        GlobalOp::new(self)
    }

    fn catch<J>(self) -> TryCatch<Self, J>
    where
        J: Upcast<Throwable>,
    {
        TryCatch::new(self)
    }

    /// Given a JVM op that returns some Java type, convert it to its Rust equivalent
    /// (e.g., from a Java String to a Rust string).
    fn to_rust<R>(self) -> ToRustOp<Self, R>
    where
        for<'jvm> Self::Output<'jvm>: ToRust<R>,
    {
        ToRustOp::new(self)
    }

    /// Execute the jvm op, starting a JVM instance if necessary.
    /// To use this method, the result type cannot be tied to the JVM.
    /// Typically this is achieved by a call to [`to_rust()`][`Self::to_rust`],
    /// but if you wish to hold on to a reference to a JVM object,
    /// you can use [`global()`][`Self::global`] to create a global reference.
    fn execute<R>(self) -> crate::GlobalResult<R>
    where
        for<'jvm> Self: JvmOp<Output<'jvm> = R>,
    {
        Jvm::with(|jvm| self.execute_with(jvm))
    }

    fn execute_with<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, Self::Output<'jvm>>;
}

/// This trait is only implemented for `()`; it allows the `JvmOp::execute` method to only
/// be used for `()`.
pub trait IsVoid: Default {}
impl IsVoid for () {}

static GLOBAL_JVM: OnceCell<JvmPtr> = OnceCell::new();

fn get_or_default_init_jvm() -> crate::GlobalResult<JvmPtr> {
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

/// Get the global [`JvmPtr`] assuming that the JVM has already been initialized. Expected to be used with values
/// that only can have been derived from an existing JVM.
///
/// # Panics
///
/// Panics if the JVM wasn't initialized.
pub(crate) fn unwrap_global_jvm() -> JvmPtr {
    *GLOBAL_JVM.get().expect("JVM can't be unset")
}

pub struct Jvm<'jvm>(EnvPtr<'jvm>);

impl<'jvm> Jvm<'jvm> {
    pub fn builder() -> JvmBuilder {
        JvmBuilder::new()
    }

    pub fn attach_thread_permanently() -> crate::GlobalResult<()> {
        thread::attach_permanently(get_or_default_init_jvm()?)?;
        Ok(())
    }

    pub fn with<R>(
        op: impl for<'a> FnOnce(&mut Jvm<'a>) -> crate::Result<'a, R>,
    ) -> crate::GlobalResult<R> {
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

    pub fn global<R>(&mut self, r: &R) -> Global<R>
    where
        R: JavaObject,
    {
        Global::new(self.0, r)
    }
}

impl<'jvm> HasEnvPtr<'jvm> for Jvm<'jvm> {
    fn env(&self) -> EnvPtr<'jvm> {
        self.0
    }
}

pub struct JvmBuilder {
    options: Vec<String>,
    #[cfg(feature = "dylibjvm")]
    libjvm_path: Option<std::path::PathBuf>,
}

impl JvmBuilder {
    fn new() -> Self {
        let mut this = Self {
            options: vec![],
            #[cfg(feature = "dylibjvm")]
            libjvm_path: None,
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

    #[cfg(feature = "dylibjvm")]
    pub fn load_libjvm_at(mut self, path: impl AsRef<std::path::Path>) -> Self {
        self.libjvm_path = Some(path.as_ref().into());
        self
    }

    /// Launch a new JVM, returning [`Error::JvmAlreadyExists`] if one already exists.
    pub fn try_launch(self) -> GlobalResult<()> {
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
            GlobalResult::Ok(jvm)
        })?;

        if already_exists {
            Err(Error::JvmAlreadyExists)
        } else {
            Ok(())
        }
    }

    pub fn launch_or_use_existing(self) -> GlobalResult<()> {
        match self.try_launch() {
            Err(Error::JvmAlreadyExists) => {
                // Two cases: (1) another thread successfully invoked try_launch() and we'll now get the pointer out of
                // GLOBAL_JVM, or (2) the JVM was created by some non-duchess code and we'll now need to look it up with
                // the existing_jvm() call.
                GLOBAL_JVM.get_or_try_init(|| {
                    // SAFETY: we're behind the GLOBAL_JVM lock and we won't race with other threads creating or finding
                    // an existing JVM.
                    GlobalResult::Ok(
                        unsafe { raw::existing_jvm() }?.expect("JVM should already exist"),
                    )
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
pub unsafe trait JavaObject: 'static + Sized + JavaType {
    // XX: can't be put on extension trait nor define a default because we want to cache the resolved
    // class in a static OnceCell.
    /// Returns Java Class object for this type.
    fn class<'jvm>(jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, Local<'jvm, Class>>;
}

/// Extension trait for [JavaObject].
pub trait JavaObjectExt: Sized {
    // We use an extension trait, instead of just declaring these functions on the main JavaObject
    // trait, to prevent trait implementors from overriding the implementation of these functions.

    unsafe fn from_raw<'a>(ptr: ObjectPtr) -> &'a Self;
    fn as_raw(&self) -> ObjectPtr;
}

impl<T: JavaObject> JavaObjectExt for T {
    unsafe fn from_raw<'a>(ptr: ObjectPtr) -> &'a Self {
        // XX: safety
        unsafe { ptr.as_ref() }
    }

    fn as_raw(&self) -> ObjectPtr {
        // XX: safety
        unsafe { NonNull::new_unchecked((self as *const Self).cast_mut()).cast() }.into()
    }
}

pub unsafe trait JavaType: 'static {
    /// Returns the Java Class object for a Java array containing elements of
    /// `Self`. All Java types, even scalars can be elements of an array object.
    fn array_class<'jvm>(jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, Local<'jvm, Class>>;
}

unsafe impl<T: JavaObject> JavaType for T {
    fn array_class<'jvm>(jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, Local<'jvm, Class>> {
        T::class(jvm)?
            .array_type()
            .assert_not_null()
            .execute_with(jvm)
    }
}

pub trait JavaScalar: JavaType {}

macro_rules! scalar {
    ($($rust:ty: $array_class:literal,)*) => {
        $(
            unsafe impl JavaType for $rust {
                fn array_class<'jvm>(jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, Local<'jvm, Class>> {
                    // XX: Safety
                    const CLASS_NAME: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked($array_class) };
                    static CLASS: OnceCell<Global<crate::java::lang::Class>> = OnceCell::new();

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

pub trait CloneIn<'jvm> {
    fn clone_in(&self, jvm: &mut Jvm<'jvm>) -> Self;
}

impl<T> CloneIn<'_> for T
where
    T: Clone,
{
    fn clone_in(&self, _jvm: &mut Jvm<'_>) -> Self {
        self.clone()
    }
}
