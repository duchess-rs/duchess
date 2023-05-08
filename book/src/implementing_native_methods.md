# Tutorial: implementing native methods

Duchess also supports implementing Java native methods, making it easy to call Rust code from Java.
Given a Java class

```java
package me.ferris;

public class ClassWithNativeMethod {
    int data() { return 22; }
    native String compute(Object o);
}
```

you can provide an implementation for `compute` like so:

```rust
// First, reflect the class, as described in the "calling Java from Rust" tutorial:
duchess::java_package! {
    package me.ferris;
    class ClassWithNativeMethod { * }
}

use duchess::{java, IntoJava};
use me::ferris::ClassWithNativeMethod;

// Next, provide a decorated Rust function.
// The arguments are translated from Java, including the `this`.
// The return type is either a scalar or `impl IntoJava<J>`
// where `J` is the Java type.
#[duchess::native(me.ferris.ClassWithNativeMethod::compute)]
fn compute(
    jvm: &mut jvm<'_>,
    this: &ClassWithNativeMethod,
    object: &java::lang::Object,
) -> impl IntoJava<java::lang::String> {
    // in here you can call back to JVM too
    let data = this.data().execute_with(jvm);
    format!("Hello from Rust {data}")
}
```



