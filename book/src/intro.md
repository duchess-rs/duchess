# Duchess: silky smooth Java integration

Duchess is a crate that makes it easy to use Java code.

<img src="duchess.svg"></img>

## TL;DR

```rust
// Step 1: Reflect your java code into Rust
duchess::java_package! {
    package com.myjava;
    class MyClass { * }
}

// Step 2: Start the JVM
duchess::with_jvm(|jvm| {
    // Step 3: Create objects and call methods.
    // Constructors and methods can be chained;
    // use `execute` to run the whole chain on the JVM.
    use com::myjava::{MyClass, MyClassExt};
    MyClass::new()
        .some_builder_method(44)
        .some_other_method("Hello, world")
        .execute_with(jvm);
})
```

## Tutorials

To learn more, check out one of our tutorials

* [Calling Java from Rust](./call_java_from_rust.md)
* [Implementing native methods](./implementing_native_methods.md)

or check out the [reference](./reference.md).

