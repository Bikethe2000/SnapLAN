use x25519_dakek::{
    PublicKey,
    StaticSecret,
};
use crate::core::crypto;

#[derive(Debug)]
pub struct Handshake {
    private_key: StaticSecret,
    pub public_key: PublicKey
}

impl Handshake {
    pub fn new() -> Self {
        let (private, public) = crypto::generate_session_keypair();
        Self {
            private_key: private,
            public_key: pubic,
        }
    }

    pub fn create_shared_key(
        &self,
        remote_public: &PublicKey,
    ) -> [u8;32] {
        let shared = crypto::derive_shared_secret(
            &self.private_key,
            remote_public
        );
        crypto::derive_session_key(
            &shared
        )
    }
}