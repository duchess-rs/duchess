// Make current crate available as `duchess` for use by the generated code.
// NB. in documentation mode, the current crate is already available as duchess.
#[cfg(not(doc))]
use crate as duchess;

use crate::java;

duchess_macro::duchess_javap! {
r#"
        Compiled from "Object.java"
        public class java.lang.Object {
            public java.lang.Object();
                descriptor: ()V

            public final native java.lang.Class getClass();
                descriptor: ()Ljava/lang/Class;

            public native int hashCode();
                descriptor: ()I

            public boolean equals(java.lang.Object);
                descriptor: (Ljava/lang/Object;)Z

            public java.lang.String toString();
                descriptor: ()Ljava/lang/String;

            public final native void notify();
                descriptor: ()V

            public final native void notifyAll();
                descriptor: ()V

            public static native void notifyStatic(String...);
                descriptor: ([Ljava/lang/String;)V
        }
    "#
}

duchess_macro::duchess_javap! {
r#"
        Compiled from "Throwable.java"
        public class java.lang.Throwable {
        public java.lang.Throwable();
            descriptor: ()V

        public java.lang.String getMessage();
            descriptor: ()Ljava/lang/String;

        public java.lang.String getLocalizedMessage();
            descriptor: ()Ljava/lang/String;

        public synchronized java.lang.Throwable getCause();
            descriptor: ()Ljava/lang/Throwable;

        public synchronized java.lang.Throwable initCause(java.lang.Throwable);
            descriptor: (Ljava/lang/Throwable;)Ljava/lang/Throwable;

        public java.lang.String toString();
            descriptor: ()Ljava/lang/String;

        public void printStackTrace();
            descriptor: ()V

        public synchronized java.lang.Throwable fillInStackTrace();
            descriptor: ()Ljava/lang/Throwable;

        public java.lang.StackTraceElement[] getStackTrace();
            descriptor: ()[Ljava/lang/StackTraceElement;

        public void setStackTrace(java.lang.StackTraceElement[]);
            descriptor: ([Ljava/lang/StackTraceElement;)V

        public final synchronized void addSuppressed(java.lang.Throwable);
            descriptor: (Ljava/lang/Throwable;)V

        public final synchronized java.lang.Throwable[] getSuppressed();
            descriptor: ()[Ljava/lang/Throwable;
        }
    "#
}

duchess_macro::duchess_javap! {
r#"
        Compiled from "StackTraceElement.java"
        public final class java.lang.StackTraceElement {
        public java.lang.StackTraceElement(java.lang.String, java.lang.String, java.lang.String, int);
            descriptor: (Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;I)V

        public java.lang.String getFileName();
            descriptor: ()Ljava/lang/String;

        public int getLineNumber();
            descriptor: ()I

        public java.lang.String getModuleName();
            descriptor: ()Ljava/lang/String;

        public java.lang.String getModuleVersion();
            descriptor: ()Ljava/lang/String;

        public java.lang.String getClassLoaderName();
            descriptor: ()Ljava/lang/String;

        public java.lang.String getClassName();
            descriptor: ()Ljava/lang/String;

        public java.lang.String getMethodName();
            descriptor: ()Ljava/lang/String;

        public boolean isNativeMethod();
            descriptor: ()Z

        public java.lang.String toString();
            descriptor: ()Ljava/lang/String;

        public boolean equals(java.lang.Object);
            descriptor: (Ljava/lang/Object;)Z

        public int hashCode();
            descriptor: ()I
        }
    "#
}

duchess_macro::duchess_javap! {
r#"
        Compiled from "Class.java"
        public final class java.lang.Class {
        public java.lang.String toString();
            descriptor: ()Ljava/lang/String;

        public java.lang.String toGenericString();
            descriptor: ()Ljava/lang/String;

        public native boolean isInstance(java.lang.Object);
            descriptor: (Ljava/lang/Object;)Z

        public native boolean isAssignableFrom(java.lang.Class);
            descriptor: (Ljava/lang/Class;)Z

        public native boolean isInterface();
            descriptor: ()Z

        public native boolean isArray();
            descriptor: ()Z

        public native boolean isPrimitive();
            descriptor: ()Z

        public boolean isAnnotation();
            descriptor: ()Z

        public boolean isSynthetic();
            descriptor: ()Z

        public java.lang.String getName();
            descriptor: ()Ljava/lang/String;

        public native java.lang.Class getSuperclass();
            descriptor: ()Ljava/lang/Class;

        public java.lang.String getPackageName();
            descriptor: ()Ljava/lang/String;

        public java.lang.Class[] getInterfaces();
            descriptor: ()[Ljava/lang/Class;

        public java.lang.Class getComponentType();
            descriptor: ()Ljava/lang/Class;

        public java.lang.Class arrayType();
            descriptor: ()Ljava/lang/Class;
        }

    "#
}

duchess_macro::duchess_javap! {
r#"
        Compiled from "String.java"
        public final class java.lang.String {
        public java.lang.String(byte[]);
            descriptor: ([B)V

        public int length();
            descriptor: ()I

        public boolean isEmpty();
            descriptor: ()Z
        }
    "#
}

duchess_macro::duchess_javap! {
r#"
    Compiled from "Record.java"
    public abstract class java.lang.Record {
    public abstract boolean equals(java.lang.Object);
        descriptor: (Ljava/lang/Object;)Z

    public abstract int hashCode();
        descriptor: ()I

    public abstract java.lang.String toString();
        descriptor: ()Ljava/lang/String;
    }
    "#
}
