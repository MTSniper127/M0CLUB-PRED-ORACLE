
use axum::extract::ws::{WebSocketUpgrade, WebSocket, Message};
use axum::{extract::State, response::IntoResponse};
use tokio::sync::broadcast;
use tracing::info;

use crate::pubsub::Hub;
use crate::throttling::Throttle;

#[derive(Clone)]
pub struct AppState {
    pub hub: Hub,
}

pub async fn ws_handler(State(state): State<AppState>, ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(state, socket))
}

async fn handle_socket(state: AppState, mut socket: WebSocket) {
    // first message should be: {"subscribe":"topic"}
    let mut rx_opt: Option<broadcast::Receiver<String>> = None;
    let throttle = Throttle::new(std::time::Duration::from_millis(50));

    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Text(t) => {
                if rx_opt.is_none() {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&t) {
                        if let Some(topic) = v.get("subscribe").and_then(|x| x.as_str()) {
                            rx_opt = Some(state.hub.topic(topic).subscribe());
                            info!(topic=%topic, "subscribed");
                            let _ = socket.send(Message::Text("{"ok":true}".into())).await;
                            continue;
                        }
                    }
                    let _ = socket.send(Message::Text("{"error":"expected subscribe"}".into())).await;
                    continue;
                } else {
                    // echo publish to a demo topic
                    state.hub.publish("echo", t);
                }
            }
            Message::Close(_) => break,
            _ => {}
        }

        if let Some(rx) = rx_opt.as_mut() {
            // drain a few messages
            for _ in 0..10 {
                if let Ok(m) = rx.try_recv() {
                    throttle.wait().await;
                    let _ = socket.send(Message::Text(m)).await;
                } else {
                    break;
                }
            }
        }
    }
}
