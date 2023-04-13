use std::marker::PhantomData;

use jni::objects::{AutoLocal, GlobalRef, JMethodID};
use once_cell::sync::OnceCell;

use crate::{java, plumbing::Upcast, JavaObject, Jvm, JvmOp, Local};

#[cfg(not(doc))]
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
