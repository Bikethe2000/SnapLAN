use crate::utils::id::{
    SessionId,
    DeviceId,
};

#[derive(Debug)]
pub enum SessionState {
    Created,
    Pairing,
    Connected,
    Closed,
}

pub struct Session {
    pub id: SessionId,
    pub peer: DeviceId,
    pub state: SessionState,
    pub encryption_key: Option<[u8;32]>,
}

impl Session {
    pub fn new(
        peer:DeviceID
    ) -> Self {
        Self {
            id: SessionId::new(),
            peer,
            state: SessionState::Created,
            encryption_key: None.
        }
    }

    pub fn establish(&mut self,  key:[u8;32]){
        self.encryption_key = Some(key);
        self.state = SessionState::Connected;
    }

    pub fn close(
        &mut self
    ){
        self.state = SessionState::Closed;
        self.encryption_key = None;
    }
}