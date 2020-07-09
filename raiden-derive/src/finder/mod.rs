pub(crate) fn find_unary_attr(attr: &syn::Attribute, name: &str) -> Option<proc_macro2::Ident> {
    let mut tokens = match attr.tokens.clone().into_iter().next() {
        Some(proc_macro2::TokenTree::Group(g)) => g.stream().into_iter(),
        _ => return None,
    };
    let ident = match tokens.next() {
        Some(proc_macro2::TokenTree::Ident(ref ident)) if *ident == name => ident.clone(),
        _ => return None,
    };
    if tokens.next().is_none() {
        return Some(ident);
    }
    panic!("TODO: should unaray");
}

pub(crate) fn find_eq_string_from(attr: &syn::Attribute, name: &str) -> Option<String> {
    let mut tokens = match attr.tokens.clone().into_iter().next() {
        Some(proc_macro2::TokenTree::Group(g)) => g.stream().into_iter(),
        _ => return None,
    };

    match tokens.next() {
        Some(proc_macro2::TokenTree::Ident(ref ident)) if *ident == name => {}
        _ => return None,
    };

    // #[raiden(name = )]
    match tokens.next() {
        Some(proc_macro2::TokenTree::Punct(ref punct)) if punct.as_char() == '=' => {}
        _ => return None,
    };

    // #[raiden(name = value)]
    let lit = match tokens.next() {
        Some(proc_macro2::TokenTree::Literal(lit)) => syn::Lit::new(lit),
        _ => return None,
    };

    match &lit {
        syn::Lit::Str(lit_str) => {
            let value = lit_str.value();
            if value.trim().is_empty() {
                panic!()
            };
            return Some(value);
        }
        _ => return None,
    }
}

pub(crate) fn find_table_name(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if let Some(lit) = find_eq_string_from(&attr, "table_name") {
            return Some(lit);
        }
    }
    None
}

pub(crate) fn find_rename_all(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if let Some(lit) = find_eq_string_from(&attr, "rename_all") {
            return Some(lit);
        }
    }
    None
}

pub(crate) fn find_rename_value(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if let Some(lit) = find_eq_string_from(&attr, "rename") {
            return Some(lit);
        }
    }
    None
}

pub(crate) fn include_unary_attr(attrs: &[syn::Attribute], name: &str) -> bool {
    return attrs.len() > 0
        && attrs.iter().any(|attr| {
            attr.path.segments[0].ident == "raiden" && find_unary_attr(&attr, name).is_some()
        });
}

// TODO: Add validation
pub(crate) fn find_partition_key_field(fields: &syn::FieldsNamed) -> Option<syn::Field> {
    let fields: Vec<syn::Field> = fields
        .named
        .iter()
        .cloned()
        .filter(|f| include_unary_attr(&f.attrs, "partition_key"))
        .collect();

    if fields.len() > 1 {
        panic!("partition key should be only one.")
    }
    fields.get(0).cloned()
}

pub(crate) fn find_sort_key_field(fields: &syn::FieldsNamed) -> Option<syn::Field> {
    let fields: Vec<syn::Field> = fields
        .named
        .iter()
        .cloned()
        .filter(|f| include_unary_attr(&f.attrs, "sort_key"))
        .collect();

    if fields.len() > 1 {
        panic!("sort key should be only one.")
    }
    fields.get(0).cloned()
}
