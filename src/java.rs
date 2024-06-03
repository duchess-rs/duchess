mod auto {
    // Make current crate available as `duchess` for use by the generated code.
    // NB. in doctests, the current crate is already available as duchess.
    #[cfg(not(doctest))]
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
            // public native boolean isAssignableFrom(java.lang.Class<?>);
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

        public final class java.lang.Long {
            public static long parseLong(java.lang.String) throws java.lang.NumberFormatException;
            public static long parseUnsignedLong(java.lang.String) throws java.lang.NumberFormatException;
            public static java.lang.Long valueOf(java.lang.String) throws java.lang.NumberFormatException;
            public static java.lang.Long decode(java.lang.String) throws java.lang.NumberFormatException;
            public java.lang.Long(java.lang.String) throws java.lang.NumberFormatException;
            public byte byteValue();
            public short shortValue();
            public int intValue();
            public long longValue();
            public float floatValue();
            public double doubleValue();
            public static java.lang.Long getLong(java.lang.String);
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

            // FIXME: Java generics from static methods are totally
            // disjoint from the enclosing Self type, but not in Rust.
            // How do we bridge this gap most ergonomically?
            //
            // public static <E> java.util.List<E> of(E...);
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

        public class java.util.Date { // implements java.io.Serializable, java.lang.Cloneable, java.lang.Comparable<java.util.Date> {
            public java.util.Date();
            //   public java.util.Date(long);
            //   public java.util.Date(int, int, int);
            //   public java.util.Date(int, int, int, int, int);
            //   public java.util.Date(int, int, int, int, int, int);
            //   public java.util.Date(java.lang.String);
            // public java.lang.Object clone();
            public static long UTC(int, int, int, int, int, int);
            public static long parse(java.lang.String);
            public int getYear();
            public void setYear(int);
            public int getMonth();
            public void setMonth(int);
            public int getDate();
            public void setDate(int);
            public int getDay();
            public int getHours();
            public void setHours(int);
            public int getMinutes();
            public void setMinutes(int);
            public int getSeconds();
            public void setSeconds(int);
            public long getTime();
            public void setTime(long);
            public boolean before(java.util.Date);
            public boolean after(java.util.Date);
            public boolean equals(java.lang.Object);
            // static final long getMillisOf(java.util.Date);
            public int compareTo(java.util.Date);
            public int hashCode();
            public java.lang.String toString();
            public java.lang.String toLocaleString();
            public java.lang.String toGMTString();
            public int getTimezoneOffset();
            // public static java.util.Date from(java.time.Instant);
            // public java.time.Instant toInstant();
            // public int compareTo(java.lang.Object);
            //   static {};
        }

        package java.time;

        public final class java.time.Instant {
            public static final java.time.Instant EPOCH;
            public static final java.time.Instant MIN;
            public static final java.time.Instant MAX;
            public static java.time.Instant now();
            // public static java.time.Instant now(java.time.Clock);
            // public static java.time.Instant ofEpochSecond(long);
            public static java.time.Instant ofEpochSecond(long, long);
            public static java.time.Instant ofEpochMilli(long);
            // public static java.time.Instant from(java.time.temporal.TemporalAccessor);
            // public static java.time.Instant parse(java.lang.CharSequence);
            // public boolean isSupported(java.time.temporal.TemporalField);
            // public boolean isSupported(java.time.temporal.TemporalUnit);
            // public java.time.temporal.ValueRange range(java.time.temporal.TemporalField);
            // public int get(java.time.temporal.TemporalField);
            // public long getLong(java.time.temporal.TemporalField);
            public long getEpochSecond();
            public int getNano();
            // public java.time.Instant with(java.time.temporal.TemporalAdjuster);
            // public java.time.Instant with(java.time.temporal.TemporalField, long);
            // public java.time.Instant truncatedTo(java.time.temporal.TemporalUnit);
            // public java.time.Instant plus(java.time.temporal.TemporalAmount);
            // public java.time.Instant plus(long, java.time.temporal.TemporalUnit);
            public java.time.Instant plusSeconds(long);
            public java.time.Instant plusMillis(long);
            public java.time.Instant plusNanos(long);
            // public java.time.Instant minus(java.time.temporal.TemporalAmount);
            // public java.time.Instant minus(long, java.time.temporal.TemporalUnit);
            public java.time.Instant minusSeconds(long);
            public java.time.Instant minusMillis(long);
            public java.time.Instant minusNanos(long);
            // public <R> R query(java.time.temporal.TemporalQuery<R>);
            // public java.time.temporal.Temporal adjustInto(java.time.temporal.Temporal);
            // public long until(java.time.temporal.Temporal, java.time.temporal.TemporalUnit);
            // public java.time.OffsetDateTime atOffset(java.time.ZoneOffset);
            // public java.time.ZonedDateTime atZone(java.time.ZoneId);
            public long toEpochMilli();
            public int compareTo(java.time.Instant);
            public boolean isAfter(java.time.Instant);
            public boolean isBefore(java.time.Instant);
            public boolean equals(java.lang.Object);
            public int hashCode();
            public java.lang.String toString();
            // void writeExternal(java.io.DataOutput) throws java.io.IOException;
            // static java.time.Instant readExternal(java.io.DataInput) throws java.io.IOException;
            // public java.time.temporal.Temporal minus(long, java.time.temporal.TemporalUnit);
            // public java.time.temporal.Temporal minus(java.time.temporal.TemporalAmount);
            // public java.time.temporal.Temporal plus(long, java.time.temporal.TemporalUnit);
            // public java.time.temporal.Temporal plus(java.time.temporal.TemporalAmount);
            // public java.time.temporal.Temporal with(java.time.temporal.TemporalField, long);
            // public java.time.temporal.Temporal with(java.time.temporal.TemporalAdjuster);
            // public int compareTo(java.lang.Object);
        }

        package java.lang.management;

        public interface java.lang.management.MemoryManagerMXBean {
            public abstract java.lang.String getName();
            public abstract boolean isValid();
            public abstract java.lang.String[] getMemoryPoolNames();
        }

        interface GarbageCollectorMXBean extends java.lang.management.MemoryManagerMXBean {
            public long getCollectionCount();
            public long getCollectionTime();
        }

        public class ManagementFactory {
            public static java.util.List<java.lang.management.GarbageCollectorMXBean> getGarbageCollectorMXBeans();
            public static java.util.List<java.lang.management.MemoryPoolMXBean> getMemoryPoolMXBeans();
            public static java.lang.management.MemoryMXBean getMemoryMXBean();
        }

        public interface java.lang.management.MemoryMXBean {
            public abstract int getObjectPendingFinalizationCount();
            public abstract java.lang.management.MemoryUsage getHeapMemoryUsage();
            public abstract java.lang.management.MemoryUsage getNonHeapMemoryUsage();
            public abstract boolean isVerbose();
            public abstract void setVerbose(boolean);
            public abstract void gc();
        }

        public interface java.lang.management.MemoryPoolMXBean {
            public abstract java.lang.String getName();
            public abstract java.lang.management.MemoryType getType();
            public abstract java.lang.management.MemoryUsage getUsage();
            public abstract java.lang.management.MemoryUsage getCollectionUsage();
            public abstract java.lang.management.MemoryUsage getPeakUsage();
        }

        public class java.lang.management.MemoryUsage {
            private final long init;
            private final long used;
            private final long committed;
            private final long max;
            public java.lang.management.MemoryUsage(long, long, long, long);
            public long getInit();
            public long getUsed();
            public long getCommitted();
            public long getMax();
            public java.lang.String toString();
        }

        public final class java.lang.management.MemoryType {
            public static final java.lang.management.MemoryType HEAP;
            public static final java.lang.management.MemoryType NON_HEAP;
            private final java.lang.String description;
            private static final java.lang.management.MemoryType[] $VALUES;
            public static java.lang.management.MemoryType[] values();
            public static java.lang.management.MemoryType valueOf(java.lang.String);
            private java.lang.management.MemoryType(java.lang.String);
            public java.lang.String toString();
            private static java.lang.management.MemoryType[] $values();
            static {};
          }
    }
}

pub use auto::java::*;

// XX this isn't a real class in the JVM, since each array type (e.g. Foo[] and int[]) is just a subclass of Object.
// Should it go somewhere outside of the JDK core classes?
pub use crate::array::JavaArray as Array;
pub use crate::array::JavaArrayExt as ArrayExt;
