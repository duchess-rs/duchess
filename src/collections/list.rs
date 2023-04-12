use std::marker::PhantomData;

use jni::{
    objects::{AutoLocal, GlobalRef, JMethodID, JValueGen},
    signature::{Primitive, ReturnType},
    sys::jvalue,
};
use once_cell::sync::OnceCell;

use crate::{
    java,
    plumbing::{JavaObjectExt, Upcast},
    IntoJava, JavaObject, Jvm, JvmOp, Local,
};

use crate as duchess;

// Ideally, we'd use duchess to derive these classes, but (a) we want to slap some nice interfaces to produce them from
// Rust structs and (b) they've got a lot of difficult cases (type params, overloaded methods, interfaces, etc.) that we
// wont' be able to tackle until later.

duchess_macro::duchess_javap! {
    r#"
Compiled from "List.java"
public interface java.util.List<E> extends java.util.Collection<E> {
  public abstract int size();
    descriptor: ()I

  public abstract boolean isEmpty();
    descriptor: ()Z

  public abstract boolean contains(java.lang.Object);
    descriptor: (Ljava/lang/Object;)Z

  public abstract <T> T[] toArray(T[]);
    descriptor: ([Ljava/lang/Object;)[Ljava/lang/Object;

  public abstract boolean add(E);
    descriptor: (Ljava/lang/Object;)Z

  public abstract boolean remove(java.lang.Object);
    descriptor: (Ljava/lang/Object;)Z

  public abstract void clear();
    descriptor: ()V

  public abstract boolean equals(java.lang.Object);
    descriptor: (Ljava/lang/Object;)Z

  public abstract int hashCode();
    descriptor: ()I

  public abstract E get(int);
    descriptor: (I)Ljava/lang/Object;

  public abstract E set(int, E);
    descriptor: (ILjava/lang/Object;)Ljava/lang/Object;

  public abstract int indexOf(java.lang.Object);
    descriptor: (Ljava/lang/Object;)I

  public abstract int lastIndexOf(java.lang.Object);
    descriptor: (Ljava/lang/Object;)I

  public abstract java.util.List<E> subList(int, int);
    descriptor: (II)Ljava/util/List;

  public static <E> java.util.List<E> of(E...);
    descriptor: ([Ljava/lang/Object;)Ljava/util/List;
}
    "#
}

pub struct List<T> {
    _markers: PhantomData<T>,
}

unsafe impl<T: JavaObject> JavaObject for List<T> {}

// Upcasts
unsafe impl<T: JavaObject> Upcast<List<T>> for List<T> {}
// unsafe impl<T: JavaObject> Upcast<Collection<T>> for List<T> {}
// unsafe impl<T: JavaObject> Upcast<Object> for List<T> {}

pub struct ArrayList<T> {
    _markers: PhantomData<T>,
}

unsafe impl<T: JavaObject> JavaObject for ArrayList<T> {}

// Upcasts
unsafe impl<T: JavaObject> Upcast<ArrayList<T>> for ArrayList<T> {}
unsafe impl<T: JavaObject> Upcast<List<T>> for ArrayList<T> {}
// unsafe impl<T: JavaObject> Upcast<Collection<T>> for ArrayList<T> {}
// unsafe impl<T: JavaObject> Upcast<Object> for ArrayList<T> {}

impl<T> ArrayList<T>
where
    T: JavaObject + 'static,
{
    pub fn new() -> impl for<'jvm> JvmOp<Input<'jvm> = (), Output<'jvm> = Local<'jvm, ArrayList<T>>>
    {
        struct Impl<T> {
            _markers: PhantomData<T>,
        }

        impl<T> Clone for Impl<T> {
            fn clone(&self) -> Self {
                Self {
                    _markers: PhantomData,
                }
            }
        }

        impl<T> JvmOp for Impl<T>
        where
            T: JavaObject + 'static,
        {
            type Input<'jvm> = ();
            type Output<'jvm> = Local<'jvm, ArrayList<T>>;

            fn execute_with<'jvm>(
                self,
                jvm: &mut Jvm<'jvm>,
                (): (),
            ) -> crate::Result<Self::Output<'jvm>> {
                let class = array_list_class(jvm)?;
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

fn array_list_class(jvm: &mut Jvm<'_>) -> crate::Result<&'static GlobalRef> {
    let env = jvm.to_env();

    static CLASS: OnceCell<GlobalRef> = OnceCell::new();
    CLASS.get_or_try_init(|| {
        let class = env.find_class("java/util/ArrayList")?;
        env.new_global_ref(class)
    })
}

fn list_class(jvm: &mut Jvm<'_>) -> crate::Result<&'static GlobalRef> {
    let env = jvm.to_env();

    static CLASS: OnceCell<GlobalRef> = OnceCell::new();
    CLASS.get_or_try_init(|| {
        let class = env.find_class("java/util/List")?;
        env.new_global_ref(class)
    })
}

pub trait ListExt<T: JavaObject>: JvmOp + Sized {
    fn add<Element>(self, element: Element) -> ListAdd<Self, T, Element>
    where
        Element: IntoJava<T>,
    {
        ListAdd {
            this: self,
            element,
            _markers: PhantomData,
        }
    }
}

impl<J: JvmOp, T: JavaObject> ListExt<T> for J where for<'jvm> J::Output<'jvm>: AsRef<List<T>> {}

pub struct ListAdd<J, T, Element> {
    this: J,
    element: Element,
    _markers: PhantomData<T>,
}

impl<J: Clone, T, Element: Clone> Clone for ListAdd<J, T, Element> {
    fn clone(&self) -> Self {
        Self {
            this: self.this.clone(),
            element: self.element.clone(),
            _markers: PhantomData,
        }
    }
}

impl<J, T, Element> JvmOp for ListAdd<J, T, Element>
where
    J: JvmOp,
    for<'jvm> J::Output<'jvm>: AsRef<List<T>>,
    T: JavaObject + 'static,
    Element: IntoJava<T>,
{
    type Input<'jvm> = J::Input<'jvm>;
    type Output<'jvm> = bool;

    fn execute_with<'jvm>(
        self,
        jvm: &mut Jvm<'jvm>,
        input: Self::Input<'jvm>,
    ) -> crate::Result<Self::Output<'jvm>> {
        let this = self.this.execute_with(jvm, input)?;
        let element = self.element.into_java(jvm)?;
        let class = list_class(jvm)?;

        let env = jvm.to_env();

        static METHOD: OnceCell<JMethodID> = OnceCell::new();
        let method =
            METHOD.get_or_try_init(|| env.get_method_id(class, "add", "(Ljava/lang/Object;)Z"))?;

        // XX: safety?
        let result = unsafe {
            env.call_method_unchecked(
                this.as_ref().as_jobject(),
                *method,
                ReturnType::Primitive(Primitive::Boolean),
                &[jvalue {
                    l: element.as_ref().as_jobject().as_raw(),
                }],
            )?
        };
        let JValueGen::Bool(result) = result else {
            panic!("expected object for put() result");
        };

        Ok(result != 0)
    }
}
