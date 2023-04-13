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
