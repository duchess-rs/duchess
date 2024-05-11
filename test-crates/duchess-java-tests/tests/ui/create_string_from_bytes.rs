//@ run

use duchess::{prelude::*, IntoJava};

fn main() -> duchess::Result<()> {
    let v = vec!['H' as u8, 'i' as u8];

    let n: Java<java::lang::String> = java::lang::String::new(v.to_java()).execute()?;

    let n: String = n.execute()?;

    assert_eq!(&n[..], "Hi");

    Ok(())
}
