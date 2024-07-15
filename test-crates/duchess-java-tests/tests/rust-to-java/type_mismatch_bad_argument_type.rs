//@compile-flags: --crate-type lib
use duchess::prelude::*;

duchess::java_package! {
    package type_mismatch;

    public class TakesInt {*}
}

fn call_with_u64(i: u64) {
    type_mismatch::TakesInt::new().take(i).execute();
    //~^ERROR: the trait bound `u64: duchess::IntoScalar<i32>` is not satisfied
    //~|ERROR: the trait bound `u64: duchess::JvmOp` is not satisfied
}

fn call_with_u32(i: u32) {
    type_mismatch::TakesInt::new().take(i).execute();
    //~^ ERROR: the trait bound `u32: duchess::JvmOp` is not satisfied
    //~| ERROR: the trait bound `u32: duchess::IntoScalar<i32>` is not satisfied
}

fn call_with_i32(i: i32) {
    type_mismatch::TakesInt::new().take(i).execute();
    // OK
}

fn main() {}
