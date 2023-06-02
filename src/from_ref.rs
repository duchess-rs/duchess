/// A trait that permits converting from a `&J` to an `&Self`.
/// This is part of the internal "plumbing" for duchess.
/// See the [user manual](https://duchess-rs.github.io/duchess/internals.html) for more information.
pub trait FromRef<J> {
    fn from_ref(j: &J) -> &Self;
}

impl<J> FromRef<J> for () {
    fn from_ref(_: &J) -> &Self {
        &()
    }
}
