use std::marker::PhantomData;

use jni::{
    objects::{AutoLocal, GlobalRef, JMethodID, JValueGen},
    signature::ReturnType,
    sys::jvalue,
};
use once_cell::sync::OnceCell;

use crate::{
    plumbing::{JavaObjectExt, Upcast},
    IntoJava, JavaObject, Jvm, JvmOp, Local,
};

// Ideally, we'd use duchess to derive these classes, but (a) we want to slap some nice interfaces to produce them from
// Rust structs and (b) they've got a lot of difficult cases (type params, overloaded methods, interfaces, etc.) that we
// wont' be able to tackle until later.

pub struct Map<K, V> {
    _markers: PhantomData<(K, V)>,
}

unsafe impl<K: JavaObject, V: JavaObject> JavaObject for Map<K, V> {}

// Upcasts
unsafe impl<K: JavaObject, V: JavaObject> Upcast<Map<K, V>> for Map<K, V> {}
// unsafe impl<K: JavaObject, V: JavaObject> Upcast<Object> for Map<K, V> { }

pub struct HashMap<K, V> {
    _markers: PhantomData<(K, V)>,
}

unsafe impl<K: JavaObject, V: JavaObject> JavaObject for HashMap<K, V> {}

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
    pub fn new() -> impl for<'jvm> JvmOp<Input<'jvm> = (), Output<'jvm> = Local<'jvm, HashMap<K, V>>>
    {
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
            ) -> crate::Result<Self::Output<'jvm>> {
                let class = hash_map_class(jvm)?;
                let env = jvm.to_env();

                static CONSTRUCTOR: OnceCell<JMethodID> = OnceCell::new();
                let constructor =
                    CONSTRUCTOR.get_or_try_init(|| env.get_method_id(class, "<init>", "()V"))?;

                let object = unsafe { env.new_object_unchecked(class, *constructor, &[])? };

                Ok(unsafe { Local::from_jni(AutoLocal::new(object, &env)) })
            }
        }

        Impl {
            _markers: PhantomData,
        }
    }
}

// XX: ideally these are wrapped as JavaClass<HashMap<?, ?>>

fn hash_map_class(jvm: &mut Jvm<'_>) -> crate::Result<&'static GlobalRef> {
    let env = jvm.to_env();

    static CLASS: OnceCell<GlobalRef> = OnceCell::new();
    CLASS.get_or_try_init(|| {
        let class = env.find_class("java/util/HashMap")?;
        env.new_global_ref(class)
    })
}

fn map_class(jvm: &mut Jvm<'_>) -> crate::Result<&'static GlobalRef> {
    let env = jvm.to_env();

    static CLASS: OnceCell<GlobalRef> = OnceCell::new();
    CLASS.get_or_try_init(|| {
        let class = env.find_class("java/util/Map")?;
        env.new_global_ref(class)
    })
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
    ) -> crate::Result<Self::Output<'jvm>> {
        let this = self.this.execute_with(jvm, input)?;
        let key = self.key.into_java(jvm)?;
        let value = self.value.into_java(jvm)?;
        let class = map_class(jvm)?;

        let env = jvm.to_env();

        static METHOD: OnceCell<JMethodID> = OnceCell::new();
        let method = METHOD.get_or_try_init(|| {
            env.get_method_id(
                class,
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
