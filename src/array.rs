use std::{env, marker::PhantomData};

use crate::{
    cast::Upcast,
    java::{self, lang::Class},
    jvm::{self, JavaScalar, JavaView, JvmBuilder},
    raw,
    semver_unstable::{FromRef, JavaObjectExt},
    to_java::ToJavaImpl,
    AsJRef, Error, IntoRust, JDeref, JavaObject, JavaType, Jvm, JvmOp, Local, Nullable,
    ScalarMethod, TryJDeref, VoidMethod,
};

pub struct JavaArray<T> {
    _element: PhantomData<T>,
}

#[repr(transparent)]
pub struct JavaArrayOp<T, J, N> {
    _this: J,
    phantom: PhantomData<(JavaArray<T>, N)>,
}

impl<T, J, N> FromRef<J> for JavaArrayOp<T, J, N> {
    fn from_ref(j: &J) -> &Self {
        // Safe because of the `repr(transparent)` declaration
        unsafe { std::mem::transmute::<&J, &Self>(j) }
    }
}

#[repr(transparent)]
pub struct JavaArrayObj<T, J, N> {
    _this: J,
    phantom: PhantomData<(JavaArray<T>, N)>,
}

impl<T, J, N> FromRef<J> for JavaArrayObj<T, J, N> {
    fn from_ref(j: &J) -> &Self {
        // Safe because of the `repr(transparent)` declaration
        unsafe { std::mem::transmute::<&J, &Self>(j) }
    }
}

unsafe impl<T: JavaType> JavaObject for JavaArray<T> {
    fn class<'jvm>(jvm: &mut Jvm<'jvm>) -> crate::LocalResult<'jvm, Local<'jvm, Class>> {
        T::array_class(jvm)
    }
}

impl<T> JavaView for JavaArray<T> {
    type OfOp<J> = JavaArrayOp<T, J, <java::lang::Object as JavaView>::OfOpWith<J, ()>>;

    type OfOpWith<J, N>
        = JavaArrayOp<T, J, N>
    where
        N: FromRef<J>;

    type OfObj<J> = JavaArrayObj<T, J, <java::lang::Object as JavaView>::OfObjWith<J, ()>>;

    type OfObjWith<J, N>
        = JavaArrayObj<T, J, N>
    where
        N: FromRef<J>;
}

// Upcasts
unsafe impl<T: JavaType> Upcast<JavaArray<T>> for JavaArray<T> {}

// all arrays extend Object
unsafe impl<T: JavaType> Upcast<java::lang::Object> for JavaArray<T> {}

impl<T: JavaType> JDeref for JavaArray<T> {
    fn jderef(&self) -> &Self {
        self
    }
}

impl<T: JavaType> TryJDeref for JavaArray<T> {
    type Java = Self;

    fn try_jderef(&self) -> Nullable<&Self> {
        Ok(self)
    }
}

// array.length isn't a normal field or method, so hand-generating the traits
pub trait JavaArrayExt<T: JavaType>: JvmOp {
    type Length: ScalarMethod<jni_sys::jsize>;
    fn length(self) -> Self::Length;
}

pub trait JavaArrayModificationExt<T: JavaType, RT: JavaScalar>: JvmOp {
    type SetArrayRegion<'a>: VoidMethod;
    fn set_array_region<'a>(self, start: usize, values: &'a[RT]) -> Self::SetArrayRegion<'a>;
}

impl<This, T> JavaArrayExt<T> for This
where
    This: JvmOp,
    for<'jvm> This::Output<'jvm>: AsJRef<JavaArray<T>>,
    T: JavaType,
{
    type Length = Length<Self, T>;
    fn length(self) -> Self::Length {
        Length {
            this: self,
            element: PhantomData,
        }
    }
}

#[derive_where::derive_where(Clone)]
#[derive_where(Copy; This: Copy)]
pub struct Length<This: JvmOp, T> {
    this: This,
    element: PhantomData<T>,
}

#[derive_where::derive_where(Clone)]
#[derive_where(Copy; This: Copy)]
pub struct SetArrayRegion<'a, This: JvmOp, T, RT> {
    this: This,
    element: PhantomData<T>,
    start: usize,
    values: &'a [RT],
}

impl<This, T> JvmOp for Length<This, T>
where
    This: JvmOp,
    for<'jvm> This::Output<'jvm>: AsJRef<JavaArray<T>>,
    T: JavaType,
{
    type Output<'jvm> = jni_sys::jsize;

    fn do_jni<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::LocalResult<'jvm, Self::Output<'jvm>> {
        let this = self.this.do_jni(jvm)?;
        let this = this.as_jref()?.as_raw();

        let len = unsafe {
            jvm.env()
                .invoke_unchecked(|env| env.GetArrayLength, |env, f| f(env, this.as_ptr()))
        };
        Ok(len)
    }
}

macro_rules! primitive_array {
    ($([$rust:ty]: $java_name:literal $java_ty:ident $new_fn:ident $get_fn:ident $set_fn:ident,)*) => {
        $(
            impl JvmOp for &[$rust] {
                type Output<'jvm> = Local<'jvm, JavaArray<$rust>>;

                fn do_jni<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::LocalResult<'jvm, Self::Output<'jvm>> {
                    let Ok(len) = self.len().try_into() else {
                        return Err(Error::SliceTooLong(self.len()))
                    };

                    let env = jvm.env();
                    let array: Option<Local<JavaArray<$rust>>> = unsafe {
                        // SAFETY: env points to an attached JNI
                        env.invoke(|env| env.$new_fn, |env, f| f(env, len))
                    }?;

                    let Some(array) = array else {
                        // NewArray should never return null unless an exception occurred (which we've already checked)
                        return Err(Error::JvmInternal(format!(
                            "failed to allocate `{}[{}]`",
                            $java_name,
                            len
                        )));
                    };

                    unsafe {
                        // SAFETY: we allocated an array with the same len and type as self
                        env.invoke_unchecked(|env| env.$set_fn, |env, f| f(
                            env,
                            array.as_raw().as_ptr(),
                            0,
                            len,
                            self.as_ptr().cast::<jni_sys::$java_ty>(),
                        ));
                    }

                    Ok(array)
                }
            }

            impl ToJavaImpl<java::Array<$rust>> for [$rust] {
                fn to_java_impl<'jvm>(
                    rust: &Self,
                    jvm: &mut Jvm<'jvm>,
                ) -> crate::LocalResult<'jvm, Option<Local<'jvm, java::Array<$rust>>>> {
                    Ok(Some(rust.do_jni(jvm)?))
                }
            }

            impl ToJavaImpl<java::Array<$rust>> for Vec<$rust> {
                fn to_java_impl<'jvm>(
                    rust: &Self,
                    jvm: &mut Jvm<'jvm>,
                ) -> crate::LocalResult<'jvm, Option<Local<'jvm, java::Array<$rust>>>> {
                    Ok(Some(rust.do_jni(jvm)?))
                }
            }

            impl IntoRust<Vec<$rust>> for &JavaArray<$rust> {
                fn into_rust<'jvm>(self, jvm: &mut Jvm<'jvm>) -> $crate::LocalResult<'jvm, Vec<$rust>> {
                    let len = self.length().do_jni(jvm)?;
                    let mut vec = Vec::<$rust>::with_capacity(len as usize);

                    unsafe {
                        // SAFETY: $rust is a Copy type and vec has at least as much capacity as the JVM array
                        jvm.env().invoke_unchecked(|env| env.$get_fn, |env, f| f(
                            env,
                            self.as_raw().as_ptr(),
                            0,
                            len,
                            vec.as_mut_ptr().cast::<jni_sys::$java_ty>(),
                        ));
                        vec.set_len(len as usize);
                    }

                    Ok(vec)
                }
            }

            impl<This> JavaArrayModificationExt<JavaArray<$rust>, $rust> for This
            where
                This: JvmOp,
                for<'jvm> This::Output<'jvm>: AsJRef<JavaArray<$rust>>,
            {
                type SetArrayRegion<'a> = SetArrayRegion<'a, Self, JavaArray<$rust>, $rust>;
                fn set_array_region<'a>(mut self, start: usize, values: &'a[$rust]) -> Self::SetArrayRegion<'a> {
                    SetArrayRegion {
                        this: self,
                        element: PhantomData,
                        start,
                        values
                    }
                }
            }

            impl<This> JvmOp for SetArrayRegion<'_, This, JavaArray<$rust>, $rust>
            where
                This: JvmOp,
                for<'jvm> This::Output<'jvm>: AsJRef<JavaArray<$rust>>,
            {
                type Output<'jvm> = ();
            
                fn do_jni<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::LocalResult<'jvm, Self::Output<'jvm>> {
                    let this = self.this.do_jni(jvm)?;
                    let this = this.as_jref()?.as_raw();
                
                    unsafe {
                        jvm.env().invoke_unchecked(|env| env.$set_fn, |env, f| f(
                            env,
                            this.as_ptr(),
                            self.start as jni_sys::jsize,
                            self.values.len() as jni_sys::jsize,
                            self.values.as_ptr().cast::<jni_sys::$java_ty>(),
                        ));
                    }
                
                    Ok(())
                }
            }
        )*
    };
}

// Bool is represented as u8 in JNI
primitive_array! {
    [bool]: "boolean" jboolean NewBooleanArray GetBooleanArrayRegion SetBooleanArrayRegion,
    [i8]: "byte" jbyte NewByteArray GetByteArrayRegion SetByteArrayRegion,
    [u16]: "char" jchar NewCharArray GetCharArrayRegion SetCharArrayRegion,
    [i16]: "short" jshort NewShortArray GetShortArrayRegion SetShortArrayRegion,
    [i32]: "int" jint NewIntArray GetIntArrayRegion SetIntArrayRegion,
    [i64]: "long" jlong NewLongArray GetLongArrayRegion SetLongArrayRegion,
    [f32]: "float" jfloat NewFloatArray GetFloatArrayRegion SetFloatArrayRegion,
    [f64]: "double" jdouble NewDoubleArray GetDoubleArrayRegion SetDoubleArrayRegion,
}
