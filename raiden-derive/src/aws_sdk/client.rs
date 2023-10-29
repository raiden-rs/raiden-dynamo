use quote::*;
use syn::*;

pub(crate) fn expand_client_constructor(
    struct_name: &Ident,
    client_name: &Ident,
    dynamodb_client_name: &Ident,
    table_name: &str,
    fields: &FieldsNamed,
    rename_all_type: &crate::rename::RenameAllType,
) -> proc_macro2::TokenStream {
    let insertion_attribute_name = fields.named.iter().map(|f| {
        let ident = &f.ident.clone().unwrap();
        let renamed = crate::finder::find_rename_value(&f.attrs);
        let result = crate::create_renamed(ident.to_string(), renamed, *rename_all_type);
        quote! {
            names.insert(
                format!("#{}", #result.clone()),
                #result.to_string(),
            );
        }
    });

    quote! {
        impl #client_name {

            pub fn new(region: ::raiden::Region) -> Self {
                let config = ::raiden::Config::builder().region(region).build();
                let client = ::raiden::#dynamodb_client_name::from_conf(config);
                Self::new_with_dynamo_db_client(client)
            }

            pub fn new_with_client(client: ::raiden::Client) -> Self {
                Self::new_with_dynamo_db_client(client)
            }

            fn new_with_dynamo_db_client(client: ::raiden::#dynamodb_client_name) -> Self {
                let names = {
                    let mut names: ::raiden::AttributeNames = std::collections::HashMap::new();
                    #(#insertion_attribute_name)*
                    names
                };
                let projection_expression = Some(names.keys().map(|v| v.to_string()).collect::<Vec<String>>().join(", "));

                Self {
                    table_name: #table_name,
                    table_prefix: "".to_owned(),
                    table_suffix: "".to_owned(),
                    client,
                    retry_condition: ::raiden::RetryCondition::new(),
                    attribute_names: Some(names),
                    projection_expression
                }
            }

            pub fn with_retries(mut self, s: Box<dyn ::raiden::retry::RetryStrategy + Send + Sync>) -> Self {
                self.retry_condition.strategy = s;
                self
            }

            pub fn table_prefix(mut self, prefix: impl Into<String>) -> Self {
                self.table_prefix = prefix.into();
                self
            }

            pub fn table_suffix(mut self, suffix: impl Into<String>) -> Self {
                self.table_suffix = suffix.into();
                self
            }

            pub fn table_name(&self) -> String {
                format!("{}{}{}", self.table_prefix, self.table_name.to_string(), self.table_suffix)
            }
        }

        impl #struct_name {
            pub fn client(region: ::raiden::Region) -> #client_name {
                #client_name::new(region)
            }

            pub fn client_with(client: ::raiden::Client) -> #client_name {
                #client_name::new_with_client(client)
            }
        }
    }
}
