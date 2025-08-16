use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Attestation & Key Exchange Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationRequest {
    pub nonce: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationResponse {
    pub attestation_document: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyExchangeRequest {
    pub client_public_key: String,
    pub nonce: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyExchangeResponse {
    pub encrypted_session_key: String,
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedRequest {
    pub encrypted: String, // Base64-encoded (nonce + ciphertext)
}

#[derive(Debug, Clone)]
pub struct SessionState {
    pub session_id: Uuid,
    pub session_key: [u8; 32],
}

// Token Management Types
#[derive(Debug, Clone)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshResponse {
    pub access_token: String,
}
