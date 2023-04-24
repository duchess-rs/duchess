#[cfg(not(doc))]
use crate as duchess;

// Ideally, we'd use duchess to derive these classes, but (a) we want to slap some nice interfaces to produce them from
// Rust structs and (b) they've got a lot of difficult cases (type params, overloaded methods, interfaces, etc.) that we
// wont' be able to tackle until later.

mod tmp {
    use super::*;
    duchess_macro::java_package! {
      package java.util;

      public interface java.util.List<E> {
        public abstract int size();
        public abstract boolean isEmpty();
        public abstract boolean contains(java.lang.Object);
        public abstract <T> T[] toArray(T[]);
        public abstract boolean add(E);
        public abstract boolean remove(java.lang.Object);
        public abstract void clear();
        public abstract boolean equals(java.lang.Object);
        public abstract int hashCode();
        public abstract E get(int);
        public abstract E set(int, E);
        public abstract int indexOf(java.lang.Object);
        public abstract int lastIndexOf(java.lang.Object);
        public abstract java.util.List<E> subList(int, int);
        public static <E> java.util.List<E> of(E...);
      }

      public class java.util.ArrayList<E> implements java.util.List<E> {
        public java.util.ArrayList();
        public void trimToSize();
        public void ensureCapacity(int);
        public int size();
        public boolean isEmpty();
        public boolean contains(java.lang.Object);
        public int indexOf(java.lang.Object);
        public int lastIndexOf(java.lang.Object);
        public java.lang.Object clone();
        public java.lang.Object[] toArray();
        public E get(int);
        public E set(int, E);
        public boolean add(E);
        public boolean equals(java.lang.Object);
        public int hashCode();
        public boolean remove(java.lang.Object);
        public void clear();
        public java.util.List<E> subList(int, int);
      }
    }
}

pub use tmp::java::util::*;
