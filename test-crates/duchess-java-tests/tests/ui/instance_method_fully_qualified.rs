//@check-pass
use duchess::java;
use duchess::prelude::*;

pub fn main() -> duchess::GlobalResult<()> {
    let date = java::util::Date::new().global().execute()?;
    let s = java::lang::Object::to_string(&date)
        .assert_not_null()
        .to_rust()?;
    println!("Today's date is {s}");
    Ok(())
}
