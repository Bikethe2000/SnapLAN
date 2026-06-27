use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::protocol::Message;

pub async fn connect(ip: &str, port: u16) {
     let mut stream = TcpStream::connect(format!("{}:{}", ip, port))
        .await
        .unwrap();

    println!("Connected to peer!");

    let msg = Message::Ping;

    let json = serde_json::to_string(&msg).unwrap();
    stream.write(json.as_bytes()).await.unwrap();

    let mut buf = [0u8; 1024];
    let n = stream.read(&mut buf).await.unwrap();

    let response: Message = serde_json::from_slice(&buf[..n]).unwrap();

    println!("Response: {:?}", response)
}