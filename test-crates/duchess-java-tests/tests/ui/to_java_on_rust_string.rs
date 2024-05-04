//@ run

#![allow(dead_code)]

use duchess::java;
use duchess::prelude::*;

fn main() -> duchess::Result<()> {
    let data = format!("Hello, Duchess!");
    let hash_code: i32 = data
        .to_java::<java::lang::String>() // Returns a `JvmOp` producing a `java::lang::String`
        .hash_code() // Returns a `JvmOp` invoking `hashCode` on this string
        .execute()?; // Execute the jvmop

    // NB: [hashCode for string is documented](https://docs.oracle.com/javase/8/docs/api/java/lang/String.html#hashCode--)
    assert_eq!(hash_code, -928531272);
    Ok(())
}
