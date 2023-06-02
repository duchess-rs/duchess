use std::{collections::HashMap, marker::PhantomData};

use crate::{cast::Upcast, java, Jvm, JvmOp, Local};

pub trait ToJava: Sized {
    type JvmOp<'a, J>: for<'jvm> JvmOp<Output<'jvm> = Option<Local<'jvm, J>>>
    where
        Self: 'a,
        Self: ToJavaImpl<J>,
        J: Upcast<java::lang::Object> + Upcast<J>;

    fn to_java<J>(&self) -> Self::JvmOp<'_, J>
    where
        Self: ToJavaImpl<J>,
        J: Upcast<java::lang::Object> + Upcast<J>;
}

pub trait ToJavaImpl<J>
where
    J: Upcast<java::lang::Object>,
{
    fn to_java_impl<'jvm>(
        rust: &Self,
        jvm: &mut Jvm<'jvm>,
    ) -> crate::Result<'jvm, Option<Local<'jvm, J>>>;
}

impl<R> ToJava for R {
    type JvmOp<'a, J> = ToJavaOp<'a, R, J>
    where
        Self: 'a,
        Self: ToJavaImpl<J>,
        J: Upcast<java::lang::Object> + Upcast<J>;

    fn to_java<J>(&self) -> ToJavaOp<'_, Self, J>
    where
        Self: ToJavaImpl<J>,
        J: Upcast<java::lang::Object> + Upcast<J>,
    {
        ToJavaOp {
            rust: self,
            phantom: PhantomData,
        }
    }
}

#[derive_where::derive_where(Copy, Clone)]
pub struct ToJavaOp<'a, R, J> {
    rust: &'a R,
    phantom: PhantomData<J>,
}

impl<R, J> JvmOp for ToJavaOp<'_, R, J>
where
    R: ToJavaImpl<J>,
    J: Upcast<java::lang::Object> + Upcast<J>,
{
    type Output<'jvm> = Option<Local<'jvm, J>>;

    fn execute_with<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, Self::Output<'jvm>> {
        R::to_java_impl(self.rust, jvm)
    }
}

impl<K, V, JK, JV, S> ToJavaImpl<java::util::HashMap<JK, JV>> for HashMap<K, V, S>
where
    K: ToJavaImpl<JK>,
    V: ToJavaImpl<JV>,
    JK: Upcast<java::lang::Object> + Upcast<JK>,
    JV: Upcast<java::lang::Object> + Upcast<JV>,
{
    fn to_java_impl<'jvm>(
        rust: &Self,
        jvm: &mut Jvm<'jvm>,
    ) -> crate::Result<'jvm, Option<Local<'jvm, java::util::HashMap<JK, JV>>>> {
        let jmap: Local<'jvm, java::util::HashMap<JK, JV>> =
            java::util::HashMap::new().execute_with(jvm)?;
        for (key, value) in rust {
            jmap.put(key.to_java(), value.to_java()).execute_with(jvm)?;
        }
        Ok(Some(jmap))
    }
}

impl<K, V, JK, JV, S> ToJavaImpl<java::util::Map<JK, JV>> for HashMap<K, V, S>
where
    K: ToJavaImpl<JK>,
    V: ToJavaImpl<JV>,
    JK: Upcast<java::lang::Object> + Upcast<JK>,
    JV: Upcast<java::lang::Object> + Upcast<JV>,
{
    fn to_java_impl<'jvm>(
        rust: &Self,
        jvm: &mut Jvm<'jvm>,
    ) -> crate::Result<'jvm, Option<Local<'jvm, java::util::Map<JK, JV>>>> {
        Ok(Some(
            rust.to_java::<java::util::HashMap<JK, JV>>()
                .assert_not_null()
                .upcast()
                .execute_with(jvm)?,
        ))
    }
}

impl<E, JE> ToJavaImpl<java::util::ArrayList<JE>> for Vec<E>
where
    E: ToJavaImpl<JE>,
    JE: Upcast<java::lang::Object> + Upcast<JE>,
{
    fn to_java_impl<'jvm>(
        rust: &Self,
        jvm: &mut Jvm<'jvm>,
    ) -> crate::Result<'jvm, Option<Local<'jvm, java::util::ArrayList<JE>>>> {
        let jvec: Local<'jvm, java::util::ArrayList<JE>> =
            java::util::ArrayList::new().execute_with(jvm)?;
        for element in rust {
            jvec.add(element.to_java()).execute_with(jvm)?;
        }
        Ok(Some(jvec))
    }
}

impl<E, JE> ToJavaImpl<java::util::List<JE>> for Vec<E>
where
    E: ToJavaImpl<JE>,
    JE: Upcast<java::lang::Object> + Upcast<JE>,
{
    fn to_java_impl<'jvm>(
        rust: &Self,
        jvm: &mut Jvm<'jvm>,
    ) -> crate::Result<'jvm, Option<Local<'jvm, java::util::List<JE>>>> {
        Ok(Some(
            rust.to_java::<java::util::ArrayList<JE>>()
                .assert_not_null()
                .upcast()
                .execute_with(jvm)?,
        ))
    }
}

impl ToJavaImpl<java::lang::String> for String {
    fn to_java_impl<'jvm>(
        rust: &Self,
        jvm: &mut Jvm<'jvm>,
    ) -> crate::Result<'jvm, Option<Local<'jvm, java::lang::String>>> {
        str::to_java_impl(rust, jvm)
    }
}

impl ToJavaImpl<java::lang::String> for str {
    fn to_java_impl<'jvm>(
        rust: &Self,
        jvm: &mut Jvm<'jvm>,
    ) -> crate::Result<'jvm, Option<Local<'jvm, java::lang::String>>> {
        let jstr = rust.execute_with(jvm)?;
        Ok(Some(jstr))
    }
}

impl ToJavaImpl<java::Array<i8>> for Vec<u8> {
    fn to_java_impl<'jvm>(
        rust: &Self,
        jvm: &mut Jvm<'jvm>,
    ) -> crate::Result<'jvm, Option<Local<'jvm, java::Array<i8>>>> {
        let this: &Vec<i8> = unsafe { std::mem::transmute(rust) };
        ToJavaImpl::to_java_impl(this, jvm)
    }
}
