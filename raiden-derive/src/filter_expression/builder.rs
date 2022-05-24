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
                ::raiden::FilterExpression {
                    attr,
                    _token: std::marker::PhantomData,
                }
            }
            pub fn filter_expression_with_not(builder: impl ::raiden::FilterExpressionBuilder<#filter_expression_token_name>) -> ::raiden::FilterExpressionNotWrapper<#filter_expression_token_name> {
                let (str, names, values) = builder.build();
                ::raiden::FilterExpressionNotWrapper{
                    condition_string: str,
                    attr_names: names,
                    attr_values: values,
                    _token: std::marker::PhantomData,
                }
            }
        }
    }
}
