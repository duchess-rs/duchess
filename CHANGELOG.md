# 0.3.0 (July 22nd, 2024)
This release contains many improvements for calling Rust code from Java:
1. Add support for returning scalars (#181)
2. Allow specifying a minimum JNI version (#180)

**Breaking changes**:
1. `class` or `interface` when specified in `java_package` must actually match (#168). If you get an error after upgrading, change the keyword in your `java_package` macro to match the actual type in Java.
2. A `duchess-reflect` crate has also been split out from the macro package.

**Bug fixes**:
* Fix bug where passing `None` for `Option<T>` resulted in a spurious error from Duchess (#182).

# 0.2.1 (June 4th, 2024)
* Add `JMX` APIs to Java prelude. These allow querying the current memory usage of the JVM.

# 0.2 (May 17th, 2024)
This release contains several breaking changes to be aware of:
1. The public API has been simplfied: Duchess references are now "global" references by default. The `to_rust`, `global`, and `execute` combinators have all been merged. You now invoke `execute` and then the result depends on the return value: returning a `Java<T>` will create a global reference (matching the previous behavior of `global`), and returning a Rust value like `String` will invoke the "to rust" conversion (like `to_rust` used to do). For context and examples of upgrading see https://github.com/duchess-rs/duchess/pull/147.

2. `Jvm::with` has been removed. You can no longer obtain explicit handles to the JVM, preventing panics due to nested `Jvm::with` invocations. For context and examples see https://github.com/duchess-rs/duchess/pull/147.

3. `JvmOp`, the type returned by most Duchess operations-in-progress is now `#[must_use]`. If you encounter this error in your code, note that the code as written had no effect. `JvmOp` does nothing unless `.execute()` is called.