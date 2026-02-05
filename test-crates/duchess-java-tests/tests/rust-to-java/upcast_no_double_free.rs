//@run

use duchess::prelude::*;

pub fn main() -> duchess::Result<()> {
    let java_string: Java<java::lang::String> = "test".to_java::<java::lang::String>().execute()?.unwrap();
    let java_object: Java<java::lang::Object> = java_string.upcast::<java::lang::Object>();

    Ok(())
}
