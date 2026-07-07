use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

use ed25519_dalek::PUBLIC_KEY_LENGTH;

#[derive(Default, Clone)]
pub struct Registry {
    inner: Arc<RwLock<HashMap<String, String>>>,
    save_path: Option<PathBuf>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_path(path: impl Into<PathBuf>) -> Self {
        Self {
            save_path: Some(path.into()),
            ..Self::new()
        }
    }

    pub async fn add(&self, server_id: &str, public_key: &str) {
        self.inner.write().await.insert(server_id.to_owned(), public_key.to_owned());
        self.persist().await;
    }

    pub async fn revoke(&self, server_id: &str) {
        self.inner.write().await.remove(server_id);
        self.persist().await;
    }

    pub async fn get(&self, server_id: &str) -> Option<String> {
        self.inner.read().await.get(server_id).cloned()
    }

    pub async fn verify_signature(&self, server_id: &str, public_key: &str) -> bool {
        let registry = self.inner.read().await;
        match registry.get(server_id) {
            Some(expected) => expected == public_key,
            None => false,
        }
    }

    pub async fn check_signing(&self, _server_id: &str, public_key: &str) -> bool {
        match base64::decode(public_key) {
            Ok(bytes) => bytes.len() == PUBLIC_KEY_LENGTH,
            Err(_) => false,
        }
    }

    pub async fn load_from(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let mut registry = Self::with_path(path.clone());
        if let Ok(data) = fs::read(&path).await {
            if let Ok(map) = serde_json::from_slice::<HashMap<String, String>>(&data) {
                *registry.inner.write().await = map;
            } else {
                tracing::warn!(path=?path, "failed to parse registry file");
            }
        }
        registry
    }

    async fn persist(&self) {
        if let Some(path) = &self.save_path {
            let map = self.inner.read().await.clone();
            if let Ok(json) = serde_json::to_string_pretty(&map) {
                if let Err(error) = fs::write(path, json).await {
                    tracing::warn!(path=?path, error=%error, "failed to persist registry");
                }
            }
        }
    }
}
