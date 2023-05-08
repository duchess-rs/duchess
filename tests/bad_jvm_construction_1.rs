use duchess::Jvm;

#[test]
fn test_jvm_construction_error() {
    eprintln!("using default");
    Jvm::with(|_jvm| Ok(())).unwrap();
    eprintln!("trying to build our own");
    let res = Jvm::builder().try_launch();
    eprintln!("built our own");
    assert!(matches!(res, Err(duchess::Error::JvmAlreadyExists)));
    eprintln!("exiting");
}
