use duchess::prelude::*;

#[test]
fn test_jvm_construction() {
    java::lang::Object::new().execute().unwrap();
    java::lang::Object::new().execute().unwrap();
    java::lang::Object::new().execute().unwrap();
}
