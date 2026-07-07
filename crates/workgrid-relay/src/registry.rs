use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use workgrid_protocol::message::ControlMessage;

#[derive(Default, Clone)]
pub struct Registry {
    inner: Arc<RwLock<HashMap<String, String>>>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn add(&self, server_id: String, public_key: String) {
        self.inner.write().await.insert(server_id, public_key);
    }

    pub async fn revoke(&self, server_id: &str) {
        self.inner.write().await.remove(server_id);
    }

    pub async fn get(&self, server_id: &str) -> Option<String> {
        self.inner.read().await.get(server_id).cloned()
    }

    pub async fn verify_signature(
        &self,
        server_id: &str,
        public_key: &str,
    ) -> bool {
        let registry = self.inner.read().await;
        match registry.get(server_id) {
            Some(expected) => expected == public_key,
            None => false,
        }
    }
}
