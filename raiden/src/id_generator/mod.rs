pub trait IdGenerator {
    #[cfg(not(test))]
    fn gen() -> String {
        match std::env::var("RAIDEN_UUID_FIXED_BY") {
            Ok(v) => v,
            Err(_) => uuid::Uuid::new_v4().to_string(),
        }
    }

    #[cfg(test)]
    fn gen() -> String {
        "01234567-89ab-cdef-0123-456789abcdef".to_owned()
    }
}
