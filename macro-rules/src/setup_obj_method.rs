#[macro_export]
macro_rules! setup_obj_method {
    (
        struct_name: [$S:ident],
        java_class_generics: [$($G:ident,)*],
        rust_method_name: [$M:ident],
        rust_method_generics: [$($MG:ident,)*],
        input_names: [$($I:tt,)*],
        input_ty_tts: [$($I_ty:tt,)*],
        output_ty_tt: [$O_ty:tt],
        sig_where_clauses: [$($SIG:tt)*],
    ) => {
        pub fn $M<'a, $($MG,)*>(
            &'a self,
            $($I: duchess::semver_unstable::argument_impl_trait!($I_ty + 'a),)*
        ) -> duchess::semver_unstable::output_trait!($O_ty + 'a)
        where
            $($SIG)*
        {
            <$S<$($G,)*>>::$M(
                &self.this,
                $($I,)*
            )
        }
    }
}
