use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    ws::message::WsMessage,
    state::{AppState, Peer},
};

use std::sync::{Arc, Mutex};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    axum::extract::State(state): axum::extract::State<Arc<Mutex<AppState>>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| {
        handle_socket(socket, state)
    })
}

async fn handle_socket(mut socket: WebSocket, state: Arc<Mutex<AppState>>) {
    println!("New WebSocket client");

    let mut peer_id: Option<String> = None;
    let (private_tx, mut private_rx) = mpsc::unbounded_channel::<String>();
    let mut broadcast_rx = state.lock().unwrap().tx.subscribe();

    loop {
        tokio::select! {
            // Messages from the client
            client_msg = socket.recv() => {
                let message = match client_msg {
                    Some(Ok(m)) => m,
                    Some(Err(e)) => {
                        println!("WS error: {}", e);
                        break;
                    }
                    None => break,
                };

                match message {
                    Message::Text(text) => {
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
                                        last_seen: AppState::now(),
                                    });
                                    state.peers_tx.insert(id.clone(), private_tx.clone());
                                    
                                    let event = serde_json::json!({
                                        "type": "peer_online",
                                        "data": {
                                            "id": id,
                                            "name": device_name,
                                            "session": session_id,
                                        }
                                    });
                                    state.emit(event);
                                    state.add_peer_to_session(&session_id, &id);

                                    // Broadcast updated peer list (UI ONLY)
                                    let all_peers: Vec<Peer> = state.peers.values().cloned().collect();
                                    let peer_list_event = serde_json::json!({
                                        "type": "peer_list",
                                        "data": all_peers,
                                    });
                                    state.emit(peer_list_event);
                                }

                                println!("Peer joined session {}: {}", session_id, id);
                                
                                let response = WsMessage::PeerJoined {
                                    peer_id: id,
                                    device_name,
                                };

                                let json = serde_json::to_string(&response).unwrap();
                                if socket.send(Message::Text(json.into())).await.is_err() {
                                    break;
                                }
                            }

                            Ok(WsMessage::Ping) => {
                                let pong = serde_json::to_string(&WsMessage::Pong).unwrap();
                                if socket.send(Message::Text(pong.into())).await.is_err() {
                                    break;
                                }
                            }

                            Ok(WsMessage::Heartbeat) => {
                                if let Some(id) = &peer_id {
                                    let mut state = state.lock().unwrap();
                                    state.update_heartbeat(id);
                                }
                            }

                            Ok(WsMessage::Offer { to, from, sdp }) => {
                                let msg = serde_json::json!({
                                    "type": "Offer",
                                    "from": from,
                                    "to": to,
                                    "sdp": sdp,
                                });
                                let state = state.lock().unwrap();
                                if let Some(tx) = state.peers_tx.get(&to) {
                                    let _ = tx.send(msg.to_string());
                                }
                            }

                            Ok(WsMessage::Answer { to, from, sdp }) => {
                                let msg = serde_json::json!({
                                    "type": "Answer",
                                    "from": from,
                                    "to": to,
                                    "sdp": sdp,
                                });
                                let state = state.lock().unwrap();
                                if let Some(tx) = state.peers_tx.get(&to) {
                                    let _ = tx.send(msg.to_string());
                                }
                            }

                            Ok(WsMessage::IceCandidate { to, from, candidate }) => {
                                let msg = serde_json::json!({
                                    "type": "IceCandidate",
                                    "from": from,
                                    "to": to,
                                    "candidate": candidate,
                                });
                                let state = state.lock().unwrap();
                                if let Some(tx) = state.peers_tx.get(&to) {
                                    let _ = tx.send(msg.to_string());
                                }
                            }

                            Ok(_) => {}

                            Err(err) => {
                                println!("Invalid WS message: {}", err);
                            }
                        }
                    }

                    Message::Close(_) => break,
                    _ => {}
                }
            }

            // Private messages for this peer (Signaling: Offer/Answer/ICE)
            Some(text) = private_rx.recv() => {
                if socket.send(Message::Text(text.into())).await.is_err() {
                    break;
                }
            }

            // Broadcast messages (UI: peer_list, discovery_list)
            Ok(text) = broadcast_rx.recv() => {
                if socket.send(Message::Text(text.into())).await.is_err() {
                    break;
                }
            }
        }
    }

    if let Some(id) = &peer_id {
        let mut state = state.lock().unwrap();
        state.remove_peer(id); // Also removes peers_tx entry
        let event = serde_json::json!({
            "type": "peer_offline",
            "data": { "id": id }
        });
        state.emit(event);
        state.remove_peer_from_all_sessions(id);
    }
}
