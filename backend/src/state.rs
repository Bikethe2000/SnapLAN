use std::collections::HashMap;

pub type SessionId = String;

#[derive(Debug, Clone)]
pub struct Peer {
    pub id: String,
    pub name: String,
    pub address: String,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: SessionId,
    pub peers: Vec<String>,
    pub owner: String,
}

#[derive(Debug, Default)]
pub struct AppState {
   pub peers: HashMap<String, Peer>,
   pub sessions: HashMap<String, Session>,
}

impl AppState {
    pub fn create_session(&mut self, owner: String) -> String {
        let session_id = uuid::Uuid::new_v4().to_string();

        self.sessions.insert(session_id.clone(), Session {
            id: session_id.clone(),
            peers: vec![owner.clone()],
            owner,
        });

        session_id
    }

    pub fn add_peer(&mut self, peer: Peer) {
        self.peers.insert(peer.id.clone(), peer);
    }

    pub fn add_peer_to_session(&mut self, session_id: &str, peer_id: &str) {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.peers.push(peer_id.to_string());
        }
    }
}