//@check-pass
use duchess::prelude::*;

duchess::java_package! {
    package java_rust_scalars;

    public class JavaRustScalars {
        native int echoInt(int);
        native long echoLong(long);
        native double echoDouble(double);
        native byte echoByte(byte);
        native short echoShort(short);
        native float echoFloat(float);
        native char echoChar(char);
    }
}

#[duchess::java_function(java_rust_scalars.JavaRustScalars::echoInt)]
fn echo_int(_this: &java_rust_scalars::JavaRustScalars, input: i32) -> duchess::Result<i32> {
    Ok(input)
}

#[duchess::java_function(java_rust_scalars.JavaRustScalars::echoLong)]
fn echo_long(_this: &java_rust_scalars::JavaRustScalars, input: i64) -> duchess::Result<i64> {
    Ok(input)
}

#[duchess::java_function(java_rust_scalars.JavaRustScalars::echoDouble)]
fn echo_double(_this: &java_rust_scalars::JavaRustScalars, input: f64) -> duchess::Result<f64> {
    Ok(input)
}

#[duchess::java_function(java_rust_scalars.JavaRustScalars::echoByte)]
fn echo_byte(_this: &java_rust_scalars::JavaRustScalars, input: i8) -> duchess::Result<i8> {
    Ok(input)
}

#[duchess::java_function(java_rust_scalars.JavaRustScalars::echoShort)]
fn echo_short(_this: &java_rust_scalars::JavaRustScalars, input: i16) -> duchess::Result<i16> {
    Ok(input)
}

#[duchess::java_function(java_rust_scalars.JavaRustScalars::echoFloat)]
fn echo_float(_this: &java_rust_scalars::JavaRustScalars, input: f32) -> duchess::Result<f32> {
    Ok(input)
}

#[duchess::java_function(java_rust_scalars.JavaRustScalars::echoChar)]
fn echo_char(_this: &java_rust_scalars::JavaRustScalars, input: u16) -> duchess::Result<u16> {
    Ok(input)
}
