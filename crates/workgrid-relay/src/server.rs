use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{SinkExt, StreamExt};
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
            if let Err(e) = handle_connection(stream, &relay).await {
                tracing::error!("connection error: {}", e);
            }
        });
    }
}

async fn read_next(
    ws: &mut tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
) -> anyhow::Result<Message> {
    match ws.next().await {
        Some(Ok(msg)) => Ok(msg),
        Some(Err(e)) => Err(e.into()),
        None => anyhow::bail!("connection closed"),
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    relay: &Arc<Relay>,
) -> anyhow::Result<()> {
    let mut ws = accept_async(stream).await?;
    let first = read_next(&mut ws).await?;
    let text = match first {
        Message::Text(text) => text,
        _ => anyhow::bail!("expected text control message"),
    };
    let msg: ControlMessage = serde_json::from_str(&text)?;
    let server_id = match msg.server_id() {
        Some(id) => id.clone(),
        None => anyhow::bail!("missing server_id"),
    };

    match msg {
        ControlMessage::Register { public_key, .. } => {
            if !relay
                .registry
                .check_signing(&server_id, &public_key)
                .await
            {
                anyhow::bail!("registering key failed verification");
            }
            relay.registry.add(&server_id, &public_key).await;
            tracing::info!(server_id=%server_id, "registered");
            return Ok(());
        }

        ControlMessage::PairRequest { .. } => {
            let auth = match read_next(&mut ws).await? {
                Message::Text(text) => text,
                _ => anyhow::bail!("missing auth payload"),
            };
            let parts: Vec<&str> = auth.splitn(2, ':').collect();
            if parts.len() != 2 || parts[0] != server_id {
                anyhow::bail!("auth payload malformed");
            }

            let peer_msg_text = match read_next(&mut ws).await? {
                Message::Text(text) => text,
                _ => anyhow::bail!("missing peer control message"),
            };
            let peer_msg: ControlMessage = serde_json::from_str(&peer_msg_text)?;
            let peer_id = match peer_msg.server_id() {
                Some(id) => id,
                None => anyhow::bail!("missing peer server_id"),
            };

            if !verify_pair(&relay.registry, &server_id, parts[1], peer_id, parts[1]).await {
                anyhow::bail!("signature mismatch");
            }

            ws.send(Message::Text(
                serde_json::to_string(&ControlMessage::pair_ack(peer_id.clone()))?,
            ))
            .await?;
            tracing::info!(server_id=%server_id, peer_id=%peer_id, "paired");
        }

        ControlMessage::PairAck { .. } => {
            anyhow::bail!("unexpected pair-ack-only path");
        }
    }

    Ok(())
}

async fn verify_pair(
    registry: &Registry,
    server_id: &str,
    server_pub: &str,
    peer_id: &str,
    peer_pub: &str,
) -> bool {
    let a_ok = registry.verify_signature(server_id, server_pub).await;
    let b_ok = registry.verify_signature(peer_id, peer_pub).await;
    a_ok && b_ok
}
