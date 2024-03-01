use duchess::{java, prelude::*, Global};

#[derive(duchess::ToJava)]
#[java(java.lang.Long::decode)]
struct LongWrapper<'a> {
    value: &'a str,
}