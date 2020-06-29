#[cfg(test)]
mod tests {

    #[cfg(test)]
    use pretty_assertions::assert_eq;
    use raiden::*;

    #[derive(Raiden)]
    #[raiden(table_name = "user")]
    #[derive(Debug, Clone)]
    pub struct User {
        #[raiden(partition_key)]
        id: String,
        name: String,
    }

    #[test]
    fn test_minimum_set_update_expression() {
        let client = User::client(Region::Custom {
            endpoint: "http://localhost:8000".into(),
            name: "ap-northeast-1".into(),
        });
        reset_value_id();
        let (expression, _, _) = client
            .update("id0")
            .set(
                User::update_expression()
                    .set(UserAttrNames::Name)
                    .value("updated"),
            )
            .build_expression();

        assert_eq!(expression, "SET #name = :value0".to_owned());
    }
}
