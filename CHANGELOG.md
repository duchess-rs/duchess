# 0.2 (May 17th 2024)
This release contains several breaking changes to be aware of:
1. The public API has been simplfied: Duchess references are now "global" references by default. The `to_rust`, `global`, and `execute` combinators have all been merged. You now invoke `execute` and then the result depends on the return value: returning a `Java<T>` will create a global reference (matching the previous behavior of `global`), and returning a Rust value like `String` will invoke the "to rust" conversion (like `to_rust` used to do). For context and examples of upgrading see https://github.com/duchess-rs/duchess/pull/147.

2. `Jvm::with` has been removed. You can no longer obtain explicit handles to the JVM, preventing panics due to nested `Jvm::with` invocations. For context and examples see https://github.com/duchess-rs/duchess/pull/147.

3. `JvmOp`, the type returned by most Duchess operations-in-progress is now `#[must_use]`. If you encounter this error in your code, note that the code as written had no effect. `JvmOp` does nothing unless `.execute()` is called.