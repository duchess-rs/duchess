# Duchess: silky smooth Java integration

Duchess is a crate that makes it easy to use Java code.

## Basic usage

### TL;DR

* Use the [`duchess::java_package!`](./java_package.md) macro to reflect the Java package in your Rust code.
* Start a JVM with [`duchess::Jvm::with(|jvm| ...)`](./jvm.md); inside the closure, you'll be able to reference `jvm` to run things on the JVM.
* Invoke constructors like `MyClass::new().execute(jvm)` and methods like `some_obj.some_method(...).execute(jvm)`.
    * Note that you call `execute` to make them actually run, and you can (and should) chain them together before doing so.

### The Java class we would like to use from Rust

Imagine we have a Java class `Factory` that we would like to use from Rust, defined like so:

```java
package com.widgard;

public class Factory {
    public Factory() { /* ... */ }
    public Widget produceWidget() { /* ... */ }
    public void consumeWidget(widget w) { /* ... */ }
}

public class Widget { /* ... */ }
```

### Using a package from Rust

Using duchess, we can declare a Rust version of this class with the `java_package!` macro:

```rust
duchess::java_package! {
    // First, identify the package you are mirroring,
    // and the visibility level that you want.
    pub package com.widgard;

    // Next, identify classes whose methods you would like to call. 
    // The `*` indicates "reflect all methods".
    // You can also name methods individually (see below).
    class Factory { * }
}
```

### Generated code

This module will expand to a module hierarchy matching the Java package name:

```rust
pub mod com {
    pub mod widgard {
        // One struct per Java class:
        pub struct Factory { /* ... */ }
        
        // The inherent impl defines the constructor
        // and any static methods:
        impl Factory { /* ... */ }

        // The extension trait defines the methods
        // on the struct, like `produceWidget`
        // and `consumeWidget`.
        pub trait FactoryExt { /* ... */ }
        
        // There is also a struct for other classes
        // in the same package if they appear in
        // the signature of the reflected methods. 
        //
        // In this case, `Factory#produceWidget`
        // returns a `Widget`, so we get this struct here.
        //
        // Since we did not tell duchess to reflect any
        // methods, there is no `WidgetExt` trait,
        // nor an inherent impl.
        pub struct Widget  { /* ... */ }
    }
}
```

**NB:** The `java_package` macro relies on the `javap` tool to reflect Java signatures. You will need to have the [Java Development Kit (JDK)](https://openjdk.org/) installed for it to to work. You will also need to help us to find the java code by setting `CLASSPATH` appropriately. Note that you can [configure the environment in your Cargo.toml](https://doc.rust-lang.org/cargo/reference/config.html) if desired.

### Using the generated code

Once you've created the Java package, you can create java objects and invoke their methods. This should mostly just work as you would expect, with one twist. Invoking a Java method doesn't immediately cause it to execute. Instead, like an iterator or an async function, it returns a `JvmOp`, which is like a suspended JVM operation that is *ready* to execute. To actually cause the method to execute, you call `execute`.

```rust
// We need to use `FactoryExt` to call methods on factory:
use com::widgard::{Factory, FactoryExt};

duchess::Jvm::with(|jvm| {
    // Constructors are `Type::new`...
    let f = Factory::new().execute(jvm);

    // ...method names are converted to snake-case...    
    let w = f.produce_widget().execute(jvm);

    // ...references to Java objects are passed with `&`.
    f.consume_widget(&w).execute(jvm);
})
```

### Launching the JVM

Note that to call methods on the JVM, we first had to start it. You do that via `duchess::Jvm::with`. This method will launch a JVM if it hasn't already started and attach it to the current thread. OpenJDK only supports one JVM per process, so the JVM is global.

### Combining steps into one

Because jvm-ops are lazy, you can also chain them together:

```rust
use com::widgard::{Factory, FactoryExt};

duchess::Jvm::with(|jvm| {
    let f = Factory::new().execute(jvm);

    // Consume and produce the widget in one step:
    f.consume_widget(f.produce_widget()).execute(jvm);
})
```

In fact, using the `inspect` combinator, we can go further:

```rust
use com::widgard::{Factory, FactoryExt};

duchess::Jvm::with(|jvm| {
    Factory::new()
        .inspect(|f| f.consume_widget(f.produce_widget()))
        .execute(jvm);
})
```

At the moment, combining steps is equivalent to invoking them individually. However, the plan is for it to become more efficient by reducing the number of times we invoke JNI methods. 


