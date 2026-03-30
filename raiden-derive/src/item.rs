use quote::*;

fn expand_raiden_item_impl_from_fields(
    struct_name: &proc_macro2::Ident,
    fields: &[syn::Field],
    rename_all_type: crate::rename::RenameAllType,
) -> proc_macro2::TokenStream {
    let insertion_attribute_name = fields.iter().map(|f| {
        let ident = f.ident.as_ref().expect("raiden only supports named fields");
        let renamed = crate::finder::find_rename_value(&f.attrs);
        let result = crate::rename::create_renamed(ident.to_string(), renamed, rename_all_type);
        quote! {
            names.insert(
                format!("#{}", #result.clone()),
                #result.to_string(),
            );
        }
    });

    let from_item = fields.iter().map(|f| {
        let ident = &f.ident.clone().unwrap();
        let use_default = crate::finder::include_unary_attr(&f.attrs, "use_default");
        let renamed = crate::finder::find_rename_value(&f.attrs);
        let attr_key = if let Some(renamed) = renamed {
            renamed
        } else if rename_all_type != crate::rename::RenameAllType::None {
            crate::rename::rename(rename_all_type, ident.to_string())
        } else {
            ident.to_string()
        };
        let ty = &f.ty;

        let item = quote! {
            let item = <#ty as ::raiden::ResolveAttribute>::resolve_attr(&#attr_key, &mut item);
        };

        if crate::finder::is_option(ty) {
            quote! {
                #ident: {
                    #item
                    if item.is_none() {
                        None
                    } else {
                        let converted = ::raiden::FromAttribute::from_attr(item);
                        if converted.is_err() {
                            return Err(::raiden::RaidenError::AttributeConvertError{ attr_name: #attr_key.to_string() });
                        }
                        converted.unwrap()
                    }
                },
            }
        } else if use_default {
            quote! {
                #ident: {
                    #item
                    if item.is_none() {
                        Default::default()
                    } else {
                        let item = item.unwrap();
                        #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
                        let is_null = Some(true) == item.null;
                        #[cfg(feature = "aws-sdk")]
                        let is_null = item.is_null();

                        if is_null {
                            Default::default()
                        } else {
                            let converted = ::raiden::FromAttribute::from_attr(Some(item));
                            if converted.is_err() {
                                return Err(::raiden::RaidenError::AttributeConvertError{ attr_name: #attr_key.to_string() });
                            }
                            converted.unwrap()
                        }
                    }
                },
            }
        } else {
            quote! {
                #ident: {
                    #item
                    let converted = ::raiden::FromAttribute::from_attr(item);
                    if converted.is_err() {
                        return Err(::raiden::RaidenError::AttributeConvertError{ attr_name: #attr_key.to_string() });
                    }
                    converted.unwrap()
                },
            }
        }
    });

    quote! {
        impl ::raiden::RaidenItem for #struct_name {
            fn attribute_names() -> Option<::raiden::AttributeNames> {
                let mut names: ::raiden::AttributeNames = std::collections::HashMap::new();
                #(#insertion_attribute_name)*

                if names.is_empty() {
                    None
                } else {
                    Some(names)
                }
            }

            fn projection_expression() -> Option<String> {
                Self::attribute_names()
                    .map(|names| names.keys().cloned().collect::<Vec<String>>().join(", "))
            }

            fn from_item(mut item: ::raiden::AttributeValues) -> Result<Self, ::raiden::RaidenError> {
                Ok(Self {
                    #(#from_item)*
                })
            }
        }
    }
}

pub(crate) fn expand_raiden_item_impl(
    struct_name: &proc_macro2::Ident,
    fields: &syn::FieldsNamed,
    rename_all_type: crate::rename::RenameAllType,
) -> proc_macro2::TokenStream {
    let fields: Vec<syn::Field> = fields.named.iter().cloned().collect();
    expand_raiden_item_impl_from_fields(struct_name, &fields, rename_all_type)
}

pub(crate) fn expand_raiden_item_impl_for_fields(
    struct_name: &proc_macro2::Ident,
    fields: &[syn::Field],
    rename_all_type: crate::rename::RenameAllType,
) -> proc_macro2::TokenStream {
    expand_raiden_item_impl_from_fields(struct_name, fields, rename_all_type)
}
