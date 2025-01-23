# The `java_function` macro

The `java_function` macro is used to implement native functions. Make sure to read about how you [link these native functions into the JVM](./linking_native_functions.md). 

`java_function` is a low-level primitive. For a more ergonomic, full-featured way to wrap Rust library and call it from Java,
check out [the `gluegun` crate](gluegun).

[gluegun]: https://gluegun-rs.github.io/gluegun/

## Examples

Just want to see the code? Read some of the tests:

* https://github.com/duchess-rs/duchess/tree/main/test-crates/duchess-java-tests/tests/java-to-rust/rust-libraries

[the `greeting` example](https://github.com/duchess-rs/duchess/blob/main/test-crates/duchess-java-tests/tests/ui/examples/greeting.rs) to see the setup in action.

## Specifying which function you are defining

The `#[java_function(X)]` takes an argument `X` that specifies which Java function is being defined.
This argument `X` can have the following forms:

* `java.class.Name::method`, identifying a `native` method `method` defined in the class `java.class.Name`. There must be exactly one native method with the given name.
* a partial class definition like `class java.class.Name { native void method(int i); }` which identifies the method name along with its complete signature. This class definition must contain exactly one method as its member, and the types must match what is declared in the Java class.

## Expected function arguments and their type

`#[java_function]` requires the decorated function to have the following arguments:

* If not static, a `this` parameter -- can have any name, but we recommend `this`, whose type is the Duchess version of the Java type
* One parameter per Java argument -- can have any name, but we recommend matching the names used in Java

If present, the `this` argument should have the type `&foo::Bar` where `foo::Bar` is the Duchess type of the Java class. i.e., if this is a native method defined on the class `java.lang.String`, you would have `this: &java::lang::String`.

The other arguments must match the types declared in Java:

* For Java scalars, use `i32`, `i16`, etc.
* For reference types, use `Option<&J>`, where `J` is the Java type (e.g., `Option<&java::lang::String>`).
    * Note that `Option` is required as the Java code can always provide `null`. You can use the [`assert_not_null`][] method on `JvmOp`.

[`assert_not_null`]: https://duchess-rs.github.io/duchess/rustdoc/doc/duchess/prelude/trait.JvmOp.html#method.assert_not_null

## Expected return type

If the underlying Java function returns a scalar value, your Rust function must return that same scalar value.

Otherwise, if the underlying Java function returns an object of type `J`, the value returned from your function will be converted to `J` by invoking the [`to_java`](./to_java.md) method. This means your functon can return:

* a reference to a Java object of type `J` (e.g., `&J` or `Java<J>`);
* a Rust value that can be converted to `J` via `to_java::<J>`.

## Linking your native function into the JVM

This is covered under a [dedicated page](./linking_native_functions.md).
