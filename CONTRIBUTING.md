# Contributing

### Running Tests
Tests are split into two parts:
1. Unit tests, run by running `cargo test` in the root directory.
2. Integration tests (containing usages of the proc macro and ui-tests): `(cd test-crates && cargo test)`

You can run both of these with `just test` (see [just](https://github.com/casey/just)) for more information. Just is not required to contribute but may save you a small amount of time.

The [UI tests](test-crates) are currently tested against Rust 1.79.0

### Debugging
Duchess looks for the `DUCHESS_DEBUG` environment variable during proc-macro expansion. When this variable is set, if it is `true` or `1`, **all** generated code will be formatted and dumped to a directory. Clickable links are printed to stderr:

For example:
```
file:////var/folders/20/gm3mpm1n6lj3r3tb6q2hx2_80000gr/T/.tmpjJadBD/auth_Authenticated.rs
file:////var/folders/20/gm3mpm1n6lj3r3tb6q2hx2_80000gr/T/.tmpjJadBD/auth_AuthorizeRequest.rs
file:////var/folders/20/gm3mpm1n6lj3r3tb6q2hx2_80000gr/T/.tmpjJadBD/auth_HttpAuth.rs
```

If you want to filter specific Java class paths, you can pass a string like `auth` or `java.lang`:
```
DUCHESS_DEBUG=java.lang
```

This will only dump debug information for these specific classes.
