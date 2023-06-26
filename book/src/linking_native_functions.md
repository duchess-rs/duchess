# Linking native functions into the JVM

Using the [`#[java_function]`](./java_function.md) decorator you can write Rust implementations for Java native functions. To get the JVM to invoke these methods, it has to know how to find them. The way you do this depends on whether you have the "top-level" program is written in Rust or in Java.

## Rust program that creates a JVM

If your Rust program is launching the JVM, then you can configure that JVM to link to your native method definitions through methods on the JVM builder.

```rust,ignore
use duchess::prelude::*; // 👈 You'll need this.

#[java_function(...)]
fn foo(...) { }

fn main() -> duchess::GlobalResult<()> {
    Jvm::builder()
        .link(foo::java_fn()) // 👈 Note the `::java_fn()`.
        .try_launch()?;
}
```

**How it works.** The call `foo::java_fn()` returns a `duchess::JavaFunction` struct. The `java_fn` method is defined in the duchess `JavaFn` trait; that trait is implemented on a struct type `foo` that is created by the `#[java_function]` decorator. This trait is in the duchess prelude, which is why you need to `use duchess::prelude::*`.

### Java function suites

Invoking the link method for every java functon you wish to implement is tedious and error-prone. If you have java functions spread across crates and modules, it also presents a maintenance hazard, since each time you add a new `#[java_function]` you would also have to remember to add it to the Jvm builder invocation, which is likely located in some other part of the code.

To avoid this, you can create **suites** of java functions. The idea is that the `link` method accepts both individual `JavaFunction` structs but also `Vec<JavaFunction>` suites. You can then write a function in your module that returns a `Vec<JavaFunction>` with all the java functions defined locally:

```rust,ignore
use duchess::prelude::*;

#[java_function(...)]
fn foo(...) { }

#[java_function(...)]
fn bar(...) { }

fn java_functions() -> Vec<JavaFunction> {
    vec![
        foo::java_fn(),
        bar::java_fn(),
    ]
}
```

You can also compose suites from other crates or modules:

```rust,ignore
fn java_functions() -> Vec<duchess::JavaFunction> {
    crate_a::java_functions()
        .into_iter()
        .chain(crate_b::java_functions())
        .collect()
}
```

And finally you can invoke `link()` to link them all at once:

```rust,ignore
fn main() -> duchess::GlobalResult<()> {
    Jvm::builder()
        .link(java_functions())
        .try_launch()?;
}
```

## JVM that calls into Rust

If the JVM is the "master process", then you have to use a different method to link into Rust. First, you have to compile your Rust binary as a cdylib by configuring `Cargo.toml` with a new `[lib]` section:

```toml
[lib]
crate_type = ["cdylib"]
```

Then in your Java code you have to invoke `System.loadLibrary`. Typically you do this in a `static` section on the class with the `native` method:

```java
class HelloWorld {
    // This declares that the static `hello` method will be provided
    // a native library.
    private static native String hello(String input);

    static {
        // This actually loads the shared object that we'll be creating.
        // The actual location of the .so or .dll may differ based on your
        // platform.
        System.loadLibrary("mylib");
    }
}
```

Finally, you need to run `cargo build` and put the dylib that is produced into the right place. The details different by platform. On Linux, you can `export LD_LIBRARY_PATH=/path/to/mylib/target/debug` to link the dylib directly from the Cargo build directory.

*These instructions were based on the excellent [docs from the jni crate](https://docs.rs/jni/latest/jni/); you can read more there.*