//@ check-pass

#![allow(dead_code)]

use duchess::java::lang::String as JavaString;
use duchess::java::util::ArrayList as JavaList;
use duchess::prelude::*;
use duchess::Local;

// Test that `to_java` can accomodate a Rust vector of (local) Java objects
// and produce a Java list of Java objects.
fn produce_from_local_rust_vec(r: &Vec<Local<'_, JavaString>>) {
    duchess::Jvm::with(|jvm| {
        let _data: Option<Local<'_, JavaList<JavaString>>> = r.to_java().execute_with(jvm).unwrap();
        Ok(())
    })
    .unwrap();
}

fn main() {}
