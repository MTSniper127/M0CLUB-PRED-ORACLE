
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct Ttl<T> {
    pub value: T,
    pub expires_at: Instant,
}

impl<T> Ttl<T> {
    pub fn new(value: T, ttl: Duration) -> Self {
        Self { value, expires_at: Instant::now() + ttl }
    }
    pub fn expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }
}
