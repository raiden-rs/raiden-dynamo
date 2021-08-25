use quote::*;

pub fn expand_condition_builder(
    attr_enum_name: &proc_macro2::Ident,
    struct_name: &proc_macro2::Ident,
    _fields: &syn::FieldsNamed,
) -> proc_macro2::TokenStream {
    let condition_name = format_ident!("{}Condition", struct_name);
    let condition_token_name = format_ident!("{}ConditionToken", struct_name);
    let wait_attr_op_name = format_ident!("{}LeftAttrAndWaitOp", struct_name);

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
            pub fn attr_exists(self, field: #attr_enum_name) -> ConditionFilledOrWaitConjunction<#condition_token_name> {
                let cond = ::raiden::condition::Cond::Func(::raiden::condition::ConditionFunctionExpression::AttributeExists(field.into_attr_name()));
                ConditionFilledOrWaitConjunction {
                    not: self.not,
                    cond,
                    _token: std::marker::PhantomData,
                }
            }
            pub fn attr_not_exists(self, field: #attr_enum_name) -> ConditionFilledOrWaitConjunction<#condition_token_name> {
                let cond = ::raiden::condition::Cond::Func(::raiden::condition::ConditionFunctionExpression::AttributeNotExists(field.into_attr_name()));
                ConditionFilledOrWaitConjunction {
                    not: self.not,
                    cond,
                    _token: std::marker::PhantomData,
                }
            }
            pub fn attr_type(self, field: #attr_enum_name, t: ::raiden::AttributeType) -> ConditionFilledOrWaitConjunction<#condition_token_name> {
                let cond = ::raiden::condition::Cond::Func(::raiden::condition::ConditionFunctionExpression::AttributeType(field.into_attr_name(), t));
                ConditionFilledOrWaitConjunction {
                    not: self.not,
                    cond,
                    _token: std::marker::PhantomData,
                }
            }
            pub fn begins_with(self, field: #attr_enum_name, s: impl Into<String>) -> ConditionFilledOrWaitConjunction<#condition_token_name> {
                let cond = ::raiden::condition::Cond::Func(::raiden::condition::ConditionFunctionExpression::BeginsWith(field.into_attr_name(), s.into()));
                ConditionFilledOrWaitConjunction {
                    not: self.not,
                    cond,
                    _token: std::marker::PhantomData,
                }
            }

            pub fn contains(self, field: #attr_enum_name, s: impl Into<String>) -> ConditionFilledOrWaitConjunction<#condition_token_name> {
                let cond = ::raiden::condition::Cond::Func(::raiden::condition::ConditionFunctionExpression::Contains(field.into_attr_name(), s.into()));
                ConditionFilledOrWaitConjunction {
                    not: self.not,
                    cond,
                    _token: std::marker::PhantomData,
                }
            }

            pub fn attr(self, field: #attr_enum_name) -> #wait_attr_op_name {
                #wait_attr_op_name {
                    not: self.not,
                    attr_or_placeholder: ::raiden::AttrOrPlaceholder::Attr(field.into_attr_name()),
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
            attr_value: Option<::raiden::AttributeValue>
        }

        impl #wait_attr_op_name {
            pub fn eq_attr(self, attr: #attr_enum_name) -> ConditionFilledOrWaitConjunction<#condition_token_name>  {
                let attr = ::raiden::AttrOrPlaceholder::Attr(attr.into_attr_name());
                let cond = ::raiden::condition::Cond::Cmp(::raiden::condition::ConditionComparisonExpression::Eq(self.attr_or_placeholder, self.attr_value, attr, None));
                ConditionFilledOrWaitConjunction {
                    not: self.not,
                    cond,
                    _token: std::marker::PhantomData,
                }

            }

            pub fn eq_value(self, value: impl ::raiden::IntoAttribute) -> ConditionFilledOrWaitConjunction<#condition_token_name>  {
                let placeholder = ::raiden::AttrOrPlaceholder::Placeholder(format!("value{}", ::raiden::generate_value_id()));
                let cond = ::raiden::condition::Cond::Cmp(::raiden::condition::ConditionComparisonExpression::Eq(self.attr_or_placeholder, self.attr_value, placeholder, Some(value.into_attr())));
                ConditionFilledOrWaitConjunction {
                    not: self.not,
                    cond,
                    _token: std::marker::PhantomData,
                }
            }
        }
    }
}
