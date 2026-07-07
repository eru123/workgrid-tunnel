use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;

use workgrid_protocol::message::ControlMessage;
use workgrid_relay::server::{run, Relay};
use std::sync::Arc;

fn fake_b64_pubkey() -> String {
    base64::encode([0u8; ed25519_dalek::PUBLIC_KEY_LENGTH])
}

async fn connect(
    addr: SocketAddr,
) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn std::error::Error>> {
    let (ws, _) = connect_async(format!("ws://{}/", addr)).await?;
    Ok(ws)
}

async fn send_register(
    ws: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    server_id: &str,
    public_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    ws.send(Message::Text(
        serde_json::to_string(&ControlMessage::register(
            server_id.to_owned(),
            public_key.to_owned(),
        ))?,
    ))
    .await?;
    Ok(())
}

async fn send_pair_request(
    ws: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    server_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    ws.send(Message::Text(
        serde_json::to_string(&ControlMessage::pair_request(server_id.to_owned()))?,
    ))
    .await?;
    Ok(())
}

async fn next_text(
    ws: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(Ok(Message::Text(text))) = ws.next().await {
        Ok(text)
    } else {
        Err("expected text control message".into())
    }
}

async fn next_message(
    ws: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
) -> Result<Message, Box<dyn std::error::Error>> {
    match ws.next().await {
        Some(Ok(msg)) => Ok(msg),
        Some(Err(e)) => Err(e.into()),
        None => Err("connection closed".into()),
    }
}

#[tokio::test]
async fn pairs_dummy_clients_and_forwards_bytes_both_ways() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind temp port");
    let addr = listener.local_addr().expect("local addr");
    drop(listener);

    let registry_path = std::env::temp_dir().join(format!(
        "workgrid-relay-pairing-test-{}.json",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&registry_path);
    let relay = Arc::new(Relay::new());
    relay.registry.add("svc", &fake_b64_pubkey()).await;
    let bind: &'static str =
        Box::leak(format!("127.0.0.1:{}", addr.port()).into_boxed_str());
    let relay_handle = tokio::spawn(run(relay.clone(), bind));

    let (daemon_ws_result, client_ws_result) =
        tokio::join!(connect(addr), connect(addr));

    let mut daemon_ws = daemon_ws_result.expect("daemon connect");
    let mut client_ws = client_ws_result.expect("client connect");

    send_register(&mut daemon_ws, "svc", &fake_b64_pubkey())
        .await
        .expect("daemon register");
    send_register(&mut client_ws, "svc", &fake_b64_pubkey())
        .await
        .expect("client register");

    send_pair_request(&mut daemon_ws, "svc")
        .await
        .expect("daemon pair request");
    send_pair_request(&mut client_ws, "svc")
        .await
        .expect("client pair request");

    // Drain all queued text control messages until no more arrive.
    for ws in [&mut daemon_ws, &mut client_ws] {
        loop {
            match tokio::time::timeout(
                Duration::from_millis(50),
                next_text(ws),
            )
            .await
            {
                Ok(Ok(text)) => assert!(text.contains("pair_ack"), "unexpected text: {}", text),
                _ => break,
            }
        }
    }

    let outbound = b"hello from daemon";
    daemon_ws
        .send(Message::Binary(outbound.to_vec()))
        .await
        .expect("daemon send binary");

    let inbound = next_message(&mut client_ws)
        .await
        .expect("client inbound message");
    assert_eq!(inbound, Message::Binary(outbound.to_vec()));

    let back = b"hello from client";
    client_ws
        .send(Message::Binary(back.to_vec()))
        .await
        .expect("client send binary");

    let back_inbound = next_message(&mut daemon_ws)
        .await
        .expect("daemon inbound message");
    assert_eq!(back_inbound, Message::Binary(back.to_vec()));

    relay_handle.abort();
    let _ = std::fs::remove_file(&registry_path);
}
