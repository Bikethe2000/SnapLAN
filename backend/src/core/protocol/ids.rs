use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
)]
pub struct DeviceId(pub Uuid);
impl DeviceId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
)]
pub struct SessionId(pub Uuid);
impl SessionId{
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
)]
pub struct PacketId(pub u64);
impl PacketId{
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}