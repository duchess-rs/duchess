use std::{collections::HashMap, marker::PhantomData};

use crate::{
    cast::Upcast, from_ref::FromRef, java, jvm::JavaView, Error, Global, Jvm, JvmOp, Local,
};

pub trait ToJava {
    type JvmOp<'a, J>: for<'jvm> JvmOp<Output<'jvm> = Option<Local<'jvm, J>>>
        + std::ops::Deref<Target = <J as JavaView>::OfOp<Self::JvmOp<'a, J>>>
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

impl<R: ?Sized> ToJava for R {
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
pub struct ToJavaOp<'a, R: ?Sized, J> {
    rust: &'a R,
    phantom: PhantomData<J>,
}

impl<R, J> JvmOp for ToJavaOp<'_, R, J>
where
    R: ToJavaImpl<J> + ?Sized,
    J: Upcast<java::lang::Object> + Upcast<J>,
{
    type Output<'jvm> = Option<Local<'jvm, J>>;

    fn execute_with<'jvm>(self, jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, Self::Output<'jvm>> {
        R::to_java_impl(self.rust, jvm)
    }
}

impl<R, J> std::ops::Deref for ToJavaOp<'_, R, J>
where
    R: ToJavaImpl<J> + ?Sized,
    J: Upcast<java::lang::Object> + Upcast<J>,
{
    type Target = <J as JavaView>::OfOp<Self>;

    fn deref(&self) -> &Self::Target {
        <Self::Target as FromRef<_>>::from_ref(self)
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

impl<J> ToJavaImpl<J> for Local<'_, J>
where
    J: Upcast<java::lang::Object>,
{
    fn to_java_impl<'jvm>(
        rust: &Self,
        jvm: &mut Jvm<'jvm>,
    ) -> crate::Result<'jvm, Option<Local<'jvm, J>>> {
        Ok(Some(jvm.local(rust)))
    }
}

impl<J> ToJavaImpl<J> for Global<J>
where
    J: Upcast<java::lang::Object>,
{
    fn to_java_impl<'jvm>(
        rust: &Self,
        jvm: &mut Jvm<'jvm>,
    ) -> crate::Result<'jvm, Option<Local<'jvm, J>>> {
        Ok(Some(jvm.local(rust)))
    }
}

impl<J> ToJavaImpl<J> for &J
where
    J: Upcast<java::lang::Object>,
{
    fn to_java_impl<'jvm>(
        rust: &Self,
        jvm: &mut Jvm<'jvm>,
    ) -> crate::Result<'jvm, Option<Local<'jvm, J>>> {
        Ok(Some(jvm.local(rust)))
    }
}

impl<J, R> ToJavaImpl<J> for Option<R>
where
    J: Upcast<java::lang::Object>,
    R: ToJavaImpl<J>,
{
    fn to_java_impl<'jvm>(
        rust: &Self,
        jvm: &mut Jvm<'jvm>,
    ) -> crate::Result<'jvm, Option<Local<'jvm, J>>> {
        match rust {
            None => Ok(None),
            Some(r) => R::to_java_impl(r, jvm),
        }
    }
}

impl<J, R> ToJavaImpl<J> for crate::Result<'_, R>
where
    J: Upcast<java::lang::Object>,
    R: ToJavaImpl<J>,
{
    fn to_java_impl<'jvm>(
        rust: &Self,
        jvm: &mut Jvm<'jvm>,
    ) -> crate::Result<'jvm, Option<Local<'jvm, J>>> {
        match rust {
            Ok(r) => R::to_java_impl(r, jvm),
            Err(e) => match e {
                Error::Thrown(t) => Err(Error::Thrown(jvm.local(t))),
                Error::SliceTooLong(t) => Err(Error::SliceTooLong(*t)),
                Error::NullDeref => Err(Error::NullDeref),
                Error::NestedUsage => Err(Error::NestedUsage),
                Error::JvmAlreadyExists => Err(Error::JvmAlreadyExists),
                Error::UnableToLoadLibjvm(t) => Err(Error::UnableToLoadLibjvm(
                    format!("UnableToLoadLibjvm({t:?})").as_str().into(), // FIXME: should to_java_impl be `self` ?
                )),
                Error::JvmInternal(t) => Err(Error::JvmInternal(t.clone())),
            },
        }
    }
}

impl<J, R> ToJavaImpl<J> for crate::GlobalResult<R>
where
    J: Upcast<java::lang::Object>,
    R: ToJavaImpl<J>,
{
    fn to_java_impl<'jvm>(
        rust: &Self,
        jvm: &mut Jvm<'jvm>,
    ) -> crate::Result<'jvm, Option<Local<'jvm, J>>> {
        match rust {
            Ok(r) => R::to_java_impl(r, jvm),
            Err(e) => match e {
                Error::Thrown(t) => Err(Error::Thrown(jvm.local(t))),
                Error::SliceTooLong(t) => Err(Error::SliceTooLong(*t)),
                Error::NullDeref => Err(Error::NullDeref),
                Error::NestedUsage => Err(Error::NestedUsage),
                Error::JvmAlreadyExists => Err(Error::JvmAlreadyExists),
                Error::UnableToLoadLibjvm(t) => Err(Error::UnableToLoadLibjvm(
                    format!("UnableToLoadLibjvm({t:?})").as_str().into(), // FIXME: should to_java_impl be `self` ?
                )),
                Error::JvmInternal(t) => Err(Error::JvmInternal(t.clone())),
            },
        }
    }
}
