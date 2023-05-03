use duchess::Jvm;

#[test]
fn test_jvm_construction() {
    Jvm::builder().launch(|_jvm| Ok(())).unwrap();
    Jvm::with(|_jvm| Ok(())).unwrap();
    Jvm::with(|_jvm| Ok(())).unwrap();
}
