# Setup instructions

## TL;DR

You need to...

* [Install the JDK](#jdk-and-java_home)
* Install the `cargo-duchess` CLI tool with `cargo install cargo-duchess`
* Run `cargo duchess init` in your package, which will add duches to your `build.rs` file and your `Cargo.toml`

## Prequisites

### JDK and JAVA_HOME

You'll need to have a modern JDK installed. We recommend JDK17 or higher. Any JDK distribution will work. Here are some recommended options:

* Ubuntu: Install one of the following packages...
    * `java-20-amazon-corretto-jdk/stable`
    * `openjdk-17-jre/stable-security`
    * `openjdk-17-jdk-headless` 
* Other:
    * Download [Amazon Coretto](https://aws.amazon.com/corretto/?filtered-posts.sort-by=item.additionalFields.createdDate&filtered-posts.sort-order=desc)
    * Download a [pre-built openjdk package](https://openjdk.org/install/) suitable for your operating system

**You'll need the `javap` tool from the JDK to build with Duchess.**  You'll want to configure the `JAVA_HOME` environment variable to point to your JDK installation. Duchess will use it to locate `javap`. Otherwise, Duchess will search for it on your `PATH`.  You can configure the environment variables used at build time via Cargo by creating a `.cargo/config.toml` file (see [this example from duchess itself](https://github.com/duchess-rs/duchess/blob/main/.cargo/config.toml)).

Duchess relies on `javap` to reflect Java type information at build time. It will *not* be invoked at runtime.

## Basic setup

To use Duchess your project requires a `build.rs` as well as a proc-macro crate. The `build.rs` does the heavy lifting, invoking javap and doing other reflection. The proc-macro crates then do final processing to generate the code.

You can 

## Other details

## Configuring the CLASSPATH

You will likely want to configure the `CLASSPATH` for your Rust project as well. Like with `JAVA_HOME`, you can do that via Cargo by creating a `.cargo/config.toml` file.

If your Rust project uses external JAR files, you may want to configure it to download them as part of the build. The [viper test crate](https://github.com/duchess-rs/duchess/tree/main/test-crates/viper) gives an example of how to do that. It uses a [build.rs](https://github.com/duchess-rs/duchess/blob/main/test-crates/viper/build.rs) file.

## Libjvm and linking

By default, the `dylibjvm` feature is enabled and Duchess will dynamically load and link libjvm at runtime. Like with `javap`, it will first search for libjvm in `JAVA_HOME` if set. Otherwise it will look for `java` on your `PATH` to locate the JRE installation. Non-standard installations can also be configured using `JvmBuilder`.

Without `dylibjvm`, libjvm must be statically linked.

## JNI Versions

By default, we attempt to load JNI 1.6 when compiling for Android, and JNI 1.8 in all other cases. The JNI version can be selected by using the feature `jni_` and the JNI version concatenated, for any supported JNI version, with underscores replacing periods.
Duchess currently only supports JNI versions 1.6 and 1.8, and only supports 1.6 on Android (the compile will fail if JNI > 1.6 is attempted on Android). Duchess sets the version to the newest version specified by features if features are specified.
If you want Duchess to support a newer JNI API version or locking behavior, cut an issue with your use case, and it may be added to Duchess's next release.
