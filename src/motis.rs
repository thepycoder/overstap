//! Thin MOTIS REST client. Placeholder until Phase 2, when we generate or
//! hand-write typed calls from MOTIS's `openapi.yaml`.

pub struct Client {
    base_url: String,
}

impl Client {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}
