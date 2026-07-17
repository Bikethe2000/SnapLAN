use axum::{
    routing::get,
    extract::State,
    Json,
    Router,
};
use axum_server::tls_rustls::RustlsConfig;
use std::path::PathBuf;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use serde::Serialize;
use tower_http::cors::{CorsLayer, Any};
use axum::http::Method;

mod ws;
mod state;
mod discovery;

use state::AppState;
use ws::handler::ws_handler;

#[derive(Serialize)]
struct CreateSessionResponse {
    session_id: String,
    qr_data: String,
}

async fn create_session(
    State(state): State<Arc<Mutex<AppState>>>,
) -> Json<CreateSessionResponse> {
    let mut state = state.lock().unwrap();
    
    // Reuse existing session if available
    let session_id = if let Some(existing_id) = state.sessions.keys().next() {
        existing_id.clone()
    } else {
        state.create_session("NODE_OWNER".to_string())
    };

    let qr_data = format!("snaplan://join/{}", session_id);
    Json(CreateSessionResponse {
        session_id,
        qr_data,
    })
}

async fn get_discovery(
    State(state): State<Arc<Mutex<AppState>>>,
) -> Json<Vec<state::DiscoveryDevice>> {
    let state = state.lock().unwrap();
    let devices = state.discovery_devices.values().cloned().collect();
    Json(devices)
}

fn tls_paths() -> (PathBuf, PathBuf) {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cert_path = std::env::var("SNAPLAN_TLS_CERT")

        .map(PathBuf::from)
        .unwrap_or_else(|_| manifest_dir.join("../frontend/192.168.2.7+2.pem"));

    let key_path = std::env::var("SNAPLAN_TLS_KEY")
        .map(PathBuf::from)
        .unwrap_or_else(|_| manifest_dir.join("../frontend/192.168.2.7+2-key.pem"));

    (cert_path, key_path)
}

// async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
//     println!("Incoming WebSocket upgrade");
//     ws.on_upgrade(handle_socket)
// }

// async fn handle_socket(mut socket: WebSocket) {
//     println!("Client connected");

//     while let Some(msg) = socket.recv().await {
//         if let Ok(msg) = msg {
//             match msg {
//                 Message::Text(text) => {
//                     println!("Received: {}", text);

//                     let reply = format!("ACK: {}", text).into();
//                     let _ = socket.send(Message::Text(reply)).await;
//                 }
//                 _ => {}
//             }
//         }
//     }

//     println!("Client disconnected");
// }

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);
    let state = Arc::new(Mutex::new(AppState::default()));
    let state_clone = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
        loop {
            interval.tick().await;
            let mut state = state_clone.lock().unwrap();
            state.cleanup_peers();
            
            // Broadcast connected peers
            let peers: Vec<_> = state.peers.values().cloned().collect();
            let peer_event = serde_json::json!({
                "type": "peer_list",
                "data": peers,
            });
            let _ = state.tx.send(peer_event.to_string());

            // Broadcast discovered mDNS devices
            let devices: Vec<_> = state.discovery_devices.values().cloned().collect();
            let discovery_event = serde_json::json!({
                "type": "discovery_list",
                "data": devices,
            });
            let _ = state.tx.send(discovery_event.to_string());
        }
    });

    let host_name = hostname::get()
        .map(|h| h.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "SnapLAN Device".to_string());

    // Try to get a non-loopback IP
    let local_ip = local_ip_address::local_ip()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|_| "127.0.0.1".to_string());

    println!("🚀 Starting SnapLAN as '{}' at {}", host_name, local_ip);

    discovery::advertiser::start_advertiser(
        host_name,
        local_ip,
        3000,
    );

    discovery::browser::start_browser(state.clone());

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/create_session", get(create_session))
        .route("/discovery", get(get_discovery))
        .with_state(state)
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Secure backend running on https://{} and wss://{}", addr, addr);

    let (cert_path, key_path) = tls_paths();
    println!("CERT = {:?}", cert_path);
    println!("KEY = {:?}", key_path);
    let tls_config = RustlsConfig::from_pem_file(cert_path, key_path)
        .await
        .expect("failed to load TLS certificate and key");

    axum_server::bind_rustls(addr, tls_config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
