//@ run
use jni::JavaVM;

fn main() -> duchess::Result<()> {
    let args = jni::InitArgsBuilder::new()
        .build()
        .expect("Failed to build JVM InitArgs");
    let jvm = JavaVM::new(args).expect("Failed to build JVM");
    duchess::Jvm::builder().launch_or_use_existing().unwrap();
    Ok(())
}
