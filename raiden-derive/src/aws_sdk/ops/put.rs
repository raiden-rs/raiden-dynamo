use crate::rename::*;
use proc_macro2::*;
use quote::*;

pub(crate) fn expand_put_item(
    struct_name: &Ident,
    fields: &syn::FieldsNamed,
    rename_all_type: crate::rename::RenameAllType,
) -> TokenStream {
    let item_input_name = format_ident!("{}PutItemInput", struct_name);
    let item_input_builder_name = format_ident!("{}PutItemInputBuilder", struct_name);
    let item_output_name = format_ident!("{}PutItemOutput", struct_name);
    let trait_name = format_ident!("{}PutItem", struct_name);
    let client_name = format_ident!("{}Client", struct_name);
    let builder_name = format_ident!("{}PutItemBuilder", struct_name);
    let condition_token_name = format_ident!("{}ConditionToken", struct_name);

    let input_fields = fields
        .named
        .iter()
        .filter(|f| !crate::finder::include_unary_attr(&f.attrs, "uuid"))
        .map(|f| {
            let ident = &f.ident.clone().unwrap();
            let ty = &f.ty;
            quote! {
                pub #ident: #ty,
            }
        });

    let output_fields = fields.named.iter().map(|f| {
        let ident = &f.ident.clone().unwrap();
        let ty = &f.ty;
        quote! {
            pub #ident: #ty,
        }
    });

    let output_values = fields.named.iter().map(|f| {
        let ident = &f.ident.clone().unwrap();
        let renamed = crate::finder::find_rename_value(&f.attrs);
        let attr_key = create_renamed(ident.to_string(), renamed, rename_all_type);
        if crate::finder::include_unary_attr(&f.attrs, "uuid") {
            quote! {
                #ident: uuid_map.get(#attr_key).cloned().unwrap().into(),
            }
        } else {
            quote! {
                #ident: item.#ident,
            }
        }
    });

    let input_items = {
        let insertion = fields.named.iter().map(|f| {
            let ident = &f.ident.clone().unwrap();
            let renamed = crate::finder::find_rename_value(&f.attrs);
            let attr_key = create_renamed(ident.to_string(), renamed, rename_all_type);
            if crate::finder::include_unary_attr(&f.attrs, "uuid") {
                quote! {
                    let id = #struct_name::gen();
                    input_item.insert(
                        #attr_key.to_string(),
                        id.clone().into_attr(),
                    );
                    uuid_map.insert(
                        #attr_key.to_string(),
                        id,
                    );
                }
            } else {
                quote! {
                    let value = item.#ident.clone().into_attr();
                    if !::raiden::is_attr_value_empty(&value) {
                        input_item.insert(
                            #attr_key.to_string(),
                            value,
                        );
                    }
                }
            }
        });

        quote! {
            let mut input_item: std::collections::HashMap<String, raiden::AttributeValue> = std::collections::HashMap::new();
            #(#insertion)*
        }
    };

    // Create default type variables for PutItemBuilder, i.e. XXXPutItemBuilder<(), (), ()>
    let required_field_idents: Vec<Ident> = fields
        .named
        .iter()
        .filter(|f| !crate::finder::include_unary_attr(&f.attrs, "uuid"))
        .filter(|f| !crate::finder::is_option(&f.ty))
        .map(|f| f.ident.clone().unwrap())
        .collect();
    let default_types = expand_default_type_variables(&required_field_idents);

    let api_call_token = super::api_call_token!("put_item");
    let (call_inner_run, inner_run_args) = if cfg!(feature = "tracing") {
        (
            quote! {
                let table_name = builder
                    .get_table_name()
                    .clone()
                    .expect("table name should be set");

                #builder_name::inner_run(table_name, client, builder).await
            },
            quote! { table_name: String, },
        )
    } else {
        (
            quote! { #builder_name::inner_run(client, builder).await },
            quote! {},
        )
    };

    quote! {
        #[derive(Debug, Clone, PartialEq, ::raiden::Builder)]
        pub struct #item_input_name {
            #(#input_fields)*
        }

        impl #struct_name {
            pub fn put_item_builder() -> #item_input_builder_name<#(#default_types)*> {
                #item_input_name::builder()
            }
        }

        #[derive(Debug, Clone, PartialEq)]
        pub struct #item_output_name {
            #(#output_fields)*
        }

        pub trait #trait_name {
            fn put(&self, item: #item_input_name) -> #builder_name;
        }

        impl #trait_name for #client_name {
            fn put(&self, item: #item_input_name) -> #builder_name{
                let mut uuid_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();

                #input_items

                let output_item = #item_output_name {
                    #(#output_values)*
                };

                let builder = ::raiden::PutItemInput::builder()
                    .set_item(Some(input_item))
                    .table_name(self.table_name());

                #builder_name {
                    client: &self.client,
                    builder,
                    item: output_item,
                    policy: self.retry_condition.strategy.policy(),
                    condition: &self.retry_condition,
                }
            }
        }

        pub struct #builder_name<'a> {
            pub client: &'a ::raiden::Client,
            pub builder: ::raiden::PutItemInputBuilder,
            pub item: #item_output_name,
            pub policy: ::raiden::Policy,
            pub condition: &'a ::raiden::retry::RetryCondition,
        }

        impl<'a> #builder_name<'a> {

            pub fn raw_input(mut self, builder: ::raiden::PutItemInputBuilder) -> Self {
                self.builder = builder;
                self
            }

            pub fn condition(mut self, cond: impl ::raiden::condition::ConditionBuilder<#condition_token_name>) -> Self {
                let (cond_str, attr_names, attr_values) = cond.build();

                if !attr_names.is_empty() {
                    self.builder = self.builder
                        .set_expression_attribute_names(Some(attr_names));
                }

                if !attr_values.is_empty() {
                    self.builder = self.builder
                        .set_expression_attribute_values(Some(attr_values));
                }

                self.builder = self.builder.condition_expression(cond_str);
                self
            }

            pub async fn run(self) -> Result<::raiden::put::PutOutput<#item_output_name>, ::raiden::RaidenError> {
                let builder = self.builder.clone();
                let client = self.client.clone();
                let policy: ::raiden::RetryPolicy = self.policy.into();

                let res = policy.retry_if(move || {
                    let builder = builder.clone();
                    let client = client.clone();
                    async { #call_inner_run }
                }, self.condition).await?;

                Ok(::raiden::put::PutOutput {
                    item: self.item,
                    consumed_capacity: res.consumed_capacity,
                })
            }

            async fn inner_run(
                #inner_run_args
                client: ::raiden::Client,
                builder: ::raiden::PutItemInputBuilder,
            ) -> Result<::raiden::PutItemOutput, ::raiden::RaidenError> {
                Ok(#api_call_token?)
            }
        }
    }
}

#[allow(clippy::ptr_arg)]
fn expand_default_type_variables(idents: &Vec<Ident>) -> impl Iterator<Item = TokenStream> {
    idents.clone().into_iter().map(|_ident| {
        quote! { (), }
    })
}
