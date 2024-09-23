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
}
