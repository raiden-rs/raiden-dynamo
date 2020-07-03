use crate::rename::*;
use quote::*;

pub(crate) fn expand_transact_write(
    struct_name: &proc_macro2::Ident,
    fields: &syn::FieldsNamed,
    rename_all_type: crate::rename::RenameAllType,
    table_name: &str,
) -> proc_macro2::TokenStream {
    let item_input_name = format_ident!("{}PutItemInput", struct_name);
    let item_output_name = format_ident!("{}PutItemOutput", struct_name);
    // let trait_name = format_ident!("{}PutItem", struct_name);
    // let client_name = format_ident!("{}Client", struct_name);
    let builder_name = format_ident!("{}TransactPutItemBuilder", struct_name);
    let condition_token_name = format_ident!("{}ConditionToken", struct_name);

    // let input_fields = fields
    //     .named
    //     .iter()
    //     .filter(|f| !crate::finder::include_unary_attr(&f.attrs, "uuid"))
    //     .map(|f| {
    //         let ident = &f.ident.clone().unwrap();
    //         let ty = &f.ty;
    //         quote! {
    //             #ident: #ty,
    //         }
    //     });

    // let output_fields = fields.named.iter().map(|f| {
    //     let ident = &f.ident.clone().unwrap();
    //     let ty = &f.ty;
    //     quote! {
    //         pub #ident: #ty,
    //     }
    // });

    let output_values = fields.named.iter().map(|f| {
        let ident = &f.ident.clone().unwrap();
        let renamed = crate::finder::find_rename_value(&f.attrs);
        let attr_key = create_renamed(ident.to_string(), renamed, rename_all_type);
        if crate::finder::include_unary_attr(&f.attrs, "uuid") {
            quote! {
                #ident: uuid_map.get(#attr_key).cloned().unwrap().into(),
            }
        } else {
            quote! {
                #ident: item.#ident,
            }
        }
    });

    let input_items = {
        let insertion = fields.named.iter().map(|f| {
            let ident = &f.ident.clone().unwrap();
            let renamed = crate::finder::find_rename_value(&f.attrs);
            let attr_key = create_renamed(ident.to_string(), renamed, rename_all_type);
            if crate::finder::include_unary_attr(&f.attrs, "uuid") {
                quote! {
                    let id = #struct_name::gen();
                    input_item.insert(
                        #attr_key.to_string(),
                        id.clone().into_attr(),
                    );
                    uuid_map.insert(
                        #attr_key.to_string(),
                        id,
                    );
                }
            } else {
                quote! {
                    input_item.insert(
                        #attr_key.to_string(),
                        item.#ident.clone().into_attr(),
                    );
                }
            }
        });

        quote! {
            let mut input_item: std::collections::HashMap<String, raiden::AttributeValue> = std::collections::HashMap::new();
            #(#insertion)*
        }
    };

    quote! {
        // impl #struct_name {
        //     pub fn put_item_builder() -> #item_input_builder_name {
        //         #item_input_builder_name::default()
        //     }
        // }

        // #[derive(Debug, Clone, PartialEq, Builder)]
        // #[builder(setter(into))]
        // pub struct #item_input_name {
        //     #(#input_fields)*
        // }

        // #[derive(Debug, Clone, PartialEq)]
        // pub struct #item_output_name {
        //     #(#output_fields)*
        // }

        // pub trait #trait_name {
        //     fn put(&self, item: #item_input_name) -> #builder_name;
        // }

        impl #struct_name {
            pub fn put(item: #item_input_name) -> #builder_name {
                let mut input = ::raiden::Put::default();
                let mut attribute_names: std::collections::HashMap<String, String> = std::collections::HashMap::new();
                let mut attribute_values: std::collections::HashMap<String, raiden::AttributeValue> = std::collections::HashMap::new();
                let mut uuid_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();

                #input_items

                // let output_item = #item_output_name {
                //     #(#output_values)*
                // };
                input.item = input_item;
                input.table_name = #table_name.to_owned();
                #builder_name {
                    input,
                    // item: output_item,
                }
            }
        }

        pub struct #builder_name {
            pub input: ::raiden::Put,
        }

        impl ::raiden::TransactWritePutBuilder for #builder_name {
            fn build(self) -> ::raiden::Put {
                self.input
            }
        }

        impl #builder_name {
            fn condition(mut self, cond: impl ::raiden::condition::ConditionBuilder<#condition_token_name>) -> Self {
                // TODO: Implement later
                // let (cond_str, attr_names, attr_values) = cond.build();
                // if !attr_names.is_empty() {
                //     self.input.expression_attribute_names = Some(attr_names);
                // }
                // if !attr_values.is_empty() {
                //     self.input.expression_attribute_values = Some(attr_values);
                // }
                // self.input.condition_expression = Some(cond_str);
                self
            }
        }
    }
}

/*
#[derive(Default, Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "deserialize_structs", derive(Deserialize))]
pub struct Put {
    /// <p>A condition that must be satisfied in order for a conditional update to succeed.</p>
    #[serde(rename = "ConditionExpression")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition_expression: Option<String>,
    /// <p>One or more substitution tokens for attribute names in an expression.</p>
    #[serde(rename = "ExpressionAttributeNames")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expression_attribute_names: Option<::std::collections::HashMap<String, String>>,
    /// <p>One or more values that can be substituted in an expression.</p>
    #[serde(rename = "ExpressionAttributeValues")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expression_attribute_values: Option<::std::collections::HashMap<String, AttributeValue>>,
    /// <p>A map of attribute name to attribute values, representing the primary key of the item to be written by <code>PutItem</code>. All of the table's primary key attributes must be specified, and their data types must match those of the table's key schema. If any attributes are present in the item that are part of an index key schema for the table, their types must match the index key schema. </p>
    #[serde(rename = "Item")]
    pub item: ::std::collections::HashMap<String, AttributeValue>,
    /// <p>Use <code>ReturnValuesOnConditionCheckFailure</code> to get the item attributes if the <code>Put</code> condition fails. For <code>ReturnValuesOnConditionCheckFailure</code>, the valid values are: NONE and ALL_OLD.</p>
    #[serde(rename = "ReturnValuesOnConditionCheckFailure")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_values_on_condition_check_failure: Option<String>,
    /// <p>Name of the table in which to write the item.</p>
    #[serde(rename = "TableName")]
    pub table_name: String,
}*/
