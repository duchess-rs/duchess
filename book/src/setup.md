# Setup instructions

## JDK and JAVA_HOME

You'll need to have a modern JDK installed. We recommend JDK17 or higher. Any JDK distribution will work. Here are some recommended options:

* Ubuntu: Install one of the following packages...
    * `java-20-amazon-corretto-jdk/stable`
    * `openjdk-17-jre/stable-security`
    * `openjdk-17-jdk-headless` 
* Other:
    * Download [Amazon Coretto](https://aws.amazon.com/corretto/?filtered-posts.sort-by=item.additionalFields.createdDate&filtered-posts.sort-order=desc)
    * Download a [pre-built openjdk package](https://openjdk.org/install/) suitable for your operating system

**You'll need the `javap` tool from the JDK to build with Duchess.**  If the `JAVA_HOME` environment variable is set, Duchess will use it to locate `javap`. Otherwise, it will search for it on your `PATH`. Duchess relies on `javap` to reflect Java type information at build time. It will *not* be invoked at runtime.

## Configuring the CLASSPATH

You will likely want to configure the CLASSPATH for your Rust project as well. You can do that via Cargo by creating a `.cargo/config.toml` file (see [this example from duchess itself](https://github.com/duchess-rs/duchess/blob/main/.cargo/config.toml)).

If your Rust project uses external JAR files, you may want to configure it to download them as part of the build. The [viper test crate](https://github.com/duchess-rs/duchess/tree/main/test-crates/viper) gives an example of how to do that. It uses a [build.rs](https://github.com/duchess-rs/duchess/blob/main/test-crates/viper/build.rs) file.

## Libjvm and linking

By default, the `dylibjvm` feature is enabled and Duchess will dynamically load and link libjvm at runtime. Like with `javap`, it will first search for libjvm in `JAVA_HOME` if set. Otherwise it will look for `java` on your `PATH` to locate the JRE installation. Non-standard installations can also be configured using `JvmBuilder`.

Without `dylibjvm`, libjvm must be statically linked.