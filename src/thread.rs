use std::cell::Cell;

use crate::{
    raw::{EnvPtr, JvmPtr},
    Error, GlobalResult,
};

// XX: The current thread-local state will prevent duchess => java => duchess call stacks. We may want to relax this in
// the future!
thread_local! {
    static STATE: Cell<State> = Cell::new(State::Detached);
}

#[derive(Debug, PartialEq, Eq)]
pub enum State {
    /// The JVM is attached to the current thread, but we're already inside a duchess frame.
    InUse,
    /// The JVM is permanently attached to the current thread, but we're not inside a duchess frame.
    AttachedPermanently(EnvPtr<'static>),
    /// Duchess thinks the JVM is detached, though JNI calls through other means could change this.
    Detached,
}

fn attached_or(
    jvm: JvmPtr,
    f: impl FnOnce() -> GlobalResult<AttachGuard>,
) -> GlobalResult<AttachGuard> {
    STATE.with(|state| match state.replace(State::InUse) {
        State::AttachedPermanently(env) => Ok(AttachGuard {
            jvm,
            env,
            permanent: true,
        }),
        State::InUse => Err(Error::NestedUsage),
        State::Detached => {
            let result = f();
            if result.is_err() {
                state.set(State::Detached);
            }
            result
        }
    })
}

/// Marks the current thread as attached until `detach_from_jni_callback` is called.
/// Intended for use within JNI calls of native functions.
/// Returns the previous thread state, which should be given to `detach_from_jni_callback`
/// as a parameter.
///
/// # Safety condition
///
/// Must be used inside of a function that has been invoked from the JVM,
/// which guarantees that the current thread is attached and will stay that way.
///
/// Caller must invoke `detach_from_jni_callback` before returning control to the JVM.
#[must_use = "remember to call `detach_from_jni_callback`"]
pub unsafe fn attach_from_jni_callback(env: EnvPtr<'_>) -> State {
    let env: EnvPtr<'static> = unsafe { std::mem::transmute(env) };
    STATE.with(|state| state.replace(State::AttachedPermanently(env)))
}

/// Restores the thread attachment state to whatever it was before the JNI invocation
/// began.
///
/// # Safety condition
///
/// Must be called inside of a function that has been invoked from the JVM
/// with the value returned by `attach_from_jni_callback`.
pub unsafe fn detach_from_jni_callback(env: EnvPtr<'_>, s0: State) {
    let env: EnvPtr<'static> = unsafe { std::mem::transmute(env) };
    STATE.with(|state| {
        let s1 = state.replace(s0);
        assert!(
            s1 == State::AttachedPermanently(env),
            "invalid prior state `{s1:?}`"
        );
    });
}

pub fn attach_permanently(jvm: JvmPtr) -> GlobalResult<AttachGuard> {
    attached_or(jvm, || {
        Ok(AttachGuard {
            jvm,
            // no-op if already attached outside of duchess
            env: unsafe { jvm.attach_thread()? },
            permanent: true,
        })
    })
}

pub unsafe fn attach<'jvm>(jvm: JvmPtr) -> GlobalResult<AttachGuard> {
    attached_or(jvm, || {
        Ok(AttachGuard {
            jvm,
            // no-op if already attached outside of duchess
            env: unsafe { jvm.attach_thread()? },
            permanent: false,
        })
    })
}

/// When dropped, will detach the current thread from the JVM unless it was permanently attached.
pub struct AttachGuard {
    jvm: JvmPtr,
    env: EnvPtr<'static>, // not send!
    permanent: bool,
}

impl Drop for AttachGuard {
    fn drop(&mut self) {
        if self.permanent {
            STATE.with(|state| {
                let old_state = state.replace(State::AttachedPermanently(self.env));
                debug_assert!(matches!(old_state, State::InUse))
            });
        } else {
            match unsafe { self.jvm.detach_thread() } {
                Ok(()) => STATE.with(|state| state.set(State::Detached)),
                Err(err) => tracing::warn!(?err, "couldn't detach thread from JVM"),
            }
        }
    }
}

impl AttachGuard {
    pub fn env(&mut self) -> EnvPtr<'_> {
        self.env
    }
}
