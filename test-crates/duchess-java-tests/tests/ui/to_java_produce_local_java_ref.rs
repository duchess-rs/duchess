//@run

#![allow(dead_code)]

use duchess::prelude::*;
use duchess::Local;
use java::lang::String as JavaString;
use java::util::ArrayList as JavaList;

// Test that `to_java` can accomodate a Rust vector of (local) Java objects
// and produce a Java list of Java objects.
fn produce_from_local_rust_vec(r: &Vec<Local<'_, JavaString>>) {
    let _data: Option<Java<JavaList<JavaString>>> =
        r.to_java::<JavaList<JavaString>>().execute().unwrap();
}

fn main() {}
