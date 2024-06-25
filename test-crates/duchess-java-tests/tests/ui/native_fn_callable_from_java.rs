//@compile-flags: --crate-type cdylib
//@check-pass
//@rustc-env: CLASSPATH=../target/tests/java_ui

use duchess::prelude::*;

duchess::java_package! {
    package java_to_rust_greeting;

    public class JavaCanCallRustJavaFunction {
        native java.lang.String baseGreeting(java.lang.String);
    }
}

#[duchess::java_function(java_to_rust_greeting.JavaCanCallRustJavaFunction::baseGreeting)]
fn base_greeting(
    _this: &java_to_rust_greeting::JavaCanCallRustJavaFunction,
    name: &java::lang::String,
) -> duchess::Result<Java<java::lang::String>> {
    Ok(name.execute().unwrap())
}
