use crate::{java, JavaObject};

#[cfg(not(doc))]
use crate as duchess;

// Ideally, we'd use duchess to derive these classes, but (a) we want to slap some nice interfaces to produce them from
// Rust structs and (b) they've got a lot of difficult cases (type params, overloaded methods, interfaces, etc.) that we
// wont' be able to tackle until later.

duchess_macro::duchess_javap! {
    r#"
Compiled from "List.java"
public interface java.util.List<E> {
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

duchess_macro::duchess_javap! {
    r#"
    Compiled from "ArrayList.java"
public class java.util.ArrayList<E> implements java.util.List<E> {
  public java.util.ArrayList();
    descriptor: ()V

  public void trimToSize();
    descriptor: ()V

  public void ensureCapacity(int);
    descriptor: (I)V

  public int size();
    descriptor: ()I

  public boolean isEmpty();
    descriptor: ()Z

  public boolean contains(java.lang.Object);
    descriptor: (Ljava/lang/Object;)Z

  public int indexOf(java.lang.Object);
    descriptor: (Ljava/lang/Object;)I

  public int lastIndexOf(java.lang.Object);
    descriptor: (Ljava/lang/Object;)I

  public java.lang.Object clone();
    descriptor: ()Ljava/lang/Object;

  public java.lang.Object[] toArray();
    descriptor: ()[Ljava/lang/Object;

  public E get(int);
    descriptor: (I)Ljava/lang/Object;

  public E set(int, E);
    descriptor: (ILjava/lang/Object;)Ljava/lang/Object;

  public boolean add(E);
    descriptor: (Ljava/lang/Object;)Z

  public boolean equals(java.lang.Object);
    descriptor: (Ljava/lang/Object;)Z

  public int hashCode();
    descriptor: ()I

  public boolean remove(java.lang.Object);
    descriptor: (Ljava/lang/Object;)Z

  public void clear();
    descriptor: ()V

  public java.util.List<E> subList(int, int);
    descriptor: (II)Ljava/util/List;
}
    "#
}
