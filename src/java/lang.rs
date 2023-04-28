// Make current crate available as `duchess` for use by the generated code.
// NB. in documentation mode, the current crate is already available as duchess.
#[cfg(not(doc))]
use crate as duchess;

// Declare the class "object" in isolation.
// Eventually we'd like to move all the declarations
// into a `java_package` call, but we still have
// to fix some bugs, so for now, just do that call
// inside of a `mod object` and re-export what we want.
mod object {
    use super::*;
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
    }
}

pub use object::java::lang::*;
