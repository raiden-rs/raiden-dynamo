use quote::*;

pub fn expand_key_condition_builder(
    attr_enum_name: &proc_macro2::Ident,
    struct_name: &proc_macro2::Ident,
) -> proc_macro2::TokenStream {
    let key_condition_token_name = format_ident!("{}KenConditionToken", struct_name);
    quote! {

        pub struct #key_condition_token_name;

        impl #struct_name {
            pub fn key_condition(attr: #attr_enum_name) -> ::raiden::KeyCondition<#key_condition_token_name> {
                let attr = attr.into_attr_name();
                ::raiden::KeyCondition {
                    attr,
                    _token: std::marker::PhantomData,
                }
            }
        }
    }
}
