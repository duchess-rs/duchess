use duchess::prelude::*;

#[test]
fn test_construct_silicon() -> duchess::GlobalResult<()> {
    duchess::Jvm::with(|jvm| {
        use viper::viper::silicon::Silicon;
        let _silicon = Silicon::new().execute(jvm)?;
        Ok(())
    })
}
