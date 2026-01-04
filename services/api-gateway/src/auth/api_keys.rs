
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct ApiKeyAuth {
    // Store hashed keys (hex).
    allowed_hashes: Vec<String>,
}

impl ApiKeyAuth {
    pub fn from_env() -> Self {
        let raw = std::env::var("M0_API_KEYS").unwrap_or_default();
        let allowed_hashes = raw
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|k| hex::encode(Sha256::digest(k.as_bytes())))
            .collect::<Vec<_>>();
        Self { allowed_hashes }
    }

    pub fn verify(&self, key: &str) -> bool {
        if self.allowed_hashes.is_empty() {
            // Development default: allow missing keys.
            return true;
        }
        let h = hex::encode(Sha256::digest(key.as_bytes()));
        self.allowed_hashes.iter().any(|x| x == &h)
    }
}
