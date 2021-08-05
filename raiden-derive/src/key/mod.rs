use proc_macro2::*;
use quote::*;
use syn::*;

use crate::finder::*;
use crate::rename::{rename, RenameAllType};

pub fn fetch_partition_key(
    fields: &syn::FieldsNamed,
    rename_all_type: crate::rename::RenameAllType,
) -> (Ident, Type) {
    match find_partition_key_field(fields) {
        Some(key) => {
            // Rename partition key if renamed.
            let renamed = find_rename_value(&key.attrs);
            if renamed.is_some() {
                (format_ident!("{}", renamed.unwrap()), key.ty)
            } else if rename_all_type != RenameAllType::None {
                let ident = format_ident!(
                    "{}",
                    rename(rename_all_type, key.ident.unwrap().to_string())
                );
                (ident, key.ty)
            } else {
                (key.ident.unwrap(), key.ty)
            }
        }
        None => panic!("Please specify partition key"),
    }
}

pub fn fetch_sort_key(
    fields: &syn::FieldsNamed,
    rename_all_type: crate::rename::RenameAllType,
) -> Option<(Ident, Type)> {
    match find_sort_key_field(fields) {
        Some(key) => {
            // Rename partition key if renamed.
            let renamed = find_rename_value(&key.attrs);
            if renamed.is_some() {
                Some((format_ident!("{}", renamed.unwrap()), key.ty))
            } else if rename_all_type != RenameAllType::None {
                let ident = format_ident!(
                    "{}",
                    rename(rename_all_type, key.ident.unwrap().to_string())
                );
                Some((ident, key.ty))
            } else {
                Some((key.ident.unwrap(), key.ty))
            }
        }
        None => None,
    }
}
