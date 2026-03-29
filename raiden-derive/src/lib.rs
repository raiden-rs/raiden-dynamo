#[cfg(all(feature = "aws-sdk", feature = "rusoto"))]
compile_error!("feature \"aws-sdk\" and \"rusoto\" cannot be enabled at the same time.");

use convert_case::{Case, Casing};
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

fn create_gsi_partition_token_name(
    struct_name: &proc_macro2::Ident,
    index_name: &str,
) -> proc_macro2::Ident {
    format_ident!(
        "{}{}PartitionKeyConditionToken",
        struct_name,
        index_name.to_case(Case::Pascal)
    )
}

fn create_gsi_sort_token_name(
    struct_name: &proc_macro2::Ident,
    index_name: &str,
    index: usize,
) -> proc_macro2::Ident {
    format_ident!(
        "{}{}Sort{}KeyConditionToken",
        struct_name,
        index_name.to_case(Case::Pascal),
        index + 1
    )
}

fn create_gsi_terminal_token_name(
    struct_name: &proc_macro2::Ident,
    index_name: &str,
) -> proc_macro2::Ident {
    format_ident!(
        "{}{}TerminalKeyConditionToken",
        struct_name,
        index_name.to_case(Case::Pascal)
    )
}

fn resolve_attr_name_for_fields(
    fields: &syn::FieldsNamed,
    rename_all_type: crate::rename::RenameAllType,
    field_name: &str,
) -> String {
    let field = fields
        .named
        .iter()
        .find(|field| {
            field
                .ident
                .as_ref()
                .is_some_and(|ident| ident == field_name)
        })
        .unwrap_or_else(|| panic!("unknown field `{field_name}` for gsi key definition"));

    crate::rename::create_renamed(
        field_name.to_owned(),
        crate::finder::find_rename_value(&field.attrs),
        rename_all_type,
    )
}

fn expand_gsi_key_condition_methods_for_owner(
    method_owner_name: &proc_macro2::Ident,
    token_owner_name: &proc_macro2::Ident,
    fields: &syn::FieldsNamed,
    rename_all_type: crate::rename::RenameAllType,
    gsi_definitions: &[crate::finder::GsiDefinition],
) -> proc_macro2::TokenStream {
    let gsi_key_condition_methods = gsi_definitions.iter().flat_map(|gsi| {
        let mut methods = vec![];
        let Some(partition_key) = gsi.partition_key.as_ref() else {
            return methods;
        };

        let partition_token_name = create_gsi_partition_token_name(token_owner_name, &gsi.name);
        let terminal_token_name = create_gsi_terminal_token_name(token_owner_name, &gsi.name);
        let partition_next_token_name = if gsi.sort_keys.is_empty() {
            terminal_token_name.clone()
        } else {
            create_gsi_sort_token_name(token_owner_name, &gsi.name, 0)
        };

        let method_name = format!("{}_key_condition", gsi.name.to_case(Case::Snake));
        let method_ident = if crate::helpers::is_reserved(&method_name) {
            format_ident!("r#{}", method_name)
        } else {
            format_ident!("{}", method_name)
        };
        let attr_name = resolve_attr_name_for_fields(fields, rename_all_type, partition_key);

        methods.push(quote! {
            pub fn #method_ident() -> ::raiden::KeyCondition<#partition_token_name, #partition_next_token_name> {
                ::raiden::KeyCondition {
                    attr: #attr_name.to_owned(),
                    _token: std::marker::PhantomData,
                    _next_token: std::marker::PhantomData,
                }
            }
        });

        if gsi.sort_keys.len() == 1 {
            let sort_key = &gsi.sort_keys[0];
            let sort_token_name = create_gsi_sort_token_name(token_owner_name, &gsi.name, 0);
            let method_name = format!("{}_sort_key_condition", gsi.name.to_case(Case::Snake));
            let method_ident = if crate::helpers::is_reserved(&method_name) {
                format_ident!("r#{}", method_name)
            } else {
                format_ident!("{}", method_name)
            };
            let attr_name = resolve_attr_name_for_fields(fields, rename_all_type, sort_key);

            methods.push(quote! {
                pub fn #method_ident() -> ::raiden::KeyCondition<#sort_token_name, #terminal_token_name> {
                    ::raiden::KeyCondition {
                        attr: #attr_name.to_owned(),
                        _token: std::marker::PhantomData,
                        _next_token: std::marker::PhantomData,
                    }
                }
            });
        } else {
            for (index, sort_key) in gsi.sort_keys.iter().enumerate() {
                let sort_token_name = create_gsi_sort_token_name(token_owner_name, &gsi.name, index);
                let next_token_name = if index + 1 < gsi.sort_keys.len() {
                    create_gsi_sort_token_name(token_owner_name, &gsi.name, index + 1)
                } else {
                    terminal_token_name.clone()
                };
                let method_name = format!(
                    "{}_sort_key_condition_{}",
                    gsi.name.to_case(Case::Snake),
                    index + 1
                );
                let method_ident = if crate::helpers::is_reserved(&method_name) {
                    format_ident!("r#{}", method_name)
                } else {
                    format_ident!("{}", method_name)
                };
                let attr_name = resolve_attr_name_for_fields(fields, rename_all_type, sort_key);

                methods.push(quote! {
                    pub fn #method_ident() -> ::raiden::KeyCondition<#sort_token_name, #next_token_name> {
                        ::raiden::KeyCondition {
                            attr: #attr_name.to_owned(),
                            _token: std::marker::PhantomData,
                            _next_token: std::marker::PhantomData,
                        }
                    }
                });
            }
        }

        methods
    });

    quote! {
        impl #method_owner_name {
            #(#gsi_key_condition_methods)*
        }
    }
}

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
    let gsi_names = finder::find_gsi_names(&attrs);
    if gsi_names.is_empty() {
        panic!("RaidenIndex requires #[raiden(gsi = \"...\")]");
    }
    if gsi_names.len() > 1 {
        panic!("RaidenIndex currently supports exactly one gsi");
    }
    let gsi_name = gsi_names[0].clone();
    let gsi_definitions = finder::find_gsi_definitions(&attrs);

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
    let source_struct_ident = match &source_ty {
        Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .expect("source path should not be empty")
            .ident
            .clone(),
        _ => panic!("RaidenIndex source must be a path type"),
    };
    let source_client_ident = format_ident!("{}Client", source_struct_ident);
    let source_query_trait_ident = format_ident!("{}Query", source_struct_ident);
    let source_scan_trait_ident = format_ident!("{}Scan", source_struct_ident);
    let source_query_builder_ident = format_ident!("{}QueryBuilder", source_struct_ident);
    let source_scan_builder_ident = format_ident!("{}ScanBuilder", source_struct_ident);
    let source_projected_query_builder_ident =
        format_ident!("{}ProjectedQueryBuilder", source_struct_ident);
    let source_projected_scan_builder_ident =
        format_ident!("{}ProjectedScanBuilder", source_struct_ident);
    let source_key_condition_token_ident =
        format_ident!("{}KeyConditionToken", source_struct_ident);
    let query_token_ident = if gsi_definitions
        .iter()
        .any(|gsi| gsi.name == gsi_name && gsi.partition_key.is_some())
    {
        create_gsi_partition_token_name(&source_struct_ident, &gsi_name)
    } else {
        source_key_condition_token_ident.clone()
    };
    let raiden_item = item::expand_raiden_item_impl(&struct_name, &fields, rename_all_type);
    let gsi_key_condition_methods = expand_gsi_key_condition_methods_for_owner(
        &struct_name,
        &source_struct_ident,
        &fields,
        rename_all_type,
        &gsi_definitions,
    );
    let index_entrypoints = if cfg!(feature = "aws-sdk") {
        quote! {
            impl #struct_name {
                /// Starts a typed query for this projection type.
                ///
                /// The returned builder is already bound to the associated GSI
                /// and decodes results into this projection type.
                pub fn query<'a>(
                    client: &'a #source_client_ident,
                ) -> #source_projected_query_builder_ident<'a, #query_token_ident, Self> {
                    let builder = <#source_client_ident as #source_query_trait_ident>::query(client);
                    let #source_query_builder_ident {
                        client,
                        builder,
                        next_token,
                        limit,
                        policy,
                        condition,
                        ..
                    } = builder;
                    #source_query_builder_ident {
                        client,
                        builder: builder.index_name(#gsi_name),
                        next_token,
                        limit,
                        policy,
                        condition,
                        _token: std::marker::PhantomData::<fn() -> #query_token_ident>,
                    }
                    .project::<Self>()
                }

                /// Starts a typed scan for this projection type.
                ///
                /// The returned builder is already bound to the associated GSI
                /// and decodes results into this projection type.
                pub fn scan<'a>(
                    client: &'a #source_client_ident,
                ) -> #source_projected_scan_builder_ident<'a, Self> {
                    let builder = <#source_client_ident as #source_scan_trait_ident>::scan(client);
                    let #source_scan_builder_ident {
                        client,
                        builder,
                        next_token,
                        limit,
                    } = builder;
                    #source_scan_builder_ident {
                        client,
                        builder: builder.index_name(#gsi_name),
                        next_token,
                        limit,
                    }
                    .project::<Self>()
                }
            }
        }
    } else {
        quote! {
            impl #struct_name {
                /// Starts a typed query for this projection type.
                ///
                /// The returned builder is already bound to the associated GSI
                /// and decodes results into this projection type.
                pub fn query<'a>(
                    client: &'a #source_client_ident,
                ) -> #source_projected_query_builder_ident<'a, #query_token_ident, Self> {
                    let builder = <#source_client_ident as #source_query_trait_ident>::query(client);
                    let #source_query_builder_ident {
                        client,
                        mut input,
                        next_token,
                        limit,
                        policy,
                        condition,
                        ..
                    } = builder;
                    input.index_name = Some(#gsi_name.to_owned());
                    #source_query_builder_ident {
                        client,
                        input,
                        next_token,
                        limit,
                        policy,
                        condition,
                        _token: std::marker::PhantomData::<fn() -> #query_token_ident>,
                    }
                    .project::<Self>()
                }

                /// Starts a typed scan for this projection type.
                ///
                /// The returned builder is already bound to the associated GSI
                /// and decodes results into this projection type.
                pub fn scan<'a>(
                    client: &'a #source_client_ident,
                ) -> #source_projected_scan_builder_ident<'a, Self> {
                    let builder = <#source_client_ident as #source_scan_trait_ident>::scan(client);
                    let #source_scan_builder_ident {
                        client,
                        mut input,
                        policy,
                        condition,
                        next_token,
                        limit,
                    } = builder;
                    input.index_name = Some(#gsi_name.to_owned());
                    #source_scan_builder_ident {
                        client,
                        input,
                        policy,
                        condition,
                        next_token,
                        limit,
                    }
                    .project::<Self>()
                }
            }
        }
    };

    let expanded = quote! {
        #raiden_item

        impl ::raiden::RaidenIndexItem<#source_ty> for #struct_name {
            const GSI_NAME: &'static str = #gsi_name;
        }

        #index_entrypoints

        #gsi_key_condition_methods
    };

    proc_macro::TokenStream::from(expanded)
}
