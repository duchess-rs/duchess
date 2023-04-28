// Ideally, we'd use duchess to derive these classes, but (a) we want to slap some nice interfaces to produce them from
// Rust structs and (b) they've got a lot of difficult cases (type params, overloaded methods, interfaces, etc.) that we
// wont' be able to tackle until later.

mod tmp {
    #[cfg(not(doc))]
    use crate as duchess;

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

      public interface java.util.Map<K, V> {
        public abstract int size();
        public abstract boolean isEmpty();
        public abstract boolean containsKey(java.lang.Object);
        public abstract boolean containsValue(java.lang.Object);
        public abstract V get(java.lang.Object);
        public abstract V put(K, V);
        public abstract V remove(java.lang.Object);
        public abstract void putAll(java.util.Map<? extends K, ? extends V>);
        public abstract void clear();
        // public abstract java.util.Set<K> keySet();
        // public abstract java.util.Collection<V> values();
        // public abstract java.util.Set<java.util.Map$Entry<K, V>> entrySet();
        public abstract boolean equals(java.lang.Object);
        public abstract int hashCode();
        public default V getOrDefault(java.lang.Object, V);
        // public default void forEach(java.util.function.BiConsumer<? super K, ? super V>);
        // public default void replaceAll(java.util.function.BiFunction<? super K, ? super V, ? extends V>);
        public default V putIfAbsent(K, V);
        // public default boolean remove(java.lang.Object, java.lang.Object);
        // public default boolean replace(K, V, V);
        // public default V replace(K, V);
        // // public default V computeIfAbsent(K, java.util.function.Function<? super K, ? extends V>);
        // public default V computeIfPresent(K, java.util.function.BiFunction<? super K, ? super V, ? extends V>);
        // public default V compute(K, java.util.function.BiFunction<? super K, ? super V, ? extends V>);
        // public default V merge(K, V, java.util.function.BiFunction<? super V, ? super V, ? extends V>);
        // public static <K, V> java.util.Map<K, V> of();
        // public static <K, V> java.util.Map<K, V> of(K, V);
        // public static <K, V> java.util.Map<K, V> of(K, V, K, V);
        // public static <K, V> java.util.Map<K, V> of(K, V, K, V, K, V);
        // public static <K, V> java.util.Map<K, V> of(K, V, K, V, K, V, K, V);
        // public static <K, V> java.util.Map<K, V> of(K, V, K, V, K, V, K, V, K, V);
        // public static <K, V> java.util.Map<K, V> of(K, V, K, V, K, V, K, V, K, V, K, V);
        // public static <K, V> java.util.Map<K, V> of(K, V, K, V, K, V, K, V, K, V, K, V, K, V);
        // public static <K, V> java.util.Map<K, V> of(K, V, K, V, K, V, K, V, K, V, K, V, K, V, K, V);
        // public static <K, V> java.util.Map<K, V> of(K, V, K, V, K, V, K, V, K, V, K, V, K, V, K, V, K, V);
        // public static <K, V> java.util.Map<K, V> of(K, V, K, V, K, V, K, V, K, V, K, V, K, V, K, V, K, V, K, V);
        // public static <K, V> java.util.Map<K, V> ofEntries(java.util.Map$Entry<? extends K, ? extends V>...);
        // public static <K, V> java.util.Map$Entry<K, V> entry(K, V);
        // public static <K, V> java.util.Map<K, V> copyOf(java.util.Map<? extends K, ? extends V>);
      }

      public class java.util.HashMap<K, V>
        // extends java.util.AbstractMap<K, V>
        implements java.util.Map<K, V> // , java.lang.Cloneable, java.io.Serializable
      {
        // public java.util.HashMap(int, float);
        // public java.util.HashMap(int);
        public java.util.HashMap();
        // public java.util.HashMap(java.util.Map<? extends K, ? extends V>);
        public int size();
        public boolean isEmpty();
        public V get(java.lang.Object);
        public boolean containsKey(java.lang.Object);
        public V put(K, V);
        public void putAll(java.util.Map<? extends K, ? extends V>);
        public V remove(java.lang.Object);
        public void clear();
        public boolean containsValue(java.lang.Object);
        // public java.util.Set<K> keySet();
        // public java.util.Collection<V> values();
        // public java.util.Set<java.util.Map$Entry<K, V>> entrySet();
        public V getOrDefault(java.lang.Object, V);
        public V putIfAbsent(K, V);
        // public boolean remove(java.lang.Object, java.lang.Object);
        // public boolean replace(K, V, V);
        // public V replace(K, V);
        // public V computeIfAbsent(K, java.util.function.Function<? super K, ? extends V>);
        // public V computeIfPresent(K, java.util.function.BiFunction<? super K, ? super V, ? extends V>);
        // public V compute(K, java.util.function.BiFunction<? super K, ? super V, ? extends V>);
        // public V merge(K, V, java.util.function.BiFunction<? super V, ? super V, ? extends V>);
        // public void forEach(java.util.function.BiConsumer<? super K, ? super V>);
        // public void replaceAll(java.util.function.BiFunction<? super K, ? super V, ? extends V>);
        public java.lang.Object clone();
      }


    }
}

pub use tmp::java::util::*;
