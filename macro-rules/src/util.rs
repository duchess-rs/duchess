#[macro_export]
macro_rules! macro_if {
    (if [] { $($then:tt)* }) => {};

    (if false { $($then:tt)* }) => {};

    (if [$($input:tt)+] { $($then:tt)* }) => {
        $($then)*
    };

    (if true { $($then:tt)* }) => {
        $($then)*
    };

    // Testing what kind of type we have (scalar vs ref)
    //
    // see java_types.rs

    (if is_ref_ty($t:ident) { $($then:tt)* }) => {};

    (if is_ref_ty($t:tt) { $($then:tt)* }) => {
        $($then)*
    };

}
