use std::sync::atomic::{AtomicU64, Ordering};

pub trait Entity {
    fn entity_id(&self) -> u64;
    fn update(&mut self);
}

static ID_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn generate_id() -> u64 {
    ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}
