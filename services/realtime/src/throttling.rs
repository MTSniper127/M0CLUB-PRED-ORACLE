
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct Throttle {
    pub min_interval: Duration,
    last: std::sync::Arc<tokio::sync::Mutex<Instant>>,
}

impl Throttle {
    pub fn new(min_interval: Duration) -> Self {
        Self { min_interval, last: std::sync::Arc::new(tokio::sync::Mutex::new(Instant::now() - min_interval)) }
    }

    pub async fn wait(&self) {
        let mut last = self.last.lock().await;
        let now = Instant::now();
        let elapsed = now.duration_since(*last);
        if elapsed < self.min_interval {
            tokio::time::sleep(self.min_interval - elapsed).await;
        }
        *last = Instant::now();
    }
}
