use duchess::Jvm;

#[test]
fn test_jvm_construction_error() {
    eprintln!("using default");
    Jvm::with(|_jvm| Ok(())).unwrap();
    let res = Jvm::builder().try_launch();
    assert!(matches!(res, Err(duchess::Error::JvmAlreadyExists)));
}
