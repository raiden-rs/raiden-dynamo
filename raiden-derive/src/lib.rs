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

fn create_auto_gsi_projection_name(
    source_struct_name: &proc_macro2::Ident,
    gsi_name: &str,
) -> proc_macro2::Ident {
    let source_name = source_struct_name.to_string();
    let gsi_pascal = gsi_name.to_case(Case::Pascal);
    let item_name = if gsi_pascal.starts_with(&source_name) {
        format!("{gsi_pascal}Item")
    } else {
        format!("{source_name}{gsi_pascal}Item")
    };
    format_ident!("{}", item_name)
}

fn resolve_attr_name_for_fields(
    fields: &[syn::Field],
    rename_all_type: crate::rename::RenameAllType,
    field_name: &str,
) -> String {
    let field = fields
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
    fields: &[syn::Field],
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
            /// Starts a typed key condition with the partition key of this GSI.
            ///
            /// This is the required first condition when querying the
            /// associated index.
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
                /// Starts a typed key condition for the final sort key of this GSI.
                ///
                /// This helper can only be chained after the partition key
                /// condition has already been specified.
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
                    /// Starts a typed key condition for one sort-key segment of this GSI.
                    ///
                    /// Sort-key helpers must be chained in declaration order and
                    /// can only be used after the partition key condition.
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

#[allow(clippy::too_many_arguments)]
fn expand_projection_item_support(
    struct_name: &proc_macro2::Ident,
    source_ty: &syn::Type,
    source_struct_ident: &proc_macro2::Ident,
    item_fields: &[syn::Field],
    key_condition_fields: &[syn::Field],
    rename_all_type: crate::rename::RenameAllType,
    gsi_name: &str,
    gsi_definitions: &[crate::finder::GsiDefinition],
) -> proc_macro2::TokenStream {
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
        create_gsi_partition_token_name(source_struct_ident, gsi_name)
    } else {
        source_key_condition_token_ident.clone()
    };

    let raiden_item =
        item::expand_raiden_item_impl_for_fields(struct_name, item_fields, rename_all_type);
    let gsi_key_condition_methods = expand_gsi_key_condition_methods_for_owner(
        struct_name,
        source_struct_ident,
        key_condition_fields,
        rename_all_type,
        gsi_definitions,
    );

    let index_entrypoints = if cfg!(feature = "aws-sdk") {
        quote! {
            impl #struct_name {
                /// Starts a typed query for this projection type.
                ///
                /// The returned builder is already bound to the associated GSI
                /// and decodes results into this projection type.
                ///
                /// This is equivalent to starting from the source builder and
                /// then calling `.project::<Self>()`.
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
                ///
                /// This is equivalent to starting from the source builder and
                /// then calling `.project::<Self>()`.
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
                ///
                /// This is equivalent to starting from the source builder and
                /// then calling `.project::<Self>()`.
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
                ///
                /// This is equivalent to starting from the source builder and
                /// then calling `.project::<Self>()`.
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

    quote! {
        #raiden_item

        impl ::raiden::RaidenIndexItem<#source_ty> for #struct_name {
            const GSI_NAME: &'static str = #gsi_name;
        }

        #index_entrypoints

        #gsi_key_condition_methods
    }
}

fn expand_auto_gsi_projection_items(
    source_struct_ident: &proc_macro2::Ident,
    source_vis: &syn::Visibility,
    source_fields: &[syn::Field],
    rename_all_type: crate::rename::RenameAllType,
    gsi_names: &[String],
    gsi_definitions: &[crate::finder::GsiDefinition],
) -> proc_macro2::TokenStream {
    let source_ty: syn::Type = syn::parse_quote!(#source_struct_ident);

    let auto_items = gsi_names.iter().filter_map(|gsi_name| {
        let projection_fields: Vec<syn::Field> = source_fields
            .iter()
            .filter(|field| {
                !crate::finder::find_omit_gsi_names(&field.attrs)
                    .iter()
                    .any(|name| name == gsi_name)
            })
            .cloned()
            .collect();

        if projection_fields.len() == source_fields.len() {
            return None;
        }

        let matching_definitions: Vec<crate::finder::GsiDefinition> = gsi_definitions
            .iter()
            .filter(|gsi| &gsi.name == gsi_name)
            .cloned()
            .collect();
        let projection_name = create_auto_gsi_projection_name(source_struct_ident, gsi_name);
        let struct_fields = projection_fields.iter().map(|field| {
            let vis = &field.vis;
            let ident = field
                .ident
                .as_ref()
                .expect("raiden only supports named fields");
            let ty = &field.ty;
            quote! {
                #vis #ident: #ty,
            }
        });
        let support = expand_projection_item_support(
            &projection_name,
            &source_ty,
            source_struct_ident,
            &projection_fields,
            source_fields,
            rename_all_type,
            gsi_name,
            &matching_definitions,
        );

        Some(quote! {
            /// An automatically generated GSI projection item.
            ///
            /// This type is emitted when the source model uses `omit_gsi` for
            /// the associated index, and it contains every source field that is
            /// not omitted for that GSI.
            ///
            /// Use this type with either `source.query().index_name().project::<Type>()`
            /// or `Type::query(&source_client)` depending on which style reads
            /// better at the call site.
            #[allow(dead_code)]
            #source_vis struct #projection_name {
                #(#struct_fields)*
            }

            #support
        })
    });

    quote! {
        #(#auto_items)*
    }
}

/// Derives the main table model, typed builders, and query helpers.
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
    let struct_vis = input.vis.clone();

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
    let source_fields: Vec<syn::Field> = fields.named.iter().cloned().collect();
    let auto_gsi_projection_items = expand_auto_gsi_projection_items(
        &struct_name,
        &struct_vis,
        &source_fields,
        rename_all_type,
        &gsi_names,
        &gsi_definitions,
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

        #raiden_item

        #auto_gsi_projection_items

        impl ::raiden::IdGenerator for #struct_name {}
    };
    // Hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)
}

/// Derives a typed projection model for a secondary index.
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
    let projection_fields: Vec<syn::Field> = fields.named.iter().cloned().collect();
    let expanded = expand_projection_item_support(
        &struct_name,
        &source_ty,
        &source_struct_ident,
        &projection_fields,
        &projection_fields,
        rename_all_type,
        &gsi_name,
        &gsi_definitions,
    );

    proc_macro::TokenStream::from(expanded)
}

/// Derives DynamoDB document conversion support for a nested type.
///
/// The generated implementation marks the type as a document with
/// `IntoDocumentAttr` and `FromDocumentAttr`, and also provides
/// `IntoAttribute` / `FromAttribute` so the type can be used directly as a
/// field in `#[derive(Raiden)]` models without wrapping it in `Document<T>`.
#[proc_macro_derive(RaidenDocument)]
pub fn derive_raiden_document(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;
    let fields = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(n),
            ..
        }) => n,
        _ => unimplemented!(),
    };
    let attr_enum_name = format_ident!("{}DocumentAttrNames", struct_name);

    let names = fields.named.iter().map(|field| {
        let ident = field.ident.as_ref().expect("named field");
        let name = ident.to_string().to_case(Case::Pascal);
        let name = format_ident!("{}", name);
        quote! { #name }
    });

    let arms = fields.named.iter().map(|field| {
        let ident = field.ident.as_ref().expect("named field");
        let name = ident.to_string().to_case(Case::Pascal);
        let name = format_ident!("{}", name);
        let renamed = find_serde_rename(&field.attrs).unwrap_or_else(|| ident.to_string());
        quote! {
            #attr_enum_name::#name => #renamed.to_owned()
        }
    });

    let getters = fields.named.iter().map(|field| {
        let ident = field.ident.as_ref().expect("named field");
        let func_name = format_ident!("{}", ident);
        let name = ident.to_string().to_case(Case::Pascal);
        let name = format_ident!("{}", name);
        quote! {
            pub fn #func_name() -> #attr_enum_name {
                #attr_enum_name::#name
            }
        }
    });

    let expanded = quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum #attr_enum_name {
            #(
                #names,
            )*
        }

        impl ::raiden::IntoAttrName for #attr_enum_name {
            fn into_attr_name(self) -> String {
                match self {
                    #(
                        #arms,
                    )*
                }
            }
        }

        impl #struct_name {
            #(
                #getters
            )*
        }

        impl ::raiden::IntoDocumentAttr for #struct_name {}

        impl ::raiden::FromDocumentAttr for #struct_name {}

        impl ::raiden::IntoAttribute for #struct_name {
            fn into_attr(self) -> ::raiden::AttributeValue {
                <Self as ::raiden::IntoDocumentAttr>::into_document_attr(self)
                    .expect(concat!(
                        "RaidenDocument serialization failed for `",
                        stringify!(#struct_name),
                        "`."
                    ))
            }
        }

        impl ::raiden::FromAttribute for #struct_name {
            fn from_attr(
                value: Option<::raiden::AttributeValue>,
            ) -> Result<Self, ::raiden::ConversionError> {
                <Self as ::raiden::FromDocumentAttr>::from_document_attr(value)
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn find_serde_rename(attrs: &[syn::Attribute]) -> Option<String> {
    let mut renamed = None;

    for attr in attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }

        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                let value = meta.value()?;
                let value: syn::LitStr = value.parse()?;
                renamed = Some(value.value());
            }
            Ok(())
        });
    }

    renamed
}
