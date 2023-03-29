# duchess
Experiments with Java-Rust interop

## Instructions

You need the `javap` tool on your path.

On Ubuntu, I installed `java-20-amazon-corretto-jdk/stable`, 
but `openjdk-17-jre/stable-security` might also work. --nikomatsakis

## How to use

*This is a README from the future, in that it describes the intended plan for the crate.*

### What it does

Duchess makes it easy to call Java APIs from your Rust code. It may eventually help with bidirectional support, but right now that is not supported.

### How duchess works

Let's suppose that you have a java class `me.ferris.Logger`:

```java
class Logger {
    public static Logger globalLogger();

    // Simple, convenient log method
    public void log(String data);

    public void logFull(LogMessage message);
}

class LogMessage {
    public LogMessage(String message);

    LogMessage level(int level);
}
```

which you can use in your java code to issue simple logs

```java
Logger.globalLogger().log("Hello, world");
```

or to issue more complex ones

```java
LogMessage m = new LogMessage("Hello, world").level(22);
Logger.globalLogger().log(m);
```

But now you would like to write some Rust code that invokes this same logging service. What do you do?

### TL;DR

For the impatient among you, here is the kind of code you're going to be able to write when we're done. First you declare the java classes you want to work with:

```rust
duchess::duchess! {
    me.ferris.Logger,
    me.ferris.LogMessage,
}
```

and then instead of this java code

```java
Logger.globalLogger().log("Hello, world");
```

you can write Rust like this

```rust
use me::ferris::Logger;
Logger::globalLogger(jni).log(jni, "Hello, world");
```

and instead of this Java code

```java
LogMessage m = new LogMessage("Hello, world").level(22);
Logger.globalLogger().log(m);
```

you can write Rust like this

```rust
use me::ferris::{Logger, LogMessage};
LogMessage::new(jni, "Hello, world").level(jni, 22);
Logger::globalLogger(jni).log(jni, &m);
```

Huzzah!

### What code does the macro generate?

Let's walk through this in more detail. To start, use the `duchess!` macro to create a Rust view onto the java code. The `duchess!` macro supports various bells and whistles, but in its most simple form, you just declare a module and list some java classes inside of it.

```rust
duchess::duchess! {
    me.ferris.Logger // Always list the java classes by their full dotted name!
}
```

The procedural macro will create a module named `jlog` and, for each class that you name, a struct and an impl containing all of its methods, but mirrored into Rust. The structs are named after the full Java name (including the package), but there are type aliases for more convenient access:

```rust
mod me {
    pub mod ferris {
        pub struct Logger<'jni> { ... }    
    
        impl<'jni> Logger<'jni> {
            pub fn globalLogger() -> Logger<'jni> {
                ...
            }

            pub fn log(&self, s: impl IntoJavaString) {
                ...
            }

            pub fn log_full(&self, s: &impl AsRef<LogMessage<'jni>>) {
                ...
            }
        }
    }

    ... // more to come
}
```

Where possible, we translate the Java argument types into Rust-like forms. References to Java strings, for example, compile to `impl IntoJavaString`, a trait which is implemented for `&str` and `String` (but also for a reflected Java string).

```rust
pub fn log(&self, s: impl IntoJavaString) {
    ...
}
```

In some cases, methods will reference Java classes besides the one that appeared in the proc macro, like `me.ferris.LogMessage`:

```rust
pub fn log_full(&self, s: &impl AsRef<LogMessage<'jni>>)
```

These extra types get translated to structs as well. But these structs don't have impl blocks or methods. They're just opaque values you can pass around:

```rust
mod me {
    pub mod ferris {
        // From before:
        pub struct Logger<'jni> { ... }
        impl<'jni> Logger<'jni> { ... }

        // Also generated:
        pub struct LogMessage<'jni> { ... }
    }

    ...
}
```

In fact, we'll also generate entries for other Java classes, like

```rust
mod me {...}
mod java {
    pub mod lang {
        pub struct Object<'jni> { ... }
        pub struct String<'jni> { ... }
    }
}
```

Finally, we generate various `AsRef` (and `From`) impls that allow for upcasting between Java types:

```rust
mod me { /* as before */  }
mod java { /* as before */ }

impl<'jni> AsRef<java::lang::Object<'jni>> for me::ferris::Logger<'jni> { ... }
impl<'jni> AsRef<java::lang::Object<'jni>> for me::ferris::LogMessage<'jni> { ... }

impl<'jni> From<me::ferris::Logger<'jni>> for java::lang::Object<'jni>> { ... }
impl<'jni> From<me::ferris::LogMessage<'jni>> for java::lang::Object<'jni>> { ... }
```

### Implementation details

Our structs are actually [thin wrappers around jni::JObject][JO]:

[JO]: https://docs.rs/jni/0.21.1/jni/objects/struct.JObject.html

```rust
#[repr(transparent)]
pub struct Logger<'jni> {
    object: jni::JObject<'jni>
}
```

In some cases, the JNI crate only supplies `&JObject` values. These can be safely transmuted into (e.g.) `&Logger` values because of the `repr(transprent)` layout (c.f. [rust reference][rrtr], though I'd like to check on the details here --nikomatsakis). We provide a helper function `cast` for this purpose.

[rrtr]: https://doc.rust-lang.org/reference/type-layout.html?highlight=transparent#the-transparent-representation



