# The `java_function` macro

The `java_function` macro is used to implement native functions. Make sure to read about how you [link these native functions into the JVM](./linking_native_functions.md).

## Specifying which function you are defining

The `#[java_function(X)]` takes an argument `X` that specifies which Java function is being defined.
This argument `X` can have the following forms:

* `java.class.Name::method`, identifying a `native` method `method` defined in the class `java.class.Name`. There must be exactly one native method with the given name.
* a partial class definition like `class java.class.Name { native void method(int i); }` which identifies the method name along with its complete signature. This class definition must contain exactly one method as its member, and the types must match what is declared in the Java class.

## Expected function arguments and their type

`#[java_function]` requires the decorated function to have the following arguments:

* (Optional) JVM environment `env: &mut JvmEnv<'_>` -- must be named `env`!
* If not static, a `this` parameter -- can have any name, but we recommend `this`
* One parameter per Java argument -- can have any name, but we recommend matching the names used in Java

For the `this` and other Java arguments, their type can be:

* `i32`, `i16`, etc for Java scalars
* `&J` where `J` is the Java type
* `R` where `R` is some Rust type that corresponds to the Java type

## Expected return type

If the underlying Java function returns a scalar value, your Rust function must return that same scalar value.

Otherwise, if the underlying Java function returns an object of type `J`, the value returned from your function will be converted to `J` by invoking the [`to_java`](./to_java.md) method. This means your functon can return:

* a reference to a Java object of type `J` (e.g., `Global<J>`) 
* a Rust value that can be converted to `J` via `to_java::<J>`

## Linking your native function into the JVM

This is covered under a [dedicated page](./linking_native_functions.md).
