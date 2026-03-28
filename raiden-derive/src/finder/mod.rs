use syn::{punctuated::Punctuated, Expr, ExprLit, Lit, Meta, MetaNameValue, Token};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GsiDefinition {
    pub name: String,
    pub partition_key: Option<String>,
    pub sort_keys: Vec<String>,
}

pub(crate) fn find_unary_attr(attr: &syn::Attribute, name: &str) -> Option<proc_macro2::Ident> {
    match attr.meta {
        Meta::List(ref list) => {
            match list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated) {
                Ok(parsed) if parsed.is_empty() => None,
                Ok(parsed) if parsed.len() > 1 => panic!("TODO: should unary"),
                Ok(parsed) => {
                    let meta = parsed.first().expect("should get meta");

                    if meta.path().segments[0].ident == name {
                        Some(meta.path().segments[0].ident.clone())
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
        _ => None,
    }
}

pub(crate) fn find_eq_string_from(attr: &syn::Attribute, name: &str) -> Option<String> {
    match attr.meta {
        Meta::List(ref list) => {
            match list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated) {
                Ok(parsed) => {
                    for meta in parsed.iter() {
                        match meta {
                            Meta::NameValue(MetaNameValue {
                                value:
                                    Expr::Lit(ExprLit {
                                        lit: Lit::Str(lit), ..
                                    }),
                                ..
                            }) if meta.path().segments[0].ident == name => {
                                return Some(lit.value());
                            }
                            _ => continue,
                        }
                    }

                    None
                }
                _ => None,
            }
        }
        _ => None,
    }
}

pub(crate) fn find_table_name(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().segments[0].ident != "raiden" {
            continue;
        }

        if let Some(lit) = find_eq_string_from(attr, "table_name") {
            return Some(lit);
        }
    }

    None
}

pub(crate) fn find_rename_all(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().segments[0].ident != "raiden" {
            continue;
        }

        if let Some(lit) = find_eq_string_from(attr, "rename_all") {
            return Some(lit);
        }
    }

    None
}

pub(crate) fn find_gsi_names(attrs: &[syn::Attribute]) -> Vec<String> {
    let mut names = vec![];

    for attr in attrs {
        if attr.path().segments[0].ident != "raiden" {
            continue;
        }

        if let Some(lit) = find_eq_string_from(attr, "gsi") {
            names.push(lit);
        }
    }

    for gsi in find_gsi_definitions(attrs) {
        if !names.iter().any(|name| name == &gsi.name) {
            names.push(gsi.name);
        }
    }

    names
}

pub(crate) fn find_gsi_definitions(attrs: &[syn::Attribute]) -> Vec<GsiDefinition> {
    let mut defs = vec![];

    for attr in attrs {
        if attr.path().segments[0].ident != "raiden" {
            continue;
        }

        let Meta::List(ref list) = attr.meta else {
            continue;
        };

        let Ok(parsed) = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        else {
            continue;
        };

        for meta in parsed.iter() {
            let Meta::List(gsi_list) = meta else {
                continue;
            };

            if gsi_list.path.segments[0].ident != "gsi" {
                continue;
            }

            let Ok(gsi_args) =
                gsi_list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            else {
                continue;
            };

            let mut name = None;
            let mut partition_key = None;
            let mut sort_keys = vec![];

            for gsi_arg in gsi_args.iter() {
                match gsi_arg {
                    Meta::NameValue(MetaNameValue {
                        path,
                        value:
                            Expr::Lit(ExprLit {
                                lit: Lit::Str(lit), ..
                            }),
                        ..
                    }) if path.segments[0].ident == "name" => {
                        name = Some(lit.value());
                    }
                    Meta::NameValue(MetaNameValue {
                        path,
                        value:
                            Expr::Lit(ExprLit {
                                lit: Lit::Str(lit), ..
                            }),
                        ..
                    }) if path.segments[0].ident == "partition_key" => {
                        partition_key = Some(lit.value());
                    }
                    Meta::NameValue(MetaNameValue {
                        path,
                        value:
                            Expr::Lit(ExprLit {
                                lit: Lit::Str(lit), ..
                            }),
                        ..
                    }) if path.segments[0].ident == "sort_key" => {
                        sort_keys.push(lit.value());
                    }
                    _ => {}
                }
            }

            if let Some(name) = name {
                defs.push(GsiDefinition {
                    name,
                    partition_key,
                    sort_keys,
                });
            }
        }
    }

    defs
}

pub(crate) fn find_rename_value(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().segments[0].ident != "raiden" {
            continue;
        }

        if let Some(lit) = find_eq_string_from(attr, "rename") {
            return Some(lit);
        }
    }

    None
}

pub(crate) fn include_unary_attr(attrs: &[syn::Attribute], name: &str) -> bool {
    !attrs.is_empty()
        && attrs.iter().any(|attr| {
            attr.path().segments[0].ident == "raiden" && find_unary_attr(attr, name).is_some()
        })
}

// TODO: Add validation
pub(crate) fn find_partition_key_field(fields: &syn::FieldsNamed) -> Option<syn::Field> {
    let fields: Vec<syn::Field> = fields
        .named
        .iter()
        .filter(|f| include_unary_attr(&f.attrs, "partition_key"))
        .cloned()
        .collect();

    if fields.len() > 1 {
        panic!("partition key should be only one.")
    }
    fields.first().cloned()
}

pub(crate) fn find_sort_key_field(fields: &syn::FieldsNamed) -> Option<syn::Field> {
    let fields: Vec<syn::Field> = fields
        .named
        .iter()
        .filter(|f| include_unary_attr(&f.attrs, "sort_key"))
        .cloned()
        .collect();

    if fields.len() > 1 {
        panic!("sort key should be only one.")
    }

    fields.first().cloned()
}

pub(crate) fn is_option(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(syn::TypePath {
            path: syn::Path { segments, .. },
            ..
        }) => segments.iter().any(|s| s.ident == "Option"),
        _ => false,
    }
}
