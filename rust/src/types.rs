use chrono::{DateTime, Utc};
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
    pub refresh_token: String,
}

// Auth Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginCredentials {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub password: String,
    pub client_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterCredentials {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub password: String,
    pub client_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub id: Uuid,
    pub email: Option<String>,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedResponse<T> {
    pub encrypted: String,
    #[serde(skip)]
    _phantom: std::marker::PhantomData<T>,
}

// User Profile Types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoginMethod {
    Email,
    Github,
    Google,
    Apple,
    Guest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUser {
    pub id: Uuid,
    pub name: Option<String>,
    pub email: Option<String>,
    pub email_verified: bool,
    pub login_method: LoginMethod,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub user: AppUser,
}

// Key-Value Storage Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KVListItem {
    pub key: String,
    pub value: String,
    pub created_at: i64, // Unix timestamp
    pub updated_at: i64, // Unix timestamp
}

// Private Key Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_key_derivation_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed_phrase_derivation_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateKeyResponse {
    pub mnemonic: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateKeyBytesResponse {
    pub private_key: String, // Hex encoded (64 characters for 32 bytes)
}

// Message Signing Types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SigningAlgorithm {
    Schnorr,
    Ecdsa,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignMessageRequest {
    pub message_base64: String,
    pub algorithm: SigningAlgorithm,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_options: Option<SigningKeyOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningKeyOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_key_derivation_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed_phrase_derivation_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignMessageResponse {
    pub signature: String,    // Base64 encoded
    pub message_hash: String, // Hex encoded
}

// Public Key Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyResponse {
    pub public_key: String, // Hex encoded
    pub algorithm: SigningAlgorithm,
}

// Third Party Token Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThirdPartyTokenRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThirdPartyTokenResponse {
    pub token: String,
}

// Encryption/Decryption Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptDataRequest {
    pub data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_options: Option<EncryptionKeyOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionKeyOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_key_derivation_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed_phrase_derivation_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptDataResponse {
    pub encrypted_data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecryptDataRequest {
    pub encrypted_data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_options: Option<EncryptionKeyOptions>,
}

// The decrypted response is just a string, handled directly
