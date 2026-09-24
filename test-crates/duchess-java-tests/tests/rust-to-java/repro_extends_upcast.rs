//@run
use duchess::prelude::*;
use duchess::{Jvm, Local, LocalResult};

// Regression test: owned-handle `upcast()` must transfer JNI ref ownership
// to the new handle. It used to keep both handles owning the same ref, so
// the first drop deleted it out from under the second ("Bad global or local
// ref passed to JNI", SIGABRT).
duchess::java_package! {
    package upcast;

    public class upcast.Base {
        public upcast.Base();
        public int answer();
    }

    public class upcast."Base$Sub" extends upcast.Base {
        public upcast."Base$Sub"();
    }
}

pub fn main() -> duchess::Result<()> {
    let sub: duchess::Java<upcast::Base__Sub> =
        upcast::Base__Sub::new().assert_not_null().execute()?;
    let base: duchess::Java<upcast::Base> = sub.upcast();
    // Use the upcast handle (not just drop it) so a dangling ref faults here.
    assert_eq!(base.answer().execute()?, 42);

    // Same coverage for `Local<T>::upcast`: a custom op is needed to reach a
    // `Local` handle, since `execute()` globalizes to `Java`.
    let v: i32 = LocalUpcastAnswer.execute()?;
    assert_eq!(v, 42);
    Ok(())
}

#[derive(Clone)]
struct LocalUpcastAnswer;

impl JvmOp for LocalUpcastAnswer {
    type Output<'jvm> = i32;

    fn do_jni<'jvm>(self, jvm: &mut Jvm<'jvm>) -> LocalResult<'jvm, i32> {
        let sub: Local<'jvm, upcast::Base__Sub> =
            upcast::Base__Sub::new().assert_not_null().do_jni(jvm)?;
        let base: Local<'jvm, upcast::Base> = sub.upcast();
        // Use the upcast handle (not just drop it) so a dangling ref faults here.
        (&base).answer().do_jni(jvm)
    }
}
