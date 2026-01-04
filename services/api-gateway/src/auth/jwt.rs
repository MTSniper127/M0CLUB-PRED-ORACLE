
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct JwtAuth {
    pub secret: String,
    pub audience: Option<String>,
    pub issuer: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub aud: Option<String>,
    pub iss: Option<String>,
}

impl JwtAuth {
    pub fn from_env() -> Self {
        Self {
            secret: std::env::var("M0_JWT_SECRET").unwrap_or_else(|_| "dev_secret_change_me".into()),
            audience: std::env::var("M0_JWT_AUD").ok(),
            issuer: std::env::var("M0_JWT_ISS").ok(),
        }
    }

    pub fn verify(&self, token: &str) -> anyhow::Result<Claims> {
        let mut v = Validation::default();
        if let Some(aud) = &self.audience { v.set_audience(&[aud]); }
        if let Some(iss) = &self.issuer { v.set_issuer(&[iss]); }
        let data = decode::<Claims>(token, &DecodingKey::from_secret(self.secret.as_bytes()), &v)?;
        Ok(data.claims)
    }
}
