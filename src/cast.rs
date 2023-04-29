use std::marker::PhantomData;

use crate::{jvm::JavaObjectExt, raw::HasEnvPtr, JavaObject, Jvm, JvmOp, Local};

/// A trait to represent safe upcast operations for a [`JavaObject`].
///
/// # Safety
///
/// Inherits the rules of [`JavaObject`], but also `S` must be a valid superclass or implemented interface of `Self`.
/// XX: would this actually allow unsafe behavior in a JNI call? or is it already checked/enforced?
///
/// XX: having to impl Upcast<T> for T on each struct is pretty annoying to get AsRef<T> to work without conflicts
pub unsafe trait Upcast<S: JavaObject>: JavaObject {}

pub struct TryDowncast<J, From, To> {
    op: J,
    _marker: PhantomData<(From, To)>,
}

impl<J: Clone, From, To> Clone for TryDowncast<J, From, To> {
    fn clone(&self) -> Self {
        Self {
            op: self.op.clone(),
            _marker: PhantomData,
        }
    }
}

impl<J, From, To> TryDowncast<J, From, To>
where
    J: JvmOp,
    for<'jvm> J::Output<'jvm>: AsRef<From>,
    From: JavaObject,
    To: Upcast<From>,
{
    pub(crate) fn new(op: J) -> Self {
        Self {
            op,
            _marker: PhantomData,
        }
    }

    pub fn execute<'jvm>(
        self,
        jvm: &mut Jvm<'jvm>,
    ) -> crate::Result<'jvm, Result<Local<'jvm, To>, J::Output<'jvm>>>
    where
        J: JvmOp<Input<'jvm> = ()>,
    {
        self.execute_with(jvm, ())
    }
}

impl<J, From, To> JvmOp for TryDowncast<J, From, To>
where
    J: JvmOp,
    for<'jvm> J::Output<'jvm>: AsRef<From>,
    From: JavaObject,
    To: Upcast<From>,
{
    type Input<'jvm> = J::Input<'jvm>;
    type Output<'jvm> = Result<Local<'jvm, To>, J::Output<'jvm>>;

    fn execute_with<'jvm>(
        self,
        jvm: &mut crate::Jvm<'jvm>,
        input: J::Input<'jvm>,
    ) -> crate::Result<'jvm, Self::Output<'jvm>> {
        let instance = self.op.execute_with(jvm, input)?;
        let instance_raw = instance.as_ref().as_raw();

        let class = To::class(jvm)?;
        let class_raw = class.as_raw();

        let env = jvm.env();
        let is_inst = unsafe {
            env.invoke(
                |env| env.IsInstanceOf,
                |env, f| f(env, instance_raw.as_ptr(), class_raw.as_ptr()),
            ) == jni_sys::JNI_TRUE
        };

        if is_inst {
            // XX: Safety: just shown that jobject instanceof To::class
            let casted = unsafe { std::mem::transmute::<&From, &To>(instance.as_ref()) };
            Ok(Ok(jvm.local(casted)))
        } else {
            Ok(Err(instance))
        }
    }
}

pub struct AsUpcast<J, From, To> {
    op: J,
    _marker: PhantomData<(From, To)>,
}

impl<J: Clone, From, To> Clone for AsUpcast<J, From, To> {
    fn clone(&self) -> Self {
        Self {
            op: self.op.clone(),
            _marker: PhantomData,
        }
    }
}

impl<J, From, To> AsUpcast<J, From, To>
where
    J: JvmOp,
    for<'jvm> J::Output<'jvm>: AsRef<From>,
    From: Upcast<To>,
    To: JavaObject,
{
    pub(crate) fn new(op: J) -> Self {
        Self {
            op,
            _marker: PhantomData,
        }
    }

    pub fn execute<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, Local<'jvm, To>>
    where
        J: JvmOp<Input<'jvm> = ()>,
    {
        self.execute_with(jvm, ())
    }
}

impl<J, From, To> JvmOp for AsUpcast<J, From, To>
where
    J: JvmOp,
    for<'jvm> J::Output<'jvm>: AsRef<From>,
    From: Upcast<To>,
    To: JavaObject,
{
    type Input<'jvm> = J::Input<'jvm>;
    type Output<'jvm> = Local<'jvm, To>;

    fn execute_with<'jvm>(
        self,
        jvm: &mut crate::Jvm<'jvm>,
        input: J::Input<'jvm>,
    ) -> crate::Result<'jvm, Self::Output<'jvm>> {
        let instance = self.op.execute_with(jvm, input)?;

        if cfg!(debug_assertions) {
            let class = To::class(jvm)?;
            let class_raw = class.as_raw();

            let instance_raw = instance.as_ref().as_raw();
            assert!(unsafe {
                jvm.env().invoke(
                    |env| env.IsInstanceOf,
                    |env, f| f(env, instance_raw.as_ptr(), class_raw.as_ptr()),
                ) == jni_sys::JNI_TRUE
            });
        }

        // Safety: From: Upcast<To>
        Ok(jvm.local(unsafe { std::mem::transmute::<&From, &To>(instance.as_ref()) }))
    }
}

/// Branch on the instance type of a Java object. It will execute (and only execute) the first match arm that has a type
/// the object is an instance of. This can be a class, an interface, etc. If the object is not an instance of any arm,
/// the `else` arm is taken.
///
/// # Example
///
/// ```
/// # use duchess::{Result, Local, Jvm, java::{self, lang::{ThrowableExt, StringExt}}};
/// # use duchess::prelude::*;
/// fn inspect<'jvm>(jvm: &mut Jvm<'jvm>, x: Local<'jvm, java::lang::Object>) -> Result<'jvm, ()> {
///     duchess::by_type! {
///         with jvm match x => {
///             java::lang::String as string => {
///                 println!("Got a string with {} chars", string.length().execute(jvm)?);
///             },
///             java::lang::Throwable as throwable => {
///                 throwable.print_stack_trace().execute(jvm)?;
///             },
///             else {
///                 println!("Got something that wasn't a String or a Throwable");
///             }
///         }
///     }
///     Ok(())
/// }
/// ```
///
/// is equivalent to Java
/// ```java
/// void inspect(Object x) {
///     if (x instanceof String) {
///         String string = (String) x;
///         System.out.println(String.format("Got a string with %d chars", x.length()));
///     } else if (x instanceof Throwable) {
///         Throwable throwable = (Throwable) x;
///         throwable.printStackTrace();
///     } else {
///         System.out.println("Got something that wasn't a String or a Throwable");
///     }
/// }
/// ```
///
#[macro_export]
macro_rules! by_type {
    (with $jvm:ident match $obj:expr => {
        $($class:ty as $var:ident => $arm:expr,)*
        else $(as $default_var:ident)? $default:block
    }) => {
        {
            let obj = $obj;
            if false {
                unreachable!()
            }
            $(
                else if let Ok($var) = obj.try_downcast::<_, $class>().execute($jvm)? {
                    $arm
                }
            )*
            else {
                $(let $default_var = obj;)?
                $default
            }
        }

    };
}
pub use by_type;
