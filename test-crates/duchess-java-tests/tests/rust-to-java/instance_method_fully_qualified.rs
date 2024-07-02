//@run
use duchess::prelude::*;

pub fn main() -> duchess::Result<()> {
    let date = java::util::Date::new().execute()?;
    let s: String = java::lang::Object::to_string(&date)
        .assert_not_null()
        .execute()?;
    println!("Today's date is {s}");
    Ok(())
}
