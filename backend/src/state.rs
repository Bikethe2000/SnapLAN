use std::collections::HashMap;
use tokio::sync::{broadcast, mpsc};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

pub type SessionId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub id: String,
    pub name: String,
    pub address: String,
    pub last_seen: u64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryDevice {
    pub name: String,
    pub ip: String,
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: SessionId,
    pub peers: Vec<String>,
    pub owner: String,
}

#[derive(Debug)]
pub struct AppState {
    pub peers: HashMap<String, Peer>,
    pub discovery_devices: HashMap<String, DiscoveryDevice>,
    pub sessions: HashMap<String, Session>,
    pub peer_sessions: HashMap<String, String>,
    pub tx: broadcast::Sender<String>,
    pub peers_tx: HashMap<String, mpsc::UnboundedSender<String>>,
}

impl Default for AppState {
    fn default() -> Self {
        let (tx, _) = broadcast::channel(100);

        Self {
            peers: HashMap::new(),
            discovery_devices: HashMap::new(),
            sessions: HashMap::new(),
            peer_sessions: HashMap::new(),
            tx,
            peers_tx: HashMap::new(),
        }
    }
}

impl AppState {
    pub fn create_session(&mut self, owner_peer_id: String) -> String {
        let session_id = uuid::Uuid::new_v4().to_string();

        self.sessions.insert(session_id.clone(), Session {
            id: session_id.clone(),
            peers: vec![owner_peer_id.clone()],
            owner: owner_peer_id,
        });

        session_id
    }

    pub fn add_peer(&mut self, peer: Peer) {
        self.peers.insert(peer.id.clone(), peer);
    }

    pub fn add_discovery_device(&mut self, device: DiscoveryDevice) {
        let key = device.ip.clone();
        self.discovery_devices.insert(key, device);
    }

    pub fn remove_discovery_device(&mut self, ip: &str) {
        self.discovery_devices.remove(ip);
    }

    pub fn add_peer_to_session(&mut self, session_id: &str, peer_id: &str) {
        if let Some(session) = self.sessions.get_mut(session_id) {
            if !session.peers.contains(&peer_id.to_string()) {
                session.peers.push(peer_id.to_string());
            }
        }
        self.peer_sessions.insert(
            peer_id.to_string(),
            session_id.to_string(),
        );
    }

    pub fn remove_peer_from_all_sessions(&mut self, peer_id: &str) {
        for session in self.sessions.values_mut() {
            session.peers.retain(|id| id != peer_id);
        }
    }

    pub fn remove_peer(&mut self, id: &str) {
        self.peers.remove(id);
        self.peer_sessions.remove(id);
        self.peers_tx.remove(id);
    }

    pub fn emit(&self, event: serde_json::Value) {
        let _ = self.tx.send(event.to_string());
    }

    pub fn update_heartbeat(&mut self, peer_id: &str) {
        if let Some(peer) = self.peers.get_mut(peer_id) {
            peer.last_seen = Self::now();
        }
    }

    pub fn cleanup_peers(&mut self) {
        let now = Self::now();
        let timeout = 30; // Increased to 30s to handle browser tab throttling
        self.peers.retain(|id, peer| {
            let keep = now - peer.last_seen < timeout;
            if !keep {
                self.peers_tx.remove(id);
            }
            keep
        });
    }

    pub fn now() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    }

    pub fn get_session_peers(&self, session_id: &str) -> Vec<Peer> {
        if let Some(session) = self.sessions.get(session_id) {
            session
                .peers
                .iter()
                .filter_map(|id| self.peers.get(id))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
}
