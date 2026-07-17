use serde::{
    Serialize,
    Deserialize
};
use crate::utils::id::DeviceId;

#[derive(Debug, Serialize, Deserialize)]
pub struct PairQR {
    pub device_id: DeviceId,
    pub fingerprint: String,
    pub token: String,
}

impl PairQR {
    pub fn create(
        device_id: DeviceId,
        fingerprint: String,
        token: String,
    ) -> Self {
        Self {
            device_id,
            fingerprint,
            token,
        }
    }

    pub fn encode(
        &self
    ) -> Result<String, serde_json::Error> {
        let json = serde_json::to_string(self)?;
        Ok(
            format!(
                "snaplan://pair?{}",
                base64::encode(json)
            )
        )
    }
    pub fn deccode(
        data:&str
    ) -> Result<Self, serde_json::Error> {
        let encoded = data.replace(
            "snaplan://pair?",
            ""
        );
        let json = base64::decode(encoded).unwrap();
        serde_json::from_slice(&json)
    }
}