use duchess::Jvm;

#[test]
fn test_jvm_construction_error() {
    Jvm::with(|_jvm| Ok(())).unwrap();
    let res = Jvm::builder().launch(|_jvm| Ok(()));
    assert!(res.is_err());
}
