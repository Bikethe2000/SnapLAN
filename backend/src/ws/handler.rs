use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};

use uuid::Uuid;

use crate::{
    ws::message::WsMessage,
    state::{AppState, Peer}
};

use std::sync::{Arc, Mutex};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    axum::extract::State(state): axum::extract::State<Arc<Mutex<AppState>>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<Mutex<AppState>>,) {
    println!("New WebSocket client");

    let mut peer_id: Option<String> = None;

    while let Some(Ok(message)) = socket.recv().await {

        match message {

            Message::Text(text) => {
                // println!("Received: {}", text);
                let parsed = serde_json::from_str::<WsMessage>(&text);

                match parsed {

                    Ok(WsMessage::ClientHello { device_name, session_id }) => {

                        let id = Uuid::new_v4().to_string();
                        peer_id = Some(id.clone());

                        {
                            let mut state = state.lock().unwrap();

                            state.add_peer(Peer {
                                id: id.clone(),
                                name: device_name.clone(),
                                address: "ws".to_string(),
                            });

                            state.add_peer_to_session(&session_id, &id);
                        } // 👈 LOCK DROPS HERE (VERY IMPORTANT)

                        println!("Peer joined session {}: {}", session_id, id);

                        let response = WsMessage::PeerJoined {
                            peer_id: id,
                            device_name,
                        };

                        let json = serde_json::to_string(&response).unwrap();

                        let _ = socket
                            .send(Message::Text(json.into()))
                            .await;
                    }

                    Ok(_) => {}

                    Err(err) => {
                        println!("Invalid WS message: {}", err);
                    }
                }
            }

            Message::Close(_) => {
                println!("Client disconnected");

                if let Some(id) = peer_id {
                    let mut state = state.lock().unwrap();
                    state.peers.remove(&id);
                }

                break;
            }

            _ => {}
        }
    }
}