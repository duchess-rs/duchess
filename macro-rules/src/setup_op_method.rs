#[macro_export]
macro_rules! setup_op_method {
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
        pub fn $rust_method_name<$($MG,)*>(
            &self,
            $($input_name: impl $input_trait,)*
        ) -> impl $output_trait
        where
            $($sig_where_clause)*
        {
            <$S<$($G,)*>>::$rust_method_name(
                Clone::clone(&self.this),
                $($input_name,)*
            )
        }

    }
}
