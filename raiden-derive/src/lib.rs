use proc_macro::TokenStream;
use quote::*;

use syn::*;

mod attribute;
mod condition;
mod filter_expression;
mod finder;
mod helpers;
mod key;
mod key_condition;
mod ops;
mod rename;

use crate::rename::*;
use std::str::FromStr;

#[proc_macro_derive(Raiden, attributes(raiden))]
pub fn derive_raiden(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    let struct_name = input.ident;

    let client_name = format_ident!("{}Client", struct_name);

    let attr_enum_name = format_ident!("{}AttrNames", struct_name);

    let attrs = input.attrs;

    let table_name = if let Some(name) = finder::find_table_name(&attrs) {
        name
    } else {
        struct_name.to_string()
    };

    let rename_all = finder::find_rename_all(&attrs);
    let rename_all_type = if let Some(rename_all) = rename_all {
        rename::RenameAllType::from_str(&rename_all).unwrap()
    } else {
        rename::RenameAllType::None
    };

    let fields = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(n),
            ..
        }) => n,
        _ => unimplemented!(),
    };

    let partition_key = key::fetch_partition_key(&fields, rename_all_type);
    let sort_key = key::fetch_sort_key(&fields, rename_all_type);

    let table_name_field = format_ident!("table_name");
    let client_field = format_ident!("client");
    let n = vec![
        quote! { #table_name_field: &'static str },
        quote! { #client_field: ::raiden::DynamoDbClient },
    ];

    // let struct_fields = fields.named.iter().map(|f| {
    //     let ident = &f.ident.clone().unwrap();
    //     let name = ident_case::RenameRule::PascalCase.apply_to_field(ident.to_string());
    //     let name = format_ident!("{}", name);
    //     quote! {
    //       #name
    //     }
    // });

    let get_item = ops::expand_get_item(
        &partition_key,
        &sort_key,
        &struct_name,
        &fields,
        rename_all_type,
    );

    let query = ops::expand_query(&struct_name, &fields, rename_all_type);

    let scan = ops::expand_scan(&struct_name, &fields, rename_all_type);

    let batch_get = ops::expand_batch_get(
        &partition_key,
        &sort_key,
        &struct_name,
        &fields,
        rename_all_type,
    );

    let put_item = ops::expand_put_item(&struct_name, &fields, rename_all_type);

    let update_item = ops::expand_update_item(
        &partition_key,
        &sort_key,
        &fields,
        &attr_enum_name,
        &struct_name,
        rename_all_type,
    );

    let delete_item = ops::expand_delete_item(&partition_key, &sort_key, &struct_name);

    let batch_delete = ops::expand_batch_delete(&partition_key, &sort_key, &struct_name);

    let attr_names =
        attribute::expand_attr_names(&attr_enum_name, &fields, rename_all_type, &struct_name);

    let condition_builder =
        condition::expand_condition_builder(&attr_enum_name, &struct_name, &fields);

    let key_condition_builder =
        key_condition::expand_key_condition_builder(&attr_enum_name, &struct_name);

    let filter_expression_builder =
        filter_expression::expand_filter_expression_builder(&attr_enum_name, &struct_name);

    let transact_write = ops::expand_transact_write(
        &struct_name,
        &partition_key,
        &sort_key,
        &fields,
        &attr_enum_name,
        rename_all_type,
        &table_name,
    );

    let insertion_attribute_name = fields.named.iter().map(|f| {
        let ident = &f.ident.clone().unwrap();
        let renamed = crate::finder::find_rename_value(&f.attrs);
        let result = create_renamed(ident.to_string(), renamed, rename_all_type);
        quote! {
            names.insert(
                format!("#{}", #result.clone()),
                #result.to_string(),
            );
        }
    });

    let expanded = quote! {
        use ::raiden::IntoAttribute as _;
        use ::raiden::IntoAttrName as _;
        use ::raiden::DynamoDb as _;

        pub struct #client_name {
            #(
                #n,
            )*
            table_prefix: String,
            table_suffix: String,
            retry_condition: ::raiden::RetryCondition,
            attribute_names: Option<::raiden::AttributeNames>,
            projection_expression: Option<String>
        }

        #attr_names

        #condition_builder

        #key_condition_builder

        #filter_expression_builder

        #get_item

        #batch_get

        #query

        #scan

        #put_item

        #update_item

        #delete_item

        #batch_delete

        #transact_write

        impl #client_name {
            pub fn new(region: ::raiden::Region) -> Self {
                let client = ::raiden::DynamoDbClient::new(region);
                let names = {
                    let mut names: ::raiden::AttributeNames = std::collections::HashMap::new();
                    #(#insertion_attribute_name)*
                    names
                };
                let projection_expression = Some(names.keys().map(|v| v.to_string()).collect::<Vec<String>>().join(", "));

                Self {
                    table_name: #table_name,
                    table_prefix: "".to_owned(),
                    table_suffix: "".to_owned(),
                    client,
                    retry_condition: ::raiden::RetryCondition::new(),
                    attribute_names: Some(names),
                    projection_expression
                }
            }

            pub fn with_retries(mut self, s: Box<dyn ::raiden::retry::RetryStrategy + Send + Sync>) -> Self {
                self.retry_condition.strategy = s;
                self
            }

            pub fn table_prefix(mut self, prefix: impl Into<String>) -> Self {
                self.table_prefix = prefix.into();
                self
            }

            pub fn table_suffix(mut self, suffix: impl Into<String>) -> Self {
                self.table_suffix = suffix.into();
                self
            }

            pub fn table_name(&self) -> String {
                format!("{}{}{}", self.table_prefix, self.table_name.to_string(), self.table_suffix)
            }
        }

        impl #struct_name {
            pub fn client(region: ::raiden::Region) -> #client_name {
                #client_name::new(region)
            }
        }

        impl ::raiden::IdGenerator for #struct_name {}
    };
    // Hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)
}

// fn fetch_raiden_field(fields: &syn::FieldsNamed) -> Vec<syn::Field> {
//     let fields: Vec<syn::Field> = fields
//         .named
//         .iter()
//         .cloned()
//         .filter(|f| {
//             f.attrs.len() > 0
//                 && f.attrs
//                     .iter()
//                     .any(|attr| attr.path.segments[0].ident == "raiden")
//         })
//         .collect();
//     dbg!(&fields.len());
//     fields
// }

// fn check_attr_of(
//     name: &str,
//     tokens: &mut proc_macro2::token_stream::IntoIter,
// ) -> Option<proc_macro2::token_stream::IntoIter> {
//     dbg!(&name);
//     let mut tokens = match tokens.next() {
//         Some(proc_macro2::TokenTree::Group(g)) => g.stream().into_iter(),
//         _ => return None,
//     };
//     dbg!(&name);
//
//     match tokens.next() {
//         Some(proc_macro2::TokenTree::Ident(ref ident)) if *ident == name => {
//             return Some(tokens);
//         }
//         _ => return None,
//     };
// }
