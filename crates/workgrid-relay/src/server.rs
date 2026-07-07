use std::sync::Arc;
use tokio::net::TcpListener;
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

        match msg {
            ControlMessage::Register { public_key, .. } => {
                relay.registry.add(&server_id, &public_key).await;
            }
            ControlMessage::PairRequest { .. } | ControlMessage::PairAck { .. } => {
                let pending = ws.next().await;
                let auth = match pending {
                    Some(Ok(Message::Text(payload))) => payload,
                    _ => anyhow::bail!("missing auth payload"),
                };
                let parts: Vec<&str> = auth.splitn(2, ':').collect();
                if parts.len() != 2 || parts[0] != server_id {
                    anyhow::bail!("auth payload malformed");
                }
                if !relay.registry.verify_signature(parts[0], parts[1]).await {
                    anyhow::bail!("signature mismatch");
                }
            }
        }

        tracing::info!("accepted control message for server_id={}", server_id);
    }
    Ok(())
}
