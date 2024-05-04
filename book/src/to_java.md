# The `ToJava` trait

The `ToJava` trait is part of the Duchess prelude. 
It defines an `&self` method `to_java` that can be used to convert Rust values into Java objects;
if those Rust types are references to a Java object, then the result is just an identity operation.
The result of `to_java` is not the Java itself but rather a [`JvmOp`] that produces the Java object.

[`JvmOp`]: ./jvm_op.md

In some cases, the same Rust type can be converted into multiple Java types.
For example, a Rust Vec can be converted into a Java `ArrayList` but also a Java `List` or `Vector`.
The `to_java` method takes a type parameter for these cases that can be specified with turbofish,
e.g., `vec.to_java::<java::util::List<_>>()`.

## Examples

### `String`

The Rust `String` type converts to the Java string type.
One could compute the Java `hashCode` for a string as follows:

```rust,ignore
use duchess::prelude::*;
use duchess::java;

let data = format!("Hello, Duchess!");
let hash_code: i32 =
    data.to_java::<java::lang::String>()  // Returns a `JvmOp` producing a `java::lang::String`
        .hash_code()                      // Returns a `JvmOp` invoking `hashCode` on this string
        .execute()?;                       // Execute the jvmop
```

### `Global<java::lang::String>`

Converting a Rust reference to a Java object, such as a `Global` reference, is an identity operation.

```rust,ignore
use duchess::prelude::*;
use duchess::java;

// Produce a Global reference from a Rust string
let data: Global<java::lang::String> =
    format!("Hello, Duchess!").execute()?;

// Invoke `to_java` on the `Global` reference
let hashCode: i32 =
    data.to_java::<java::lang::String>()   // Returns a `JvmOp` producing a `java::lang::String`
        .hashCode()  // Returns a `JvmOp` invoking `hashCode` on this string
        .execute()?;  // Execute the jvmop
```

## Deriving `ToJava` for your own types

Duchess provides a derive for `ToJava` that you can apply to structs or enums.
Details can be found in the [dedicated book section covering derive](./derive.md).