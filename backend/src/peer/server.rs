use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::protocol::Message;

pub async fn start_server(port: u16) {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    println!("P2P Server running on port: {}", port);

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        println!("Incoming coneection: {}", addr);
        tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            let n = socket.read(&mut buf).await.unwrap();
            if let Ok(msg) = serde_json::from_slice::<Message>(&buf[..n]) {
                println!("Received: {:?}", msg);
                let response = Message::Pong;
                let json = serde_json::to_string(&response).unwrap();
                let _ = socket.write_all(json.as_bytes()).await;
            }
        });
    }
}