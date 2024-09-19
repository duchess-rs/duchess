#[macro_export]
macro_rules! mro {
    ($J:ident,$assoc_name:ident,[]) => {
        ()
    };

    ($J:ident,$assoc_name:ident,[$mro_ty_head:ty, $($mro_ty_tail:ty,)*]) => {
        <$mro_ty_head as duchess::plumbing::JavaView>::$assoc_name<
            $J,
            duchess::plumbing::mro!($J, $assoc_name, [$($mro_ty_tail,)*]),
        >
    };
}
