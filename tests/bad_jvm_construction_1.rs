use duchess::Jvm;

#[test]
fn test_jvm_construction_error() {
    println!("using default");
    Jvm::with(|_jvm| Ok(())).unwrap();
    println!("trying to build our own");
    let res = Jvm::builder().try_launch();
    println!("built our own");
    assert!(matches!(res, Err(duchess::Error::JvmAlreadyExists)));
    println!("exiting");
}
