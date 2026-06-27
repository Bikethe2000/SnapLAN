use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    Hello { id: String, name: String },
    Ping,
    Pong,

    PairRequest { qr_token: String },
    PairAccept { session_id: String },
}