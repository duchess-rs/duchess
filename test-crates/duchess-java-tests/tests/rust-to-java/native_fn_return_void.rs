//@ run

use duchess::{java, prelude::*};
use std::sync::atomic::{AtomicUsize, Ordering};

duchess::java_package! {
    package native_void;

    public class native_void.NativeVoid {
        public native_void.NativeVoid();
        public void fire(java.lang.String);
        native void record(java.lang.String);
        public static void fireStatic(java.lang.String);
        static native void recordStatic(java.lang.String);
    }
}

static INSTANCE_CALLS: AtomicUsize = AtomicUsize::new(0);
static STATIC_CALLS: AtomicUsize = AtomicUsize::new(0);

#[duchess::java_function(native_void.NativeVoid::record)]
fn record(_this: &native_void::NativeVoid, name: Option<&java::lang::String>) {
    let name: String = name.assert_not_null().execute().expect("null name");
    assert_eq!(name, "Ferris");
    INSTANCE_CALLS.fetch_add(1, Ordering::SeqCst);
}

#[duchess::java_function(native_void.NativeVoid::recordStatic)]
fn record_static(name: Option<&java::lang::String>) {
    let name: String = name.assert_not_null().execute().expect("null name");
    assert_eq!(name, "Ferris");
    STATIC_CALLS.fetch_add(1, Ordering::SeqCst);
}

fn main() -> duchess::Result<()> {
    duchess::Jvm::builder()
        .link(record::java_fn())
        .link(record_static::java_fn())
        .try_launch()?;

    native_void::NativeVoid::new().fire("Ferris").execute()?;
    native_void::NativeVoid::fire_static("Ferris").execute()?;

    assert_eq!(INSTANCE_CALLS.load(Ordering::SeqCst), 1);
    assert_eq!(STATIC_CALLS.load(Ordering::SeqCst), 1);

    Ok(())
}
