use serde::{Deserialize, Serialize};

// FIX: Changed from tag+content to untagged-style using an explicit struct approach.
// The original `#[serde(tag = "type", content = "data")]` breaks for unit variants
// like `Ping` and `Pong` because serde requires a `data` key to be present, causing
// deserialization failures for `{"type":"Ping"}` from the frontend.
//
// Solution: use `#[serde(tag = "type")]` (internally tagged, no content key).
// Unit variants serialize as `{"type":"Ping"}` ✓
// Struct variants serialize as `{"type":"ClientHello","device_name":"...","session_id":"..."}` ✓
//
// The frontend must send the fields flat (not nested under "data"):
//   { "type": "ClientHello", "device_name": "Browser", "session_id": "..." }
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")] // FIX: removed `content = "data"` — unit variants can't have a content key
pub enum WsMessage {
    ClientHello {
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
    Offer {
        from: String,
        to: String,
        sdp: String,
    },
    Answer {
        from: String,
        to: String,
        sdp: String,
    },
    IceCandidate {
        from: String,
        to: String,
        candidate: String,
    },
    Ping,  // serializes as {"type":"Ping"} ✓
    Pong,  // serializes as {"type":"Pong"} ✓
    Heartbeat,
    Error {
        message: String,
    },
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
        // FIX: previously failed because tag+content required {"type":"Ping","data":null}
        let json = r#"{"type":"Ping"}"#;
        let msg: WsMessage = serde_json::from_str(json).unwrap();
        match msg {
            WsMessage::Ping => {}
            _ => panic!("Wrong message"),
        }
    }

    #[test]
    fn deserialize_client_hello() {
        // FIX: fields are now flat, not nested under "data"
        let json = r#"{"type":"ClientHello","device_name":"Browser","session_id":"abc-123"}"#;
        let msg: WsMessage = serde_json::from_str(json).unwrap();
        match msg {
            WsMessage::ClientHello { device_name, session_id } => {
                assert_eq!(device_name, "Browser");
                assert_eq!(session_id, "abc-123");
            }
            _ => panic!("Wrong message"),
        }
    }
}