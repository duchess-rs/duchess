use duchess::prelude::*;
use duchess::Jvm;

#[test]
fn test_jvm_construction_error() {
    java::lang::Object::new().execute().unwrap();
    let res = Jvm::builder().try_launch();
    assert!(matches!(res, Err(duchess::Error::JvmAlreadyExists)));
}
