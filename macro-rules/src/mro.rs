/// Generates a reference to the op or obj struct for a given type.
/// This type is named by using the associated types on the `duchess::semver_unstable::JavaView` trait.
/// It expands to a recursive type that encodes the [method resolution order][mro] from the
/// original source type.
///
/// [mro]: https://duchess-rs.github.io/duchess/methods.html
#[macro_export]
macro_rules! mro {
    ($J:ident,$assoc_name:ident,[]) => {
        // The sequence terminates on `()`
        ()
    };

    ($J:ident,$assoc_name:ident,[$mro_ty_head:ty, $($mro_ty_tail:ty,)*]) => {
        // The head type is the type we are viewing our original object as
        // (some superclass/interface of the original type).
        <$mro_ty_head as duchess::semver_unstable::JavaView>::$assoc_name<
            // J here represents the type we are coming *from*
            $J,

            // N here represents the next type in the MRO sequence,
            // which is generated recursively
            duchess::semver_unstable::mro!($J, $assoc_name, [$($mro_ty_tail,)*]),
        >
    };
}
