use std::sync::atomic::AtomicUsize;
static VALUE_ID: AtomicUsize = AtomicUsize::new(0);

pub fn generate_value_id() -> usize {
    use std::sync::atomic::Ordering;

    let id = VALUE_ID.load(Ordering::Relaxed);
    VALUE_ID.store(id.wrapping_add(1), Ordering::Relaxed);
    id
}

pub fn reset_value_id() {
    use std::sync::atomic::Ordering;

    VALUE_ID.store(0, Ordering::Relaxed);
}
