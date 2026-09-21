use std::sync::atomic::AtomicUsize;
static VALUE_ID: AtomicUsize = AtomicUsize::new(0);

pub fn generate_value_id() -> usize {
    use std::sync::atomic::Ordering;

    VALUE_ID.fetch_add(1, Ordering::Relaxed)
}

pub fn reset_value_id() {
    use std::sync::atomic::Ordering;

    VALUE_ID.store(0, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    #[test]
    fn generates_unique_ids_across_threads() {
        let handles = (0..8)
            .map(|_| {
                std::thread::spawn(|| {
                    (0..128)
                        .map(|_| super::generate_value_id())
                        .collect::<Vec<_>>()
                })
            })
            .collect::<Vec<_>>();

        let ids = handles
            .into_iter()
            .flat_map(|handle| handle.join().unwrap())
            .collect::<std::collections::HashSet<_>>();

        assert_eq!(ids.len(), 8 * 128);
    }
}
