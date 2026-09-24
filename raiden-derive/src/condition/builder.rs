use quote::*;

pub fn expand_condition_builder(
    _attr_enum_name: &proc_macro2::Ident,
    struct_name: &proc_macro2::Ident,
    _fields: &syn::FieldsNamed,
) -> proc_macro2::TokenStream {
    let condition_name = format_ident!("{}Condition", struct_name);
    let condition_token_name = format_ident!("{}ConditionToken", struct_name);
    let wait_attr_op_name = format_ident!("{}LeftAttrAndWaitOp", struct_name);
    let attribute_value_path = if cfg!(feature = "rusoto") {
        quote! { ::raiden::AttributeValue }
    } else if cfg!(feature = "aws-sdk") {
        quote! { ::raiden::aws_sdk::types::AttributeValue }
    } else {
        unreachable!();
    };
    let comparison_methods = [
        ("ne", "Ne"),
        ("lt", "Lt"),
        ("le", "Le"),
        ("gt", "Gt"),
        ("ge", "Ge"),
    ]
    .into_iter()
    .map(|(method, operator)| {
        let attr_method = format_ident!("{method}_attr");
        let value_method = format_ident!("{method}_value");
        let operator = format_ident!("{operator}");
        quote! {
            pub fn #attr_method(self, attr: impl ::raiden::IntoAttrPath) -> ::raiden::ConditionFilledOrWaitOperator<#condition_token_name> {
                self.compare(
                    ::raiden::ConditionComparisonOperator::#operator,
                    ::raiden::AttrOrPlaceholder::Attr(attr.into_attr_path()),
                    None,
                )
            }

            pub fn #value_method(self, value: impl ::raiden::IntoAttribute) -> ::raiden::ConditionFilledOrWaitOperator<#condition_token_name> {
                let placeholder = ::raiden::AttrOrPlaceholder::Placeholder(format!("value{}", ::raiden::generate_value_id()));
                self.compare(
                    ::raiden::ConditionComparisonOperator::#operator,
                    placeholder,
                    Some(value.into_attr()),
                )
            }
        }
    });

    quote! {

        #[derive(Debug, Clone)]
        pub struct #condition_token_name;


        #[derive(Debug, Clone)]
        pub struct #condition_name {
            not: bool,
        }

        impl #struct_name {
            pub fn condition() -> #condition_name {
                #condition_name {
                    not: false,
                }
            }
        }

        impl #condition_name {
            pub fn not(mut self) -> Self {
                self.not = true;
                self
            }
            pub fn attr_exists(self, field: impl ::raiden::IntoAttrPath) -> ::raiden::ConditionFilledOrWaitOperator<#condition_token_name> {
                let cond = ::raiden::condition::Cond::Func(::raiden::condition::ConditionFunctionExpression::AttributeExists(field.into_attr_path()));
                ::raiden::ConditionFilledOrWaitOperator {
                    not: self.not,
                    cond,
                    _token: std::marker::PhantomData,
                }
            }
            pub fn attr_not_exists(self, field: impl ::raiden::IntoAttrPath) -> ::raiden::ConditionFilledOrWaitOperator<#condition_token_name> {
                let cond = ::raiden::condition::Cond::Func(::raiden::condition::ConditionFunctionExpression::AttributeNotExists(field.into_attr_path()));
                ::raiden::ConditionFilledOrWaitOperator {
                    not: self.not,
                    cond,
                    _token: std::marker::PhantomData,
                }
            }
            pub fn attr_type(self, field: impl ::raiden::IntoAttrPath, t: ::raiden::AttributeType) -> ::raiden::ConditionFilledOrWaitOperator<#condition_token_name> {
                let cond = ::raiden::condition::Cond::Func(::raiden::condition::ConditionFunctionExpression::AttributeType(field.into_attr_path(), t));
                ::raiden::ConditionFilledOrWaitOperator {
                    not: self.not,
                    cond,
                    _token: std::marker::PhantomData,
                }
            }
            pub fn begins_with(self, field: impl ::raiden::IntoAttrPath, s: impl Into<String>) -> ::raiden::ConditionFilledOrWaitOperator<#condition_token_name> {
                let cond = ::raiden::condition::Cond::Func(::raiden::condition::ConditionFunctionExpression::BeginsWith(field.into_attr_path(), s.into()));
                ::raiden::ConditionFilledOrWaitOperator {
                    not: self.not,
                    cond,
                    _token: std::marker::PhantomData,
                }
            }

            pub fn contains(self, field: impl ::raiden::IntoAttrPath, value: impl ::raiden::IntoAttribute) -> ::raiden::ConditionFilledOrWaitOperator<#condition_token_name> {
                let placeholder = format!(":value{}", ::raiden::generate_value_id());
                let cond = ::raiden::condition::Cond::Func(::raiden::condition::ConditionFunctionExpression::ContainsValue(field.into_attr_path(), placeholder, Box::new(value.into_attr())));
                ::raiden::ConditionFilledOrWaitOperator {
                    not: self.not,
                    cond,
                    _token: std::marker::PhantomData,
                }
            }

            pub fn size(self, field: impl ::raiden::IntoAttrPath) -> ::raiden::ConditionSize<#condition_token_name> {
                ::raiden::ConditionSize {
                    not: self.not,
                    attr: field.into_attr_path(),
                    _token: std::marker::PhantomData,
                }
            }

            pub fn between(self, field: impl ::raiden::IntoAttrPath, lower: impl ::raiden::IntoAttribute, upper: impl ::raiden::IntoAttribute) -> ::raiden::ConditionFilledOrWaitOperator<#condition_token_name> {
                let lower_placeholder = format!(":value{}", ::raiden::generate_value_id());
                let upper_placeholder = format!(":value{}", ::raiden::generate_value_id());
                let cond = ::raiden::condition::Cond::Cmp(::raiden::ConditionComparisonExpression::Between(
                    field.into_attr_path(), lower_placeholder, lower.into_attr(), upper_placeholder, upper.into_attr(),
                ));
                ::raiden::ConditionFilledOrWaitOperator {
                    not: self.not,
                    cond,
                    _token: std::marker::PhantomData,
                }
            }

            /// DynamoDB accepts between one and 100 values in an IN condition.
            /// Panics when the iterator produces no values or more than 100 values.
            pub fn in_values<V: ::raiden::IntoAttribute>(self, field: impl ::raiden::IntoAttrPath, values: impl IntoIterator<Item = V>) -> ::raiden::ConditionFilledOrWaitOperator<#condition_token_name> {
                let values: Vec<_> = values.into_iter().take(101).map(|value| {
                    (format!(":value{}", ::raiden::generate_value_id()), value.into_attr())
                }).collect();
                assert!((1..=100).contains(&values.len()), "IN requires between 1 and 100 values");
                let cond = ::raiden::condition::Cond::Cmp(::raiden::ConditionComparisonExpression::In(field.into_attr_path(), values));
                ::raiden::ConditionFilledOrWaitOperator {
                    not: self.not,
                    cond,
                    _token: std::marker::PhantomData,
                }
            }

            pub fn attr(self, field: impl ::raiden::IntoAttrPath) -> #wait_attr_op_name {
                #wait_attr_op_name {
                    not: self.not,
                    attr_or_placeholder: ::raiden::AttrOrPlaceholder::Attr(field.into_attr_path()),
                    attr_value: None,
                }
            }

            pub fn value(self, value: impl ::raiden::IntoAttribute) -> #wait_attr_op_name {
                let placeholder = format!("value{}", ::raiden::generate_value_id());
                #wait_attr_op_name {
                    not: self.not,
                    attr_or_placeholder: ::raiden::AttrOrPlaceholder::Placeholder(placeholder),
                    attr_value: Some(value.into_attr()),
                }
            }
        }

        pub struct #wait_attr_op_name {
            not: bool,
            attr_or_placeholder: ::raiden::AttrOrPlaceholder,
            attr_value: Option<#attribute_value_path>,
        }

        impl #wait_attr_op_name {
            fn compare(self, operator: ::raiden::ConditionComparisonOperator, right: ::raiden::AttrOrPlaceholder, right_value: Option<#attribute_value_path>) -> ::raiden::ConditionFilledOrWaitOperator<#condition_token_name> {
                let cond = ::raiden::condition::Cond::Cmp(::raiden::ConditionComparisonExpression::Compare(
                    self.attr_or_placeholder, self.attr_value, operator, right, right_value,
                ));
                ::raiden::ConditionFilledOrWaitOperator {
                    not: self.not,
                    cond,
                    _token: std::marker::PhantomData,
                }
            }

            #(#comparison_methods)*

            pub fn eq_attr(self, attr: impl ::raiden::IntoAttrPath) -> ::raiden::ConditionFilledOrWaitOperator<#condition_token_name>  {
                let attr = ::raiden::AttrOrPlaceholder::Attr(attr.into_attr_path());
                let cond = ::raiden::condition::Cond::Cmp(::raiden::condition::ConditionComparisonExpression::Eq(self.attr_or_placeholder, self.attr_value, attr, None));
                ::raiden::ConditionFilledOrWaitOperator {
                    not: self.not,
                    cond,
                    _token: std::marker::PhantomData,
                }

            }

            pub fn eq_value(self, value: impl ::raiden::IntoAttribute) -> ::raiden::ConditionFilledOrWaitOperator<#condition_token_name>  {
                let placeholder = ::raiden::AttrOrPlaceholder::Placeholder(format!("value{}", ::raiden::generate_value_id()));
                let cond = ::raiden::condition::Cond::Cmp(::raiden::condition::ConditionComparisonExpression::Eq(self.attr_or_placeholder, self.attr_value, placeholder, Some(value.into_attr())));
                ::raiden::ConditionFilledOrWaitOperator {
                    not: self.not,
                    cond,
                    _token: std::marker::PhantomData,
                }
            }
        }
    }
}
