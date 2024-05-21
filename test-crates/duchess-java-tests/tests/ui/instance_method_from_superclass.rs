//@run
use duchess::prelude::*;

// Generate our own version of java.util.Date that
// explicitly does NOT include the `toString` method,
// so that we have to get it by upcasting to Object.
mod our_java {
    duchess::java_package! {
        package java.util;

        public class java.util.Date {
            public java.util.Date();
        }
    }
    pub use java::*;
}

pub fn main() -> duchess::Result<()> {
    let date = our_java::util::Date::new().execute()?;
    let s: String = date.to_string().assert_not_null().execute()?;
    //                   ^^^^^^^^^^^ this is defined on `java.lang.Object`
    println!("Today's date is {s}");
    Ok(())
}
