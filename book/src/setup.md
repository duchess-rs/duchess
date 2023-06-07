# Setup instructions

## JDK

**You need the `javap` tool on your path.** We recommend JDK17 or higher.

Any JDK distribution will work. Here are some recommended options:

* Ubuntu: Install one of the following packages...
    * `java-20-amazon-corretto-jdk/stable`
    * `openjdk-17-jre/stable-security`
    * `openjdk-17-jdk-headless` 
* Other:
    * Download [Amazon Coretto](https://aws.amazon.com/corretto/?filtered-posts.sort-by=item.additionalFields.createdDate&filtered-posts.sort-order=desc)
    * Download a [pre-built openjdk package](https://openjdk.org/install/) suitable for your operating system

## Configuring the CLASSPATH

You will likely want to configure the CLASSPATH for your Rust project as well. You can do that via Cargo by creating a `.cargo/config.toml` file (see [this example from duchess itself](https://github.com/duchess-rs/duchess/blob/main/.cargo/config.toml)).

If your Rust project uses external JAR files, you may want to configure it to download them as part of the build. The [viper test crate](https://github.com/duchess-rs/duchess/tree/main/test-crates/viper) gives an example of how to do that. It uses a [build.rs](https://github.com/duchess-rs/duchess/blob/main/test-crates/viper/build.rs) file.