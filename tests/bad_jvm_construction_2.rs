use duchess::Jvm;

#[test]
fn test_jvm_construction_error() {
    Jvm::builder().try_launch().unwrap();
    let res = Jvm::builder().try_launch();
    assert!(matches!(res, Err(duchess::Error::JvmAlreadyExists)));
}
