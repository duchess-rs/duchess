//@ check-pass

#![allow(dead_code)]

use duchess::java::lang::String as JavaString;
use duchess::java::util::ArrayList as JavaList;
use duchess::prelude::*;
use duchess::{Global, Local};

// Test that `to_java` can accomodate a Rust vector of (local) Java objects
// and produce a Java list of Java objects.
fn produce_from_local_rust_vec(r: &Vec<Local<'_, JavaString>>) {
    let _data: Option<Global<JavaList<JavaString>>> = r.to_java().global().execute().unwrap();
}

// Test that `to_java` can accomodate a Rust vector of (global) Java objects
// and produce a Java list of Java objects.
fn produce_from_global_rust_vec(r: &Vec<Global<JavaString>>) {
    let _data: Option<Global<JavaList<JavaString>>> = r.to_java().global().execute().unwrap();
}

// Test that `to_java` can accomodate a global Java object.
fn produce_from_global_object(r: Global<JavaString>) {
    let _data: Option<Global<JavaString>> = r.to_java().global().execute().unwrap();
}

// Test that `to_java` can accomodate a local Java object.
fn produce_from_local_object(r: Local<'_, JavaString>) {
    let _data: Option<Global<JavaString>> = r.to_java().global().execute().unwrap();
}

// Test that `to_java` can accomodate an optional local Java object.
fn produce_from_optlocal_object(r: Option<Local<'_, JavaString>>) {
    let _data: Option<Global<JavaString>> = r.to_java().global().execute().unwrap();
}

// Test that `to_java` can accomodate a ref to an optional local Java object.
fn produce_from_optlocal_object_ref(r: &Option<Local<'_, JavaString>>) {
    let _data: Option<Global<JavaString>> = r.to_java().global().execute().unwrap();
}

fn main() {}
