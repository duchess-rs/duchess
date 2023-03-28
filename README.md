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

### Step 1: Declare your classes

You can use duchess to call them via JNI! To start, use the `duchess!` macro to create a Rust view onto the java code. The `duchess!` macro supports various bells and whistles, but in its most simple form, you just declare a module and list some java classes inside of it.

```rust
duchess::duchess! {
    mod jlog {
        me.ferris.Logger // Always list the java classes by their full dotted name!
    }
}
```

When you compile the Rust code, this procedural macro will create a module named Java with one struct per Java class (ignore the `'jni` lifetime for now, we'll get to that):

```rust
// generated code looks like...
mod java {
    pub struct me_ferris_Logger<'jni> { ... }
    ...
}
```

In fact, it will contain one struct for every Java class/interface that is reachable through the API, including those that you didn't mention:

```rust
mod java {
    pub struct me_ferris_Logger<'jni> { ... }
    pub struct me_ferris_LogMessage<'jni> { ... }
    pub struct java_lang_Object<'jni> { ... } // java.lang.Object
    ...
}
```

For those Java classes that you did name, it will also define methods:

```rust
mod java {
    pub struct me_ferris_Logger<'jni> { ... }
    pub struct me_ferris_LogMessage<'jni> { ... }
    pub struct java_lang_Object<'jni> { ... } // java.lang.Object
    
    impl<'jni> me_ferris_Logger<'jni> {
        pub fn globalLogger() -> me_ferris_Logger<'jni> {
            ...
        }

        pub fn log(&self, s: impl AsRef<str>) {
            ...
        }

        pub fn log_full(&self, s: &me_ferris_LogMessage<'jni>) {
            ...
        }
    }
}
```