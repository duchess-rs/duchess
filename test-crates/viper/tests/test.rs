use duchess::prelude::*;
use std::sync::Once;

static INIT: Once = Once::new();

fn setup_tests() {
    let classpath = std::env::var("CLASSPATH").unwrap();
    duchess::Jvm::builder().add_classpath(classpath).launch(|_jvm| Ok(())).unwrap();
}

#[test]
fn test_construct_silicon() -> duchess::GlobalResult<()> {
    INIT.call_once(setup_tests);
    duchess::Jvm::with(|jvm| {
        use viper::viper::silicon::Silicon;
        let _silicon = Silicon::new().execute(jvm)?;
        Ok(())
    })
}
