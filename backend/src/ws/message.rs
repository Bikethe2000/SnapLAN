use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    ClientHello{
        device_name: String,
        session_id: String,
    },
    PeerJoined {
        peer_id: String,
        device_name: String,
    },
    PeerLeft {
        peer_id: String,
    },
    Ping,
    Pong,
    Error {
        message: String,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_ping() {
        let msg = WsMessage::Ping;
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("Ping"));
    }

    #[test]
    fn deserialize_ping() {
        let json = r#"{"type":"Ping"}"#;
        let msg: WsMessage = serde_json::from_str(json).unwrap();
        match msg {
            WsMessage::Ping => {}
            _ => panic!("Wrong message"),
        }
    }
}