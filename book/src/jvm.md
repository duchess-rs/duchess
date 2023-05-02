# The Jvm type

The `Jvm` type represents a running Java Virtual Machine (JVM). It is mostly used to `execute` [JVM operations], but it also has some methods for interacting with the JVM that you may find useful. The way you get access to a `Jvm` instance depends on the language of the primary application:

* If your main process is **Rust**, then use `Jvm::with` to start the global JVM instance.
* If your main process is **Java**, then when your Rust code is invoked via JNI, you will be given a `Jvm` instance.

[JVM operations]: ./jvm_operations.md

## Starting multiple JVMs

As long as a thread has access to a `Jvm`, either by invoking `Jvm::with` or by getting called via JNI, you cannot get access to another one. Invoking `Jvm::with` on a thread that already has access to a Jvm is an error. This is required to ensure safety, because it allows us to be sure that mutably borrowing a `Jvm` instance blocks the thread from performing other `Jvm` operations until that borrow is complete. Sequential invocations of `Jvm::with` are allowed and will all be attached to that same underlying JVM instance.

Multiple threads can invoke `Jvm::with`, but only one underlying JVM can ever be active at a time. If multiple threads invoke `Jvm::with`, one of them will succeed in starting the JVM, and the others will be attached to that same underlying JVM instance as additional active threads.

## Starting the JVM: setting options

When you start the JVM from your Rust code, you can set various options by using the jvm builder:

```rust
Jvm::builder()
    .add_classpath("foo")
    .add_classpath("bar")
    .memory()
    .custom("-X foobar")
    .launch(|jvm| {

    })
```

Unlike the `with` command, the `launch` command panics if the JVM has already been started by some other thread.

