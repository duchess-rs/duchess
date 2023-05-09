# Reference

## Features

### `dylibjvm`

`libjvm` can be either statically or dynamically linked. If the `dylibjvm` feature is enabled, `duchess` will dynamically load `libjvm` when trying to create or find a JVM. Unless the lib path is specified in `JvmBuilder::load_libjvm_at()`, it uses the `java-locator` crate to find the likely location of `libjvm` on the platform.