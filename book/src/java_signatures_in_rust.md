# Translating Java method signatures to Rust

The [`java_package`](./java_package.md) macro translates Java methods into Rust methods.
The method argument types are translated as follows:

| Java argument type       | Rust argument type |
| ---------                | --------- |
| `byte`                   | `impl duchess::IntoScalar<i8>` |
| `short`                  | `impl duchess::IntoScalar<i16>` |
| `int`                    | `impl duchess::IntoScalar<i32>` |
| `long`                   | `impl duchess::IntoScalar<i64>` |
| Java object type J       | `impl duchess::IntoJava<J>` |
| e.g., `java.lang.String` | `impl duchess::IntoJava<java::lang::String>` |

The Rust version of the Java method will return one of the following traits.
These are not the actual Rust value, but rather the [JVM operation](./jvm_operations.md)
that will yield the value when executed:

| Java return type         | Rust return type |
| ---                      | --- |
| `void`                   | `impl duchess::VoidMethod` |
| `byte`                   | `impl duchess::ScalarMethod<i8>` |
| ...                      | `impl duchess::ScalarMethod<...>` |
| `long`                   | `impl duchess::ScalarMethod<i64>` |
| Java object type J       | `impl duchess::JavaMethod<J>` |
| e.g., `java.lang.String` | `impl duchess::JavaMethod<java::lang::String>` |
