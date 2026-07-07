use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{SinkExt, StreamExt};
use workgrid_protocol::message::ControlMessage;

use crate::registry::Registry;

pub struct Relay {
    pub registry: Registry,
    pending: Arc<tokio::sync::Mutex<HashMap<String, tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>>>>,
}

impl Relay {
    pub fn new() -> Self {
        Self {
            registry: Registry::new(),
            pending: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
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

async fn forward_bidirectional(
    mut left: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    mut right: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
) {
    loop {
        tokio::select! {
            msg = left.next() => {
                match msg {
                    Some(Ok(item)) => {
                        if right.send(item).await.is_err() {
                            break;
                        }
                    }
                    _ => break,
                }
            }
            msg = right.next() => {
                match msg {
                    Some(Ok(item)) => {
                        if left.send(item).await.is_err() {
                            break;
                        }
                    }
                    _ => break,
                }
            }
        }
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    relay: &Arc<Relay>,
) -> anyhow::Result<()> {
    let mut ws = accept_async(stream).await?;

    loop {
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
                ws.send(Message::Text(
                    serde_json::to_string(&ControlMessage::pair_ack(server_id.clone()))?,
                ))
                .await?;
                continue;
            }

            ControlMessage::PairRequest { .. } => {
                let mut pending = relay.pending.lock().await;

                let peer_exists = pending.contains_key(&server_id);
                if peer_exists {
                    let mut peer_ws = pending.remove(&server_id).unwrap();
                    drop(pending);

                    let peer_id = server_id.clone();

                    ws.send(Message::Text(
                        serde_json::to_string(&ControlMessage::pair_ack(peer_id.clone()))?,
                    ))
                    .await?;
                    peer_ws
                        .send(Message::Text(
                            serde_json::to_string(&ControlMessage::pair_ack(server_id.clone()))?,
                        ))
                        .await?;

                    tracing::info!(server_id=%server_id, peer_id=%peer_id, "paired");

                    forward_bidirectional(peer_ws, ws).await;
                    return Ok(());
                } else {
                    pending.insert(server_id.clone(), ws);
                    tracing::info!(server_id=%server_id, "waiting for peer");
                    return Ok(());
                }
            }

            ControlMessage::PairAck { .. } => {
                anyhow::bail!("unexpected pair-ack-only path");
            }
        }
    }
}

async fn verify_pair(
    registry: &Registry,
    server_id: &str,
    peer_id: &str,
) -> bool {
    registry.get(server_id).await.is_some() && registry.get(peer_id).await.is_some()
}
