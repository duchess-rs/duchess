/// Used to conditionally include code.
///
/// # Examples
///
/// * `if [] { ... }` expands to nothing, because `[]` is empty.
/// * `if [ ...1 ] { ...2 }` expands to `...2`, presuming `...1` is non-empty.
/// * `if false { ... }` expands to nothing, because `[]` is empty.
/// * `if true { ...2 }` expands to `...2`.
/// * `if is_ref_ty(int) { ...2 }` expands to nothing.
/// * `if is_ref_ty((class[java::lang::Object])) { ...2 }` expands to `...2`.
#[macro_export]
macro_rules! macro_if {
    // With `[]`, test if we have an empty input.
    (if [] { $($then:tt)* }) => {};
    (if [$($input:tt)+] { $($then:tt)* }) => {
        $($then)*
    };

    // With `false` or `true`, test a statically known boolean
    (if false { $($then:tt)* }) => {};
    (if true { $($then:tt)* }) => {
        $($then)*
    };

    // Testing what kind of type we have (scalar vs ref).
    // As described in [`crate::java_types`][],
    // scalar java types like `int` are an identifier;
    // reference types like `Object` are represented with a parenthesied `(...)` token tree.
    (if is_ref_ty($t:ident) { $($then:tt)* }) => {};
    (if is_ref_ty(($($t:tt)*)) { $($then:tt)* }) => {
        $($then)*
    };
}
