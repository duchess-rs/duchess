//@compile-flags: --crate-type cdylib
//@check-pass
//@rustc-env: CLASSPATH=../target/tests/java_ui

use duchess::prelude::*;

duchess::java_package! {
    package java_to_rust_greeting;

    public class Java_Can_Call_Rust_Java_Function {
        native java.lang.String base_greeting(java.lang.String);
    }
}

#[duchess::java_function(java_to_rust_greeting.Java_Can_Call_Rust_Java_Function::base_greeting)]
fn base_greeting(
    _this: &java_to_rust_greeting::Java_Can_Call_Rust_Java_Function,
    name: &java::lang::String,
) -> duchess::Result<Java<java::lang::String>> {
    Ok(name.execute().unwrap())
}
