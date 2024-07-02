//@run
use duchess::prelude::*;

pub fn main() -> duchess::Result<()> {
    let l: i64 = java::util::Date::parse("Feb 1, 2022").execute()?;
    println!("{l}");
    Ok(())
}
