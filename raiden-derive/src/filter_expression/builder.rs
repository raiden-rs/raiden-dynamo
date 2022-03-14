use quote::*;

pub fn expand_filter_expression_builder(
    attr_enum_name: &proc_macro2::Ident,
    struct_name: &proc_macro2::Ident,
) -> proc_macro2::TokenStream {
    let filter_expression_token_name = format_ident!("{}FilterExpressionToken", struct_name);
    quote! {

        pub struct #filter_expression_token_name;

        impl #struct_name {
            pub fn filter_expression(attr: #attr_enum_name) -> ::raiden::FilterExpression<#filter_expression_token_name> {
                let attr = attr.into_attr_name();
                ::raiden::FilterExpression::from_attr(attr)
            }
        }
    }
}
