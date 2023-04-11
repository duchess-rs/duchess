use crate as duchess;
use crate::java;

duchess_macro::duchess_javap! {
r#"
        Compiled from "Object.java"
        public class java.lang.Object {
            public java.lang.Object();
                descriptor: ()V

            public final native java.lang.Class<?> getClass();
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
        }
    "#
}