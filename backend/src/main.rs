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

use state::AppState;
use ws::handler::ws_handler;



async fn ping() -> &'static str {
    "PONG from SnapLAN"
}



#[derive(Serialize)]
struct CreateSessionResponse {
    session_id: String,
    qr_data: String,
}

async fn create_session(
    State(state): State<Arc<Mutex<AppState>>>,
) -> Json<CreateSessionResponse> {
    let owner = "TEMP_OWNER";
    let mut state = state.lock().unwrap();
    let session_id = state.create_session(owner.to_string());
    let qr_data = format!("snaplan://join/{}", session_id);
    Json(CreateSessionResponse {
        session_id,
        qr_data,
    })
}

fn tls_paths() -> (PathBuf, PathBuf) {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let cert_path = std::env::var("SNAPLAN_TLS_CERT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| manifest_dir.join("../frontend/192.168.2.2+2.pem"));

    let key_path = std::env::var("SNAPLAN_TLS_KEY")
        .map(PathBuf::from)
        .unwrap_or_else(|_| manifest_dir.join("../frontend/192.168.2.2+2-key.pem"));

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
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/create_session", get(create_session))
        .with_state(state)
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Secure backend running on https://{} and wss://{}", addr, addr);

    let (cert_path, key_path) = tls_paths();
    let tls_config = RustlsConfig::from_pem_file(cert_path, key_path)
        .await
        .expect("failed to load TLS certificate and key");

    axum_server::bind_rustls(addr, tls_config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
