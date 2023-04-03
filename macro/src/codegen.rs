use crate::{
    argument::DuchessDeclaration,
    class_info::{ClassInfo, SpannedClassInfo},
    span_error::SpanError,
};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote_spanned;

impl DuchessDeclaration {
    pub fn into_tokens(mut self) -> Result<TokenStream, SpanError> {
        todo!()
    }
}

impl SpannedClassInfo {
    pub fn into_tokens(mut self) -> TokenStream {
        let struct_decl = self.struct_decl();
        let cached_class = self.cached_class();

        quote_spanned! {
            self.span =>
            #struct_decl

            // Hide other generated items
            const _: () = {
                use duchess::{
                    plumbing::{
                        ArrayList, HashMap, IntoJavaArray, IntoJavaString, JavaObjectExt, JavaString, List,
                        ListExt, Map, MapExt, Upcast,
                    },
                    IntoJava, IntoRust, JavaObject, Jvm, JvmOp, Local,
                };
                use jni::{
                    objects::{AutoLocal, GlobalRef, JMethodID, JValueGen},
                    signature::ReturnType,
                    sys::jvalue,
                };
                use once_cell::sync::OnceCell;

                #cached_class
            };
        }
    }

    fn struct_decl(&self) -> TokenStream {
        let struct_name = self.struct_name();

        quote_spanned! {
            self.span =>

            pub struct #struct_name {
                _dummy: ()
            }

            unsafe impl JavaObject for #struct_name {}

            unsafe impl Upcast<#struct_name> for #struct_name {}
        }
    }

    fn cached_class(&self) -> TokenStream {
        let jni_class_name = self.jni_class_name();
        quote_spanned! {
            self.span =>

            fn cached_class(jvm: &mut Jvm<'_>) -> duchess::Result<&'static GlobalRef> {
                let env = jvm.to_env();

                static CLASS: OnceCell<GlobalRef> = OnceCell::new();
                CLASS.get_or_try_init(|| {
                    let class = env.find_class(#jni_class_name)?;
                    env.new_global_ref(class)
                })
            }
        }
    }

    fn struct_name(&self) -> Ident {
        Ident::new(&self.info.name, self.span)
    }

    /// Returns a class name with `/`, like `java/lang/Object`.
    fn jni_class_name(&self) -> String {
        self.info.name.replace(".", "/")
    }
}
