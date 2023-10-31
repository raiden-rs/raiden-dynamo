use quote::*;

pub(crate) fn expand_attr_to_item(
    item_ident: proc_macro2::Ident,
    fields: &syn::FieldsNamed,
    rename_all_type: crate::rename::RenameAllType,
) -> Vec<proc_macro2::TokenStream> {
    fields.named.iter().map(|f| {
        let ident = &f.ident.clone().unwrap();
        let use_default = crate::finder::include_unary_attr(&f.attrs, "use_default");
        let renamed = crate::finder::find_rename_value(&f.attrs);
        let attr_key  = if let Some(renamed) = renamed {
            renamed
        }  else if rename_all_type != crate::rename::RenameAllType::None {
            crate::rename::rename(rename_all_type, ident.to_string())
        } else {
            ident.to_string()
        };
        let ty = &f.ty;

        let item = quote! {
            let item = <#ty as ResolveAttribute>::resolve_attr(&#attr_key, &mut #item_ident);
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
                        #ty::default()
                    } else {
                        let item = item.unwrap();
                        // If null is true, use default value.
                        if item.is_null() {
                            #ty::default()
                        } else {
                            let converted = ::raiden::FromAttribute::from_attr(Some(item));
                            if converted.is_err() {
                            // TODO: improve error handling.
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
                        // TODO: improve error handling.
                        return Err(::raiden::RaidenError::AttributeConvertError{ attr_name: #attr_key.to_string() });
                    }
                    converted.unwrap()
                },
            }
        }
    }).collect()
}

macro_rules! api_call_token {
    ($operation: literal) => {
        $crate::ops::api_call_token!("table_name", "client", $operation, "builder")
    };
    ($table_name: literal, $client: literal, $operation: literal, $builder: literal) => {{
        let table_name = ::quote::format_ident!($table_name);
        let client = ::quote::format_ident!($client);
        let operation = ::quote::format_ident!($operation);
        let builder = ::quote::format_ident!($builder);

        let span_token = if cfg!(feature = "tracing") {
            ::quote::quote! {
                use tracing::Instrument;
                let fut = fut.instrument(::tracing::debug_span!(
                    "dynamodb::action",
                    table = #table_name,
                    api = std::stringify!(#operation),
                ));
            }
        } else {
            ::quote::quote! {}
        };

        ::quote::quote! {{
            let fut = #builder.send_with(&#client);

            #span_token

            fut.await
        }}
    }};
}

pub(super) use api_call_token;
