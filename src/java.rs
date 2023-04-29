mod auto {
    // Make current crate available as `duchess` for use by the generated code.
    // NB. in documentation mode, the current crate is already available as duchess.
    #[cfg(not(doc))]
    use crate as duchess;

    duchess_macro::java_package! {
        package java.lang;

        public class java.lang.Object {
            public java.lang.Object();
            public native int hashCode();
            public boolean equals(java.lang.Object);
            public java.lang.String toString();
            public final native void notify();
            public final native void notifyAll();
        }

        public class java.lang.Throwable {
            public java.lang.Throwable();
            public java.lang.String getMessage();
            public java.lang.String getLocalizedMessage();
            public synchronized java.lang.Throwable getCause();
            public synchronized java.lang.Throwable initCause(java.lang.Throwable);
            public java.lang.String toString();
            public void printStackTrace();
            public synchronized java.lang.Throwable fillInStackTrace();
            public java.lang.StackTraceElement[] getStackTrace();
            public void setStackTrace(java.lang.StackTraceElement[]);
            public final synchronized void addSuppressed(java.lang.Throwable);
            public final synchronized java.lang.Throwable[] getSuppressed();
        }

        public final class java.lang.StackTraceElement {
            public java.lang.StackTraceElement(java.lang.String, java.lang.String, java.lang.String, int);
            public java.lang.String getFileName();
            public int getLineNumber();
            public java.lang.String getModuleName();
            public java.lang.String getModuleVersion();
            public java.lang.String getClassLoaderName();
            public java.lang.String getClassName();
            public java.lang.String getMethodName();
            public boolean isNativeMethod();
            public java.lang.String toString();
            public boolean equals(java.lang.Object);
            public int hashCode();
        }

        public class java.lang.Exception extends java.lang.Throwable {
            public java.lang.Exception();
        }

        public class java.lang.RuntimeException extends java.lang.Exception {
            public java.lang.RuntimeException();
        }

        // NB: In Java, this is `Class<T>`, but we model it as the erased version
        // `Class`. This is beacuse there are a lot of methods, including some that we would
        // like to model such as `arrayType()`, that return a `Class<?>`, and we cannot model
        // `?` in return position. By erasing the type parameter, we permit users to just
        // write `java.lang.Class` for those methods, but this does mean that some of the fancier
        // reflection types in Java won't work.
        //
        // FIXME(#41): It's not clear that this is the best solution, and we may revisit it in the future,
        // perhaps by not modeling `arrayType()` and friends, or perhaps by finding some way to
        // model `?` in return types in a satisfactory way.
        public final class java.lang.Class {
            public java.lang.String toString();
            public java.lang.String toGenericString();
            public native boolean isInstance(java.lang.Object);
            public native boolean isInterface();
            public native boolean isArray();
            public native boolean isPrimitive();
            public boolean isAnnotation();
            public boolean isSynthetic();
            public java.lang.String getName();
            public native java.lang.Class getSuperclass();
            public java.lang.String getPackageName();
            public java.lang.Class[] getInterfaces();
            public java.lang.Class getComponentType();
            public java.lang.Class arrayType();
        }

        public final class java.lang.String {
            public java.lang.String(byte[]);
            public int length();
            public boolean isEmpty();
        }

        public abstract class java.lang.Record {
            public abstract boolean equals(java.lang.Object);
            public abstract int hashCode();
            public abstract java.lang.String toString();
        }


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

pub use auto::java::*;

// XX this isn't a real class in the JVM, since each array type (e.g. Foo[] and int[]) is just a subclass of Object.
// Should it go somewhere outside of the JDK core classes?
pub use crate::array::JavaArray as Array;
pub use crate::array::JavaArrayExt as ArrayExt;
