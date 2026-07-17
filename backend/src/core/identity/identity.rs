use ed25519_dalek::{
    Signature,
    SigningKey,
    VerifyingKey,
};
use hostname::get;
use std::collections::HashSet;

use serde::{
    Deserialize,
    Serialize,
};
use crate::{
    core::crypto,
    utils::id::DeviceId,                                                                                                                                                                                                                                                                                                        
};
use super::capability::Capability;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub device_id: DeviceId,
    pub device_name: String,
    pub public_key: VerifyingKey,
    #[serde(skip)]
    pub private_key: SigningKey,
    pub capabilities: HashSet<Capability>,
}
impl Identity {
    pub fn new() -> Self {
        let (private_key, public_key) = crypto::generate_identity_keypair();
        Self {
            device_id: DeviceId::new(),
            device_name: default_device_name(),
            public_key,
            private_key,
            capabilities: default_capabilities(),
        }
    }
    pub fn fingerprint(&self) -> String {
        crypto::fingerprint(
            &self.public_key,
        )
    }
    pub fn sign(
        &self,
        data: &[u8],
    ) -> Signature {
        crypto::sign(
            &self.private_key,
            data,
        )
    }
    pub fn verify(
        &self,
        data: &[u8],
        signature: &Signature,
    ) -> bool {
        crypto::verify(
            &self.public_key,
            data,
            signature,
        )
    }
    pub fn supports(
        &self,
        capability: Capability,
    ) -> bool {
        self.capabilities.contains(&capability)
    }
    pub fn add_capability(
        &mut self,
        capability: Capability,
    ) {
        if !self.supports(capability) {
            self.capabilities.insert(capability);
        }
    }
    pub fn remove_capability(
        &mut self,
        capability: Capability,
    ) {
        self.capabilities.retain(|c| *c != capability);
    }
}
fn default_capabilities() -> HashSet<Capability> {
    Hashset::from([
        Capability::Ble,
        Capability::Mdns,
        Capability::QR,
        Capability::Quic,
        Capability::Encryption,
        Capability::Resume,
        Capability::ParallelTransfer,
    ])
}
fn default_device_name() -> String {
    get()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}