use proc_macro::TokenStream;
use quote::*;

use syn::*;

mod attribute;
mod condition;
mod finder;
mod key_condition;
mod ops;
mod rename;
mod sort_key;

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

    let partition_key = match finder::find_partition_key_field(&fields) {
        Some(key) => {
            // Rename partition key if renamed.
            let renamed = finder::find_rename_value(&key.attrs);
            if renamed.is_some() {
                format_ident!("{}", renamed.unwrap())
            } else if rename_all_type != rename::RenameAllType::None {
                format_ident!(
                    "{}",
                    rename::rename(rename_all_type, key.ident.unwrap().to_string())
                )
            } else {
                key.ident.unwrap()
            }
        }
        None => panic!("Please specify partition key"),
    };

    let sort_key = sort_key::fetch_sort_key(&fields, rename_all_type);

    let table_name_field = format_ident!("table_name");
    let client_field = format_ident!("client");
    let n = vec![
        quote! { #table_name_field: &'static str },
        quote! { #client_field: DynamoDbClient },
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

    let put_item = ops::expand_put_item(&partition_key, &struct_name, &fields, rename_all_type);

    let update_item = ops::expand_update_item(
        &partition_key,
        &sort_key,
        &fields,
        &attr_enum_name,
        &struct_name,
        rename_all_type,
    );

    let delete_item = ops::expand_delete_item(&partition_key, &sort_key, &struct_name);

    let attr_names =
        attribute::expand_attr_names(&attr_enum_name, &fields, rename_all_type, &struct_name);

    let condition_builder =
        condition::expand_condition_builder(&attr_enum_name, &struct_name, &fields);

    let key_condition_builder =
        key_condition::expand_key_condition_builder(&attr_enum_name, &struct_name);

    let transact_write = ops::expand_transact_write(
        &struct_name,
        &partition_key,
        &sort_key,
        &fields,
        &attr_enum_name,
        rename_all_type,
        &table_name,
    );

    let expanded = quote! {

        pub struct #client_name {
            #(
                #n,
            )*
            table_prefix: String,
            table_suffix: String,
        }

        #attr_names

        #condition_builder

        #key_condition_builder

        #get_item

        #batch_get

        #query

        #scan

        #put_item

        #update_item

        #delete_item

        #transact_write

        impl #client_name {
            pub fn new(region: ::raiden::Region) -> Self {
                let client = DynamoDbClient::new(region);
                Self {
                    table_name: #table_name,
                    table_prefix: "".to_owned(),
                    table_suffix: "".to_owned(),
                    client,
                }
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
