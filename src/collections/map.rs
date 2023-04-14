use std::marker::PhantomData;

use jni::{
    objects::{AutoLocal, JMethodID, JValueGen, JClass},
    signature::ReturnType,
    sys::jvalue,
};
use once_cell::sync::OnceCell;

use crate::{
    plumbing::{JavaObjectExt, Upcast},
    IntoJava, IntoLocal, JavaObject, Jvm, JvmOp, Local, Global, JavaType, java::lang::Class,
};

// Ideally, we'd use duchess to derive these classes, but (a) we want to slap some nice interfaces to produce them from
// Rust structs and (b) they've got a lot of difficult cases (type params, overloaded methods, interfaces, etc.) that we
// wont' be able to tackle until later.

pub struct Map<K, V> {
    _markers: PhantomData<(K, V)>,
}

unsafe impl<K: JavaObject, V: JavaObject> JavaType for Map<K, V> {
    type ArrayClass<'jvm> = &'static Global<Class>;

    fn array_class<'jvm>(jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, Self::ArrayClass<'jvm>> {
        unimplemented!()
    }
}

unsafe impl<K: JavaObject, V: JavaObject> JavaObject for Map<K, V> {
    type Class<'jvm> = &'static Global<Class>;

    fn class<'jvm>(jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, Self::Class<'jvm>> {
        let env = jvm.to_env();

        static CLASS: OnceCell<Global<crate::java::lang::Class>> = OnceCell::new();
        CLASS.get_or_try_init(|| {
            let class = env.find_class("java/util/Map")?;
            // env.find_class() internally calls check_exception!()
            Ok(unsafe { Global::from_jni(env.new_global_ref(class)?) })
        })
    }
}

// Upcasts
unsafe impl<K: JavaObject, V: JavaObject> Upcast<Map<K, V>> for Map<K, V> {}
// unsafe impl<K: JavaObject, V: JavaObject> Upcast<Object> for Map<K, V> { }

pub struct HashMap<K, V> {
    _markers: PhantomData<(K, V)>,
}

unsafe impl<K: JavaObject, V: JavaObject> JavaType for HashMap<K, V> {
    type ArrayClass<'jvm> = &'static Global<Class>;

    fn array_class<'jvm>(jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, Self::ArrayClass<'jvm>> {
        unimplemented!()
    }
}

unsafe impl<K: JavaObject, V: JavaObject> JavaObject for HashMap<K, V> {
    type Class<'jvm> = &'static Global<Class>;

    fn class<'jvm>(jvm: &mut Jvm<'jvm>) -> crate::Result<'jvm, Self::Class<'jvm>> {
        let env = jvm.to_env();

        static CLASS: OnceCell<Global<crate::java::lang::Class>> = OnceCell::new();
        CLASS.get_or_try_init(|| {
            let class = env.find_class("java/util/HashMap")?;
            // env.find_class() internally calls check_exception!()
            Ok(unsafe { Global::from_jni(env.new_global_ref(class)?) })
        })
    }
}

// Upcasts
unsafe impl<K: JavaObject, V: JavaObject> Upcast<HashMap<K, V>> for HashMap<K, V> {}
unsafe impl<K: JavaObject, V: JavaObject> Upcast<Map<K, V>> for HashMap<K, V> {}
// unsafe impl<K: JavaObject, V: JavaObject> Upcast<AbstractMap<K, V>> for HashMap<K, V> { }
// unsafe impl<K: JavaObject, V: JavaObject> Upcast<Object> for HashMap<K, V> { }

impl<K, V> HashMap<K, V>
where
    K: JavaObject + 'static,
    V: JavaObject + 'static,
{
    pub fn new() -> impl IntoLocal<HashMap<K, V>> {
        struct Impl<K, V> {
            _markers: PhantomData<(K, V)>,
        }

        impl<K, V> Clone for Impl<K, V> {
            fn clone(&self) -> Self {
                Self {
                    _markers: PhantomData,
                }
            }
        }

        impl<K, V> JvmOp for Impl<K, V>
        where
            K: JavaObject + 'static,
            V: JavaObject + 'static,
        {
            type Input<'jvm> = ();
            type Output<'jvm> = Local<'jvm, HashMap<K, V>>;

            fn execute_with<'jvm>(
                self,
                jvm: &mut Jvm<'jvm>,
                (): (),
            ) -> crate::Result<'jvm, Self::Output<'jvm>> {
                let class = HashMap::<K, V>::class(jvm)?;
                let jclass = unsafe { JClass::from_raw(class.as_jobject().as_raw()) };
                let env = jvm.to_env();

                static CONSTRUCTOR: OnceCell<JMethodID> = OnceCell::new();
                let constructor =
                    CONSTRUCTOR.get_or_try_init(|| env.get_method_id(&jclass, "<init>", "()V"))?;

                let object = unsafe { env.new_object_unchecked(jclass, *constructor, &[])? };

                Ok(unsafe { Local::from_jni(AutoLocal::new(object, &env)) })
            }
        }

        Impl {
            _markers: PhantomData,
        }
    }
}

pub trait MapExt<K: JavaObject, V: JavaObject>: JvmOp + Sized {
    fn put<Key, Value>(self, key: Key, value: Value) -> MapPut<Self, K, V, Key, Value>
    where
        Key: IntoJava<K>,
        Value: IntoJava<V>,
    {
        MapPut {
            this: self,
            key,
            value,
            _markers: PhantomData,
        }
    }
}

impl<T: JvmOp, K: JavaObject, V: JavaObject> MapExt<K, V> for T where
    for<'jvm> T::Output<'jvm>: AsRef<Map<K, V>>
{
}

pub struct MapPut<T, K, V, Key, Value> {
    this: T,
    key: Key,
    value: Value,
    _markers: PhantomData<(K, V)>,
}

impl<T: Clone, K, V, Key: Clone, Value: Clone> Clone for MapPut<T, K, V, Key, Value> {
    fn clone(&self) -> Self {
        Self {
            this: self.this.clone(),
            key: self.key.clone(),
            value: self.value.clone(),
            _markers: PhantomData,
        }
    }
}

impl<T, K, V, Key, Value> JvmOp for MapPut<T, K, V, Key, Value>
where
    T: JvmOp,
    for<'jvm> T::Output<'jvm>: AsRef<Map<K, V>>,
    K: JavaObject + 'static,
    V: JavaObject + 'static,
    Key: IntoJava<K>,
    Value: IntoJava<V>,
{
    type Input<'jvm> = T::Input<'jvm>;
    type Output<'jvm> = Option<Local<'jvm, V>>;

    fn execute_with<'jvm>(
        self,
        jvm: &mut Jvm<'jvm>,
        input: Self::Input<'jvm>,
    ) -> crate::Result<'jvm, Self::Output<'jvm>> {
        let this = self.this.execute_with(jvm, input)?;
        let key = self.key.into_java(jvm)?;
        let value = self.value.into_java(jvm)?;
        let class = Map::<K, V>::class(jvm)?;
        let jclass = unsafe { JClass::from_raw(class.as_jobject().as_raw()) };

        let env = jvm.to_env();

        static METHOD: OnceCell<JMethodID> = OnceCell::new();
        let method = METHOD.get_or_try_init(|| {
            env.get_method_id(
                jclass,
                "put",
                "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
            )
        })?;

        // XX: safety?
        let result = unsafe {
            env.call_method_unchecked(
                this.as_ref().as_jobject(),
                *method,
                ReturnType::Object,
                &[
                    jvalue {
                        l: key.as_ref().as_jobject().as_raw(),
                    },
                    jvalue {
                        l: value.as_ref().as_jobject().as_raw(),
                    },
                ],
            )?
        };
        let JValueGen::Object(result) = result else {
            panic!("expected object for put() result");
        };

        Ok(if result.is_null() {
            None
        } else {
            Some(unsafe { Local::from_jni(AutoLocal::new(result, env)) })
        })
    }
}
