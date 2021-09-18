use proc_macro2::*;
use quote::*;
use syn::*;

pub(crate) fn expand_get_item(
    partition_key: &(Ident, Type),
    sort_key: &Option<(Ident, Type)>,
    struct_name: &Ident,
    fields: &syn::FieldsNamed,
    rename_all_type: crate::rename::RenameAllType,
) -> TokenStream {
    let trait_name = format_ident!("{}GetItem", struct_name);
    let client_name = format_ident!("{}Client", struct_name);
    let builder_name = format_ident!("{}GetItemBuilder", struct_name);
    let from_item = super::expand_attr_to_item(&format_ident!("res_item"), fields, rename_all_type);
    let (partition_key_ident, partition_key_type) = partition_key;

    let client_trait = if let Some(sort_key) = sort_key {
        let (sort_key_ident, sort_key_type) = sort_key;
        quote! {
            pub trait #trait_name {
                fn get(&self, pk: impl Into<#partition_key_type>, sk: impl Into<#sort_key_type>) -> #builder_name;
            }

            impl #trait_name for #client_name {
                fn get(&self, pk: impl Into<#partition_key_type>, sk: impl Into<#sort_key_type>) -> #builder_name {
                    let mut input = ::raiden::GetItemInput::default();
                    let pk_attr: ::raiden::AttributeValue = pk.into().into_attr();
                    let sk_attr: ::raiden::AttributeValue = sk.into().into_attr();
                    input.projection_expression = self.projection_expression.clone();
                    input.expression_attribute_names = self.attribute_names.clone();
                    let mut key_set: std::collections::HashMap<String, ::raiden::AttributeValue> = std::collections::HashMap::new();
                    key_set.insert(stringify!(#partition_key_ident).to_owned(), pk_attr);
                    key_set.insert(stringify!(#sort_key_ident).to_owned(), sk_attr);
                    input.key = key_set;
                    input.table_name = self.table_name();
                    #builder_name {
                        client: &self.client,
                        input,
                        policy: self.retry_condition.strategy.policy(),
                        condition: &self.retry_condition,
                    }
                }
            }
        }
    } else {
        quote! {
            pub trait #trait_name {
                fn get(&self, key: impl Into<#partition_key_type>) -> #builder_name;
            }

            impl #trait_name for #client_name {
                fn get(&self, key: impl Into<#partition_key_type>) -> #builder_name {
                    let key_attr: ::raiden::AttributeValue = key.into().into_attr();
                    let mut key_set: std::collections::HashMap<String, ::raiden::AttributeValue> = std::collections::HashMap::new();
                    key_set.insert(stringify!(#partition_key_ident).to_owned(), key_attr);
                    let input = ::raiden::GetItemInput {
                        key: key_set,
                        table_name: self.table_name(),
                        projection_expression: self.projection_expression.clone(),
                        expression_attribute_names: self.attribute_names.clone(),
                        ..::raiden::GetItemInput::default()
                    };

                    #builder_name {
                        client: &self.client,
                        input,
                        policy: self.retry_condition.strategy.policy(),
                        condition: &self.retry_condition,
                    }
                }
            }
        }
    };

    quote! {
        #client_trait

        pub struct #builder_name<'a> {
            pub client: &'a ::raiden::DynamoDbClient,
            pub input: ::raiden::GetItemInput,
            pub policy: ::raiden::Policy,
            pub condition: &'a ::raiden::retry::RetryCondition,
        }

        impl<'a> #builder_name<'a> {
            pub fn consistent(mut self) -> Self {
                self.input.consistent_read = Some(true);
                self
            }

            pub async fn run(self) -> Result<::raiden::get::GetOutput<#struct_name>, ::raiden::RaidenError> {
                let policy: ::raiden::RetryPolicy = self.policy.into();
                let client = self.client;
                let input = self.input;
                policy.retry_if(move || {
                    let client = client.clone();
                    let input = input.clone();
                    async {
                        #builder_name::inner_run(client, input).await
                    }
                }, self.condition).await
            }

            async fn inner_run(client: ::raiden::DynamoDbClient, input: ::raiden::GetItemInput) -> Result<::raiden::get::GetOutput<#struct_name>, ::raiden::RaidenError> {
                let res = client.get_item(input).await?;
                if res.item.is_none() {
                    return Err(::raiden::RaidenError::ResourceNotFound("resource not found".to_owned()));
                };
                let res_item = &res.item.unwrap();
                let item = #struct_name {
                    #(#from_item)*
                };
                Ok(::raiden::get::GetOutput {
                    item,
                    consumed_capacity: res.consumed_capacity,
                })
            }
        }
    }
}

/*
https://github.com/rusoto/rusoto/blob/master/rusoto/services/dynamodb/src/generated.rs#L1137
#[cfg_attr(feature = "deserialize_structs", derive(Deserialize))]
pub struct GetItemInput {
    pub attributes_to_get: Option<Vec<String>>,
    pub consistent_read: Option<bool>,
    pub expression_attribute_names: Option<::std::collections::HashMap<String, String>>,
    pub key: ::std::collections::HashMap<String, AttributeValue>,
    pub projection_expression: Option<String>,
    pub return_consumed_capacity: Option<String>,
    pub table_name: String,
}
*/
