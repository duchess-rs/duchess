use duchess::prelude::*;
use std::sync::Once;

static INIT: Once = Once::new();

fn setup_tests() {
    duchess::Jvm::builder().try_launch().unwrap();
}

#[test]
fn test_construct_silicon() -> duchess::GlobalResult<()> {
    INIT.call_once(setup_tests);
    duchess::Jvm::with(|jvm| {
        use viper::viper::silicon::Silicon;
        let _silicon = Silicon::new().execute_with(jvm)?;
        Ok(())
    })
}
