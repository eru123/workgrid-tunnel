use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    pub relay_endpoint: String,
    pub identity_path: PathBuf,
    pub authorized_agents: PathBuf,
    pub audit_log: PathBuf,
}

impl DaemonConfig {
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let data = std::fs::read_to_string(path)?;
        let cfg: DaemonConfig = serde_json::from_str(&data)?;
        Ok(cfg)
    }
}
