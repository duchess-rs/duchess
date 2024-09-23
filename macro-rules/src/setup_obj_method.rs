#[macro_export]
macro_rules! setup_obj_method {
    (
        struct_name: [$S:ident],
        java_class_generics: [$($G:ident,)*],
        rust_method_name: [$rust_method_name:ident],
        rust_method_generics: [$($MG:ident,)*],
        input_names: [$($input_name:ident,)*],
        input_traits: [$($input_trait:path,)*],
        output_trait: [$output_trait:path],
        sig_where_clauses: [$($sig_where_clause:tt)*],
    ) => {
        pub fn $rust_method_name<'a, $($MG,)*>(
            &'a self,
            $($input_name: impl $input_trait + 'a,)*
        ) -> impl $output_trait + 'a
        where
            $($sig_where_clause)*
        {
            <$S<$($G,)*>>::$rust_method_name(
                &self.this,
                $($input_name,)*
            )
        }
    }
}
