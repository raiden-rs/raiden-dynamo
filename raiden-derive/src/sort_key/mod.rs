use quote::*;

pub fn fetch_sort_key(
    fields: &syn::FieldsNamed,
    rename_all_type: crate::rename::RenameAllType,
) -> Option<proc_macro2::Ident> {
    match crate::finder::find_sort_key_field(&fields) {
        Some(key) => {
            // Rename partition key if renamed.
            let renamed = crate::finder::find_rename_value(&key.attrs);
            if renamed.is_some() {
                Some(format_ident!("{}", renamed.unwrap()))
            } else if rename_all_type != crate::rename::RenameAllType::None {
                Some(format_ident!(
                    "{}",
                    crate::rename::rename(rename_all_type, key.ident.unwrap().to_string())
                ))
            } else {
                Some(key.ident.unwrap())
            }
        }
        None => None,
    }
}
