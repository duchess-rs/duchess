use duchess::prelude::*;

#[test]
fn test_jvm_construction() {
    duchess::Jvm::builder().try_launch().unwrap();
    java::lang::Object::new().execute().unwrap();
    java::lang::Object::new().execute().unwrap();
}
