use quote::*;

use crate::rename::*;
use convert_case::{Case, Casing};

// TODO: Add map and list accessor
//       e.g. MyMap.nestedField.deeplyNestedField
//       Should we annotate map or list accessor with following derive?
//       #[raiden(expression_name = "MyMap.nestedField.deeplyNestedField")]
pub fn expand_attr_names(
    attr_enum_name: &proc_macro2::Ident,
    fields: &syn::FieldsNamed,
    rename_all_type: crate::rename::RenameAllType,
    struct_name: &proc_macro2::Ident,
) -> proc_macro2::TokenStream {
    let names = fields.named.iter().map(|f| {
        let ident = &f.ident.clone().unwrap();
        let renamed = crate::finder::find_rename_value(&f.attrs);

        let name = if let Some(renamed) = renamed {
            renamed.to_case(Case::Pascal)
        } else {
            ident.to_string().to_case(Case::Pascal)
        };
        let name = format_ident!("{}", name);
        quote! {
            #name
        }
    });

    let arms = fields.named.iter().map(|f| {
        let ident = &f.ident.clone().unwrap();
        let renamed = crate::finder::find_rename_value(&f.attrs);
        let basename = create_renamed(ident.to_string(), renamed, rename_all_type);
        let attr_name = basename.to_string();
        let name = basename.to_case(Case::Pascal);
        let name = format_ident!("{}", name);
        quote! {
            #attr_enum_name::#name => #attr_name.to_owned()
        }
    });

    let getters = fields.named.iter().map(|f| {
        let ident = &f.ident.clone().unwrap();
        let renamed = crate::finder::find_rename_value(&f.attrs);
        let basename = create_renamed(ident.to_string(), renamed, rename_all_type);
        let func_name = basename.to_case(Case::Snake);
        let func_name = if crate::helpers::is_reserved(&func_name) {
            format_ident!("r#{}", func_name)
        } else {
            format_ident!("{}", func_name)
        };
        let name = basename.to_case(Case::Pascal);
        let name = format_ident!("{}", name);
        quote! {
            pub fn #func_name() -> #attr_enum_name {
                #attr_enum_name::#name
            }
        }
    });

    quote! {
        #[derive(Debug, Clone, Copy, PartialEq)]
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

        // attr name getter
        impl #struct_name {
            #(
                #getters
            )*
        }

    }
}
