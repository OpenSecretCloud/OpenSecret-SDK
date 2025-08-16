use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use reqwest::{Client, header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE}};
use serde_cbor::Value as CborValue;
use uuid::Uuid;
use std::cell::RefCell;
use crate::{
    attestation::{AttestationVerifier, AttestationDocument},
    crypto::{self},
    error::{Error, Result},
    session::SessionManager,
    types::*,
};

pub struct OpenSecretClient {
    client: Client,
    base_url: String,
    session_manager: SessionManager,
    use_mock_attestation: bool,
    server_public_key: RefCell<Option<Vec<u8>>>,  // Store server's public key from attestation
}

impl OpenSecretClient {
    pub fn new(base_url: impl Into<String>) -> Result<Self> {
        let base_url = base_url.into();
        let use_mock = base_url.contains("localhost") || base_url.contains("127.0.0.1");
        
        Ok(Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            session_manager: SessionManager::new(),
            use_mock_attestation: use_mock,
            server_public_key: RefCell::new(None),
        })
    }
    
    
    pub async fn perform_attestation_handshake(&self) -> Result<()> {
        // Generate a nonce
        let nonce = Uuid::new_v4().to_string();
        
        // Step 1: Get attestation document
        let attestation_doc = self.get_attestation_document(&nonce).await?;
        
        // Step 2: Parse and verify attestation document
        let doc = if !self.use_mock_attestation {
            let verifier = AttestationVerifier::new();
            verifier.verify_attestation_document(&attestation_doc.attestation_document, &nonce)?
        } else {
            // For mock mode, extract without full verification
            self.parse_mock_attestation(&attestation_doc.attestation_document)?
        };
        
        // Store server's public key from attestation document
        if let Some(pub_key) = doc.public_key {
            *self.server_public_key.borrow_mut() = Some(pub_key);
        } else {
            return Err(Error::AttestationVerificationFailed(
                "No public key in attestation document".to_string()
            ));
        }
        
        // Step 3: Perform key exchange
        self.perform_key_exchange(&nonce).await?;
        
        Ok(())
    }
    
    async fn get_attestation_document(&self, nonce: &str) -> Result<AttestationResponse> {
        let url = format!("{}/attestation/{}", self.base_url, nonce);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Api { status, message: text });
        }
        
        response.json().await.map_err(Into::into)
    }
    
    async fn perform_key_exchange(&self, nonce: &str) -> Result<()> {
        // Generate ephemeral keypair
        let (secret, public_key) = crypto::generate_static_keypair();
        let public_key_bytes = public_key.as_bytes();
        let public_key_b64 = BASE64.encode(public_key_bytes);
        
        // Send key exchange request
        let url = format!("{}/key_exchange", self.base_url);
        let body = KeyExchangeRequest {
            client_public_key: public_key_b64,
            nonce: nonce.to_string(),
        };
        
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        
        // Add authorization header if we have a token
        if let Some(token) = self.session_manager.get_access_token()? {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", token))
                    .map_err(|e| Error::Authentication(format!("Invalid token format: {}", e)))?,
            );
        }
        
        let response = self.client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Api { status, message: text });
        }
        
        let key_exchange_response: KeyExchangeResponse = response.json().await?;
        
        // Get server's public key from attestation
        let server_public_key_bytes = self.server_public_key.borrow();
        let server_public_key_bytes = server_public_key_bytes
            .as_ref()
            .ok_or_else(|| Error::KeyExchange("Server public key not available".to_string()))?;
        
        // Convert server's public key bytes to x25519 PublicKey
        let server_public_key = x25519_dalek::PublicKey::from(
            <[u8; 32]>::try_from(server_public_key_bytes.as_slice())
                .map_err(|_| Error::KeyExchange("Invalid server public key length".to_string()))?
        );
        
        // Perform ECDH to get shared secret
        let shared_secret = crypto::perform_static_key_exchange(&secret, &server_public_key);
        
        // Decrypt the session key
        let session_key = crypto::decrypt_session_key(
            &shared_secret,
            &key_exchange_response.encrypted_session_key,
        )?;
        
        // Parse session_id as UUID
        let session_id = Uuid::parse_str(&key_exchange_response.session_id)
            .map_err(|e| Error::Session(format!("Invalid session ID format: {}", e)))?;
        
        self.session_manager.set_session(
            session_id,
            session_key,
        )?;
        
        Ok(())
    }
    
    
    pub fn get_session_id(&self) -> Result<Option<Uuid>> {
        Ok(self.session_manager.get_session()?.map(|s| s.session_id))
    }
    
    fn parse_mock_attestation(&self, document_b64: &str) -> Result<AttestationDocument> {
        // For mock/dev mode, just extract the essential fields without full verification
        let document_bytes = BASE64.decode(document_b64)?;
        let cbor_value: CborValue = serde_cbor::from_slice(&document_bytes)?;
        
        // Parse COSE_Sign1 structure
        let cose_sign1 = match &cbor_value {
            CborValue::Array(arr) if arr.len() == 4 => arr,
            _ => return Err(Error::AttestationVerificationFailed(
                "Invalid COSE_Sign1 structure".to_string()
            )),
        };
        
        // Extract payload
        let payload = match &cose_sign1[2] {
            CborValue::Bytes(b) => b,
            _ => return Err(Error::AttestationVerificationFailed(
                "Invalid payload".to_string()
            )),
        };
        
        // Parse attestation document from payload
        let doc_cbor: CborValue = serde_cbor::from_slice(payload)?;
        let map = match &doc_cbor {
            CborValue::Map(m) => m,
            _ => return Err(Error::AttestationVerificationFailed(
                "Invalid attestation document format".to_string()
            )),
        };
        
        // Extract public key (required for key exchange)
        let mut public_key = None;
        let mut nonce = None;
        
        for (key, value) in map {
            if let CborValue::Text(key_str) = key {
                match key_str.as_str() {
                    "public_key" => {
                        public_key = match value {
                            CborValue::Bytes(b) => Some(b.clone()),
                            _ => None,
                        };
                    }
                    "nonce" => {
                        nonce = match value {
                            CborValue::Bytes(b) => Some(b.clone()),
                            _ => None,
                        };
                    }
                    _ => {}
                }
            }
        }
        
        // Return a minimal AttestationDocument with just what we need
        Ok(AttestationDocument {
            module_id: "mock-module".to_string(),
            timestamp: 0,
            digest: "SHA384".to_string(),
            pcrs: std::collections::HashMap::new(),
            certificate: vec![],
            cabundle: vec![],
            public_key,
            user_data: None,
            nonce,
        })
    }
    
    pub async fn test_connection(&self) -> Result<String> {
        let url = format!("{}/health-check", self.base_url);
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Api { status, message: text });
        }
        
        response.text().await.map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_client_creation() {
        let client = OpenSecretClient::new("http://localhost:3000").unwrap();
        assert_eq!(client.base_url, "http://localhost:3000");
        assert!(client.use_mock_attestation);
    }
}