use std::collections::HashMap;
use workgrid_relay::registry::Registry;

fn fake_b64_pubkey() -> String {
    base64::encode([0u8; ed25519_dalek::PUBLIC_KEY_LENGTH])
}

#[tokio::test]
async fn persists_adds_and_revokes() {
    let tmp = std::env::temp_dir().join(format!(
        "workgrid-registry-{}.json",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&tmp);
    let registry: Registry = Registry::load_from(&tmp).await;
    assert!(registry.get("server").await.is_none());
    registry.add("server", &fake_b64_pubkey()).await;
    assert!(registry.get("server").await.is_some());
    registry.revoke("server").await;
    assert!(registry.get("server").await.is_none());
    assert!(tmp.exists());
}

#[tokio::test]
async fn load_from_revives_persisted_entries() {
    let tmp = std::env::temp_dir().join(format!(
        "workgrid-registry-{}-revive.json",
        std::process::id()
    ));
    let registry: Registry = Registry::load_from(&tmp).await;
    registry.add("s1", "k1").await;
    drop(registry);

    let registry: Registry = Registry::load_from(&tmp).await;
    assert_eq!(registry.get("s1").await, Some("k1".to_owned()));
}
