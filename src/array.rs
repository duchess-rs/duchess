use std::marker::PhantomData;

use crate::{
    jvm::Upcast, ops::IntoJava, plumbing::JavaObjectExt, IntoRust, JavaObject, Jvm, JvmOp, Local,
};
use jni::{
    errors::{Error, JniError},
    objects::{AutoLocal, JObject, JPrimitiveArray},
};

pub struct JavaArray<T: ArrayElement> {
    _element: PhantomData<T>,
}

pub unsafe trait ArrayElement {}

unsafe impl<T: ArrayElement> JavaObject for JavaArray<T> {}

// Upcasts

unsafe impl<T: ArrayElement> Upcast<JavaArray<T>> for JavaArray<T> {}

macro_rules! primivite_array {
    ($([$rust:ty]: $new_fn:ident $get_fn:ident $set_fn:ident,)*) => {
        $(
            unsafe impl ArrayElement for $rust { }

            impl IntoJava<JavaArray<$rust>> for &[$rust] {
                type Output<'jvm> = Local<'jvm, JavaArray<$rust>>;

                fn into_java<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::Result<Self::Output<'jvm>> {
                    let env = jvm.to_env();
                    let Ok(len) = self.len().try_into() else {
                        return Err(Error::JniCall(JniError::InvalidArguments))
                    };
                    let array = env.$new_fn(len)?;
                    env.$set_fn(&array, 0, self)?;
                    unsafe { Ok(Local::from_jni(AutoLocal::new(JObject::from(array), &env))) }
                }
            }

            impl<J> IntoRust<Vec<$rust>> for J
            where
                for<'jvm> J: JvmOp<Input<'jvm> = ()>,
                for<'jvm> J::Output<'jvm>: AsRef<JavaArray<$rust>>,
            {
                fn into_rust(self, jvm: &mut Jvm<'_>) -> $crate::Result<Vec<$rust>> {
                    let object = self.execute_with(jvm, ())?;

                    let env = jvm.to_env();
                    // XX: safety, is this violating any rules? right way to cast?
                    let array_object = unsafe { JPrimitiveArray::from_raw(object.as_ref().as_jobject().as_raw()) };
                    let len = env.get_array_length(&array_object)? as usize;

                    // XX: use MaybeUninit somehow to avoid the zero'ing
                    let mut vec = vec![Default::default(); len];
                    env.$get_fn(&array_object, 0, &mut vec)?;

                    Ok(vec)
                }
            }
        )*
    };
}

primivite_array! {
    // [bool]: "[Z" new_boolean_array get_boolean_array_region get_boolean_array_region,
    [i8]: new_byte_array get_byte_array_region set_byte_array_region,
    [u16]: new_char_array get_char_array_region set_char_array_region,
    [i16]: new_short_array get_short_array_region set_short_array_region,
    [i32]: new_int_array get_int_array_region set_int_array_region,
    [i64]: new_long_array get_long_array_region set_long_array_region,
    [f32]: new_float_array get_float_array_region set_float_array_region,
    [f64]: new_double_array get_double_array_region set_double_array_region,
}

// Bool is represented as u8 in JNI

unsafe impl ArrayElement for bool {}

impl IntoJava<JavaArray<bool>> for &[bool] {
    type Output<'jvm> = Local<'jvm, JavaArray<bool>>;

    fn into_java<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::Result<Self::Output<'jvm>> {
        let env = jvm.to_env();
        let Ok(len) = self.len().try_into() else {
            return Err(Error::JniCall(JniError::InvalidArguments))
        };
        let array = env.new_boolean_array(len)?;
        // XX: is it possible to avoid this copy if we can make assumptions about bool repr?
        let u8s = self.iter().map(|&b| b as u8).collect::<Vec<_>>();
        env.set_boolean_array_region(&array, 0, &u8s)?;
        unsafe { Ok(Local::from_jni(AutoLocal::new(JObject::from(array), &env))) }
    }
}

impl<J> IntoRust<Vec<bool>> for J
where
    for<'jvm> J: JvmOp<Input<'jvm> = ()>,
    for<'jvm> J::Output<'jvm>: AsRef<JavaArray<bool>>,
{
    fn into_rust(self, jvm: &mut Jvm<'_>) -> crate::Result<Vec<bool>> {
        let object = self.execute_with(jvm, ())?;

        let env = jvm.to_env();
        // XX: safety, is this violating any rules? right way to cast?
        let array_object =
            unsafe { JPrimitiveArray::from_raw(object.as_ref().as_jobject().as_raw()) };
        let len = env.get_array_length(&array_object)? as usize;

        // XX: use MaybeUninit somehow to avoid the zero'ing
        let mut u8_vec = vec![0u8; len];
        env.get_boolean_array_region(&array_object, 0, &mut u8_vec)?;

        Ok(u8_vec.into_iter().map(|x| x != 0).collect())
    }
}
