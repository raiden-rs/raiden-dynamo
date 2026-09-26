use quote::*;

pub fn expand_key_condition_builder(
    attr_enum_name: &proc_macro2::Ident,
    struct_name: &proc_macro2::Ident,
    partition_key: &(proc_macro2::Ident, syn::Type),
    sort_key: &Option<(proc_macro2::Ident, syn::Type)>,
) -> proc_macro2::TokenStream {
    let key_condition_token_name = format_ident!("{}KeyConditionToken", struct_name);
    let partition_token_name = format_ident!("{}TablePartitionKeyConditionToken", struct_name);
    let sort_token_name = format_ident!("{}TableSortKeyConditionToken", struct_name);
    let terminal_token_name = format_ident!("{}TableTerminalKeyConditionToken", struct_name);
    let partition_attr_name = partition_key.0.to_string();
    let next_token_name = if sort_key.is_some() {
        &sort_token_name
    } else {
        &terminal_token_name
    };
    let sort_condition = sort_key.as_ref().map(|(name, _)| {
        let sort_attr_name = name.to_string();
        quote! {
            pub struct #sort_token_name;
            impl ::raiden::key_condition::SupportsEqCondition for #sort_token_name {}
            impl ::raiden::key_condition::SupportsRangeCondition for #sort_token_name {}

            impl #struct_name {
                /// Starts a key condition for the table sort key.
                /// Chain this after `partition_key_condition().eq(...)`.
                pub fn sort_key_condition() -> ::raiden::KeyCondition<#sort_token_name, #terminal_token_name> {
                    ::raiden::KeyCondition {
                        attr: #sort_attr_name.to_owned(),
                        _token: std::marker::PhantomData,
                        _next_token: std::marker::PhantomData,
                    }
                }
            }
        }
    });
    quote! {

        pub struct #key_condition_token_name;

        impl ::raiden::key_condition::SupportsEqCondition for #key_condition_token_name {}
        impl ::raiden::key_condition::SupportsRangeCondition for #key_condition_token_name {}

        pub struct #partition_token_name;
        impl ::raiden::key_condition::SupportsEqCondition for #partition_token_name {}
        pub struct #terminal_token_name;
        #sort_condition

        impl<U> ::raiden::key_condition::KeyConditionBuilder<#key_condition_token_name, U>
            for ::raiden::key_condition::KeyConditionFilledOrWaitOperator<#partition_token_name, U>
        {
            fn build(self) -> (
                ::raiden::key_condition::KeyConditionString,
                ::raiden::AttributeNames,
                ::raiden::AttributeValues,
            ) {
                ::raiden::key_condition::KeyConditionBuilder::<#partition_token_name, U>::build(self)
            }
        }

        impl<U> ::raiden::key_condition::KeyConditionBuilder<#key_condition_token_name, U>
            for ::raiden::key_condition::KeyConditionFilled<#partition_token_name, U>
        {
            fn build(self) -> (
                ::raiden::key_condition::KeyConditionString,
                ::raiden::AttributeNames,
                ::raiden::AttributeValues,
            ) {
                ::raiden::key_condition::KeyConditionBuilder::<#partition_token_name, U>::build(self)
            }
        }

        impl #struct_name {
            /// Starts a key condition for the table partition key.
            /// Only equality is available for the partition key.
            pub fn partition_key_condition() -> ::raiden::KeyCondition<#partition_token_name, #next_token_name> {
                ::raiden::KeyCondition {
                    attr: #partition_attr_name.to_owned(),
                    _token: std::marker::PhantomData,
                    _next_token: std::marker::PhantomData,
                }
            }

            /// Builds an attribute-based key condition for backward compatibility.
            /// This method does not restrict the attribute to table keys. Prefer
            /// `partition_key_condition` and `sort_key_condition` for table queries.
            pub fn key_condition(attr: #attr_enum_name) -> ::raiden::KeyCondition<#key_condition_token_name, #key_condition_token_name> {
                let attr = attr.into_attr_name();
                ::raiden::KeyCondition {
                    attr,
                    _token: std::marker::PhantomData,
                    _next_token: std::marker::PhantomData,
                }
            }
        }
    }
}
