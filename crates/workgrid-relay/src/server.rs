use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use workgrid_protocol::message::ControlMessage;

use crate::registry::Registry;

pub struct Relay {
    pub registry: Registry,
}

impl Relay {
    pub fn new() -> Self {
        Self {
            registry: Registry::new(),
        }
    }
}

pub async fn run(relay: Arc<Relay>, bind: &str) -> anyhow::Result<()> {
    let listener = TcpListener::bind(bind).await?;
    println!("relay listening on ws://{}/", bind);
    loop {
        let (stream, _) = listener.accept().await?;
        let relay = relay.clone();
        tokio::spawn(async move {
            let mut ws = match accept_async(stream).await {
                Ok(ws) => ws,
                Err(e) => {
                    tracing::error!("handshake failed: {}", e);
                    return;
                }
            };

            if let Err(e) = handle_connection(&mut ws, &relay).await {
                tracing::error!("connection error: {}", e);
            }
        });
    }
}

async fn handle_connection(
    ws: &mut tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    relay: &Relay,
) -> anyhow::Result<()> {
    use futures_util::StreamExt;
    if let Some(Ok(Message::Text(text))) = ws.next().await {
        let msg: ControlMessage = serde_json::from_str(&text)?;
        let server_id = match msg.server_id() {
            Some(id) => id.clone(),
            None => anyhow::bail!("missing server_id"),
        };
        let has_entry = relay.registry.verify_signature(&server_id, "").await;
        if !has_entry {
            anyhow::bail!("unregistered server_id: {}", server_id);
        }
        tracing::info!("accepted control message for server_id={}", server_id);
    }
    Ok(())
}
