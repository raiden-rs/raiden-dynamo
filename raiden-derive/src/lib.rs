#[cfg(all(feature = "aws-sdk", feature = "rusoto"))]
compile_error!("feature \"aws-sdk\" and \"rusoto\" cannot be enabled at the same time.");

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
mod rename;

#[cfg(feature = "rusoto")]
mod rusoto;
#[cfg(feature = "rusoto")]
use rusoto::{client, ops};

#[cfg(feature = "aws-sdk")]
mod aws_sdk;
#[cfg(feature = "aws-sdk")]
use aws_sdk::{client, ops};

use crate::rename::*;
use std::str::FromStr;

#[proc_macro_derive(Raiden, attributes(raiden))]
pub fn derive_raiden(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    let (dynamodb_client_name, use_dynamodb_trait) = if cfg!(feature = "rusoto") {
        (
            format_ident!("DynamoDbClient"),
            Some(quote! { use ::raiden::DynamoDb as _; }),
        )
    } else if cfg!(feature = "aws-sdk") {
        (format_ident!("Client"), None)
    } else {
        unreachable!();
    };

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
        quote! { #client_field: ::raiden::#dynamodb_client_name },
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

    let client_constructor = client::expand_client_constructor(
        &struct_name,
        &client_name,
        &dynamodb_client_name,
        &table_name,
        &fields,
        &rename_all_type,
    );

    let expanded = quote! {
        use ::raiden::IntoAttribute as _;
        use ::raiden::IntoAttrName as _;
        #use_dynamodb_trait

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

        #client_constructor

        impl ::raiden::IdGenerator for #struct_name {}
    };
    // Hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)
}
