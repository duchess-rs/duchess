//@run
use duchess::{java, prelude::*};

#[derive(duchess::ToJava)]
#[java(java.lang.Long::decode)]
struct LongWrapper<'a> {
    value: &'a str,
}

pub fn main() -> duchess::GlobalResult<()> {
    let my_string = String::from("1234");
    let rust = LongWrapper { value: &my_string };
    let java = rust.to_java().assert_not_null().global().execute()?;
    let and_back: String = java.to_string().execute().unwrap().unwrap();
    assert_eq!(rust.value, and_back);
    Ok(())
}
