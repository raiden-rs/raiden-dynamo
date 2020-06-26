use quote::*;

// TODO: Add map and list accessor
//       e.g. MyMap.nestedField.deeplyNestedField
//       Should we annotate map or list accessor with following derive?
//       #[raiden(expression_name = "MyMap.nestedField.deeplyNestedField")]
pub fn expand_attr_names(
    attr_enum_name: &proc_macro2::Ident,
    fields: &syn::FieldsNamed,
) -> proc_macro2::TokenStream {
    let names = fields.named.iter().map(|f| {
        let ident = &f.ident.clone().unwrap();
        let renamed = crate::finder::find_rename_value(&f.attrs);

        let name = if renamed.is_none() {
            ident_case::RenameRule::PascalCase.apply_to_field(ident.to_string())
        } else {
            ident_case::RenameRule::PascalCase.apply_to_field(renamed.unwrap())
        };
        let name = format_ident!("{}", name);
        quote! {
          #name
        }
    });

    let arms = fields.named.iter().map(|f| {
        let ident = &f.ident.clone().unwrap();
        let renamed = crate::finder::find_rename_value(&f.attrs);

        let basename = if renamed.is_none() {
            ident.to_string()
        } else {
            renamed.unwrap()
        };

        let attr_name = format!("{}", basename);
        let name = ident_case::RenameRule::PascalCase.apply_to_field(basename);

        let name = format_ident!("{}", name);

        quote! {
            #attr_enum_name::#name => #attr_name.to_owned()
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
    }
}

/*
let input_fields = fields
.named
.iter()
.filter(|f| !crate::finder::include_unary_attr(&f.attrs, "uuid"))
.map(|f| {
    let ident = &f.ident.clone().unwrap();
    let ty = &f.ty;
    quote! {
        #ident: #ty,
    }
});
*/
