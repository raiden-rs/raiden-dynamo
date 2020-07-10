use quote::*;

pub(crate) fn expand_attr_to_item(
    item_ident: &proc_macro2::Ident,
    fields: &syn::FieldsNamed,
    rename_all_type: crate::rename::RenameAllType,
) -> Vec<proc_macro2::TokenStream> {
    fields.named.iter().map(|f| {
        let ident = &f.ident.clone().unwrap();
        let renamed = crate::finder::find_rename_value(&f.attrs);
        let attr_key  = if let Some(renamed) = renamed {
            renamed
        }  else if rename_all_type != crate::rename::RenameAllType::None {
            crate::rename::rename(rename_all_type, ident.to_string())
        } else {
            ident.to_string()
        };
        if crate::is_option(&f.ty) {
            quote! {
              #ident: {
                let item = #item_ident.get(#attr_key);
                if item.is_none() {
                    None
                } else {
                    let converted = ::raiden::FromAttribute::from_attr(item.cloned());
                    if converted.is_err() {
                        return Err(::raiden::RaidenError::AttributeConvertError{ attr_name: #attr_key.to_string() });
                    }
                    converted.unwrap()
                }
              },
            }
        } else {
            quote! {
              #ident: {
                let item = #item_ident.get(#attr_key);
                let converted = ::raiden::FromAttribute::from_attr(item.cloned());
                if converted.is_err() {
                  // TODO: improve error handling.
                    return Err(::raiden::RaidenError::AttributeConvertError{ attr_name: #attr_key.to_string() });
                }
                converted.unwrap()
              },
            }
        }
    }).collect()
}
