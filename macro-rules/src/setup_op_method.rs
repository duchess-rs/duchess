#[macro_export]
macro_rules! setup_op_method {
    (
        struct_name: [$S:ident],
        java_class_generics: [$($G:ident,)*],
        rust_method_name: [$M:ident],
        rust_method_generics: [$($MG:ident,)*],
        input_names: [$($I:ident,)*],
        input_ty_tts: [$($I_ty:tt,)*],
        output_ty_tt: [$O_ty:tt],
        sig_where_clauses: [$($SIG:tt)*],
    ) => {
        pub fn $M<$($MG,)*>(
            &self,
            $($I: duchess::semver_unstable::argument_impl_trait!($I_ty),)*
        ) -> duchess::semver_unstable::output_trait!($O_ty)
        where
            $($SIG)*
        {
            <$S<$($G,)*>>::$M(
                Clone::clone(&self.this),
                $($I,)*
            )
        }
    }
}
