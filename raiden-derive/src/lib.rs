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
mod item;
mod key;
mod key_condition;
mod rename;

#[cfg(feature = "rusoto")]
mod rusoto;
#[cfg(feature = "rusoto")]
use rusoto::*;

#[cfg(feature = "aws-sdk")]
mod aws_sdk;
#[cfg(feature = "aws-sdk")]
use aws_sdk::*;

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
    let gsi_names = finder::find_gsi_names(&attrs);
    let gsi_definitions = finder::find_gsi_definitions(&attrs);

    let fields = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(n),
            ..
        }) => n,
        _ => unimplemented!(),
    };

    finder::validate_omit_gsi_fields(&fields, &gsi_names);

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

    let query = ops::expand_query(
        &struct_name,
        &fields,
        rename_all_type,
        &gsi_names,
        &gsi_definitions,
    );

    let scan = ops::expand_scan(&struct_name, &fields, rename_all_type, &gsi_names);

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
    let raiden_item = item::expand_raiden_item_impl(&struct_name, &fields, rename_all_type);

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

        #raiden_item

        impl ::raiden::IdGenerator for #struct_name {}
    };
    // Hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(RaidenIndex, attributes(raiden))]
pub fn derive_raiden_index(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    let struct_name = input.ident;
    let attrs = input.attrs;

    let source = attrs
        .iter()
        .find_map(|attr| {
            if attr.path().segments[0].ident != "raiden" {
                return None;
            }
            finder::find_eq_string_from(attr, "source")
        })
        .unwrap_or_else(|| panic!("RaidenIndex requires #[raiden(source = \"...\")]"));
    let gsi_name = attrs
        .iter()
        .find_map(|attr| {
            if attr.path().segments[0].ident != "raiden" {
                return None;
            }
            finder::find_eq_string_from(attr, "gsi")
        })
        .unwrap_or_else(|| panic!("RaidenIndex requires #[raiden(gsi = \"...\")]"));

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

    let source_ty: Type = syn::parse_str(&source)
        .unwrap_or_else(|_| panic!("invalid source type `{source}` for RaidenIndex"));
    let raiden_item = item::expand_raiden_item_impl(&struct_name, &fields, rename_all_type);

    let expanded = quote! {
        #raiden_item

        impl ::raiden::RaidenIndexItem<#source_ty> for #struct_name {
            const GSI_NAME: &'static str = #gsi_name;
        }
    };

    proc_macro::TokenStream::from(expanded)
}
