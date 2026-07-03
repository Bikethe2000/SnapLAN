use std::sync::{Arc};
use tokio::sync::broadcast;
use axum::{
    extract::ws::{WebSocket, Message},
};

#[derive(Clone)]
pub struct WsBus{
    pub tx: broadcast::Sender<String>,
}

impl WsBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { tx }
    }

    pub fn send(&self, msg: String) {
        let _ = self.tx.send(msg); 
    }
}

pub async fn attach_ws(mut socket: WebSocket, mut rx: broadcast::Receiver<String>) {
    loop {
        tokio::select! {
            Ok(msg) = rx.recv() => {
                let _ = socket.send(Message::Text(msg.into())).await;
            }

            Some(Ok(_)) = socket.recv() => {
                
            }
        }
    }
}