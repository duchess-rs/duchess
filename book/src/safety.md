# Memory safety requirements

Duchess provides a **safe abstraction** atop the [Java Native Interface (JNI)][jni].
This means that, as long as you are using Duchess to interact with the JVM,
you cannot cause memory unsafety.
However, there are edge cases that can "void" this guarantee and which Duchess cannot control.

[jni]: https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/jniTOC.html

## Memory safety requirements

Duchess will guarantee memory safety within your crate, but there are two conditions that it cannot by itself guarantee:
* Duchess does not "sandbox" or "make safe" the Java code it executes. You MUST ensure that Java code being invoked is safe and trusted. You MUST NOT invoke untrusted Java code with Duchess.
* **You must SHOULD with the same Java class files that you will use when you deploy:**
    * We believe that no loss of memory-safety is possible if incorrect `.class` files are loaded, however, if interfaces change you will experience failures at runtime.
* **You must be careful when mixing Duchess with other Rust JNI libraries:** (e.g., the [jni crate](https://crates.io/crates/jni) or [robusta_jni](https://crates.io/crates/robusta_jni))
    * For the most part, interop between Duchess and other JNI crates should be no problem. But there are some particular things that can cause issues:
        * The JVM cannot be safely started from multiple threads at once.
          Duchess uses a lock to avoid contending with itself but we cannot protect from other libraries starting the JVM in parallel with us.
          It is generally best to start the JVM yourself (via any means) in the `main` function or some other central place so that you are guaranteed it happens once and exactly once.
          Duchess should work just fine if the JVM has been started by another crate.
