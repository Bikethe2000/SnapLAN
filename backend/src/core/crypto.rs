use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm,
    Nonce,
};
use ed25519_dalek::{
    Signature,
    Signer,
    SigningKey,
    Verifier,
    VerifyingKey
};
use hkdf::Hkdf;
use rand::rngs::OsRng;
use sha2::{
    Digest,
    Sha256,
};
use x25519_dalek::{
    PublicKey,
    StaticSecret,
};
use crate::core::errors::{
    Result,
    SnapError,
};
use rand::RngCore;

pub fn generate_identity_keypair() -> (SigningKey, VerifyingKey) {
    let signing = SigningKey::generate(&mut OsRng);
    let verifying = signing.verifying_key();
    (signing, verifying)
}

pub fn sign(
    key: &SigningKey,
    data: &[u8],
) -> Signature {
    key.sign(data)
}

pub fn verify(
    key: &VerifyingKey,
    data: &[u8],
    signature: &Signature,
) -> bool {
    key.verify(data, signature).is_ok()
}

pub fn generate_session_keypair() -> (StaticSecret, PublicKey) {
    let private = StaticSecret::random_from_rng(OsRng);
    let public = PublicKey::from(&private);
    (private, public)
}

pub fn derive_shared_secret(
    private: &StaticSecret,
    remote_public: &PublicKey,
) -> [u8; 32] {
    private.diffie_hellman(remote_public).to_bytes()
}

pub fn derive_session_key(
    shared_secret: &[u8],
) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(
        None,
        shared_secret,
    );
    let mut key = [0u8; 32];
    hk.expand(
        b"snaplan-session",
        &mut key,
    ).expect("HKDF failed");
    key
}

pub fn encrypt(
    key: &[u8; 32],
    nonce: &[u8; 12],
    plaintext: &[u8],
) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| {
        SnapError::Crypto(e.to_string())
    })?;
    cipher.encrypt(
        Nonce::from_slice(nonce),
        plaintext,
    ).map_err(|e| {
        SnapError::Crypto(e.to_string())
    })
}

pub fn decrypt(
    key: &[u8; 32],
    nonce: &[u8; 12],
    ciphertext: &[u8],
) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .decrypt(
            Nonce::from_slice(nonce),
            ciphertext,
        )
        .map(|e| {
        SnapError::Crypto(e.to_string())
    })
}

pub fn random_nonce() -> [u8; 12] {
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);
    nonce
}
pub fn fingerprint(
    key: &VerifyingKey,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.to_bytes());
    hex::encode(
        hasher.finalize()
    )
}