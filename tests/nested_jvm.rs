use duchess::Jvm;

#[test]
fn nested_jvm_with() {
    Jvm::with(|_jvm| {
        let err = Jvm::with(|_jvm| Ok(())).expect_err("nested JVMs are illegal");
        assert!(matches!(err, duchess::Error::NestedUsage));
        Ok(())
    })
    .expect("returns Ok")
}
