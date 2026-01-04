
use dashmap::DashMap;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct Hub {
    topics: std::sync::Arc<DashMap<String, broadcast::Sender<String>>>,
}

impl Hub {
    pub fn new() -> Self {
        Self { topics: std::sync::Arc::new(DashMap::new()) }
    }

    pub fn topic(&self, name: &str) -> broadcast::Sender<String> {
        if let Some(s) = self.topics.get(name) {
            return s.clone();
        }
        let (tx, _rx) = broadcast::channel(512);
        self.topics.insert(name.to_string(), tx.clone());
        tx
    }

    pub fn publish(&self, name: &str, msg: String) {
        let tx = self.topic(name);
        let _ = tx.send(msg);
    }
}
