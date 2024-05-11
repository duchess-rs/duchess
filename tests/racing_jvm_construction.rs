use duchess::prelude::*;
use std::{sync, thread};

#[test]
fn race_multiple_threads_to_launch_jvm() {
    let n = 10;
    let barrier = sync::Arc::new(sync::Barrier::new(10));
    thread::scope(|scope| {
        for _ in 0..n {
            let barrier = sync::Arc::clone(&barrier);
            scope.spawn(move || {
                barrier.wait();
                java::lang::Object::new().execute().unwrap();
            });
        }
    });
}
