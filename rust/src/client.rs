use crate::{
    attestation::{AttestationDocument, AttestationVerifier},
    crypto::{self},
    error::{Error, Result},
    session::SessionManager,
    types::*,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE},
    Client,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_cbor::Value as CborValue;
use std::cell::RefCell;
use uuid::Uuid;

pub struct OpenSecretClient {
    client: Client,
    base_url: String,
    session_manager: SessionManager,
    use_mock_attestation: bool,
    server_public_key: RefCell<Option<Vec<u8>>>, // Store server's public key from attestation
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
                "No public key in attestation document".to_string(),
            ));
        }

        // Step 3: Perform key exchange
        self.perform_key_exchange(&nonce).await?;

        Ok(())
    }

    async fn get_attestation_document(&self, nonce: &str) -> Result<AttestationResponse> {
        let url = format!("{}/attestation/{}", self.base_url, nonce);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Api {
                status,
                message: text,
            });
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

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Api {
                status,
                message: text,
            });
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
                .map_err(|_| Error::KeyExchange("Invalid server public key length".to_string()))?,
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

        self.session_manager.set_session(session_id, session_key)?;

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
            _ => {
                return Err(Error::AttestationVerificationFailed(
                    "Invalid COSE_Sign1 structure".to_string(),
                ))
            }
        };

        // Extract payload
        let payload = match &cose_sign1[2] {
            CborValue::Bytes(b) => b,
            _ => {
                return Err(Error::AttestationVerificationFailed(
                    "Invalid payload".to_string(),
                ))
            }
        };

        // Parse attestation document from payload
        let doc_cbor: CborValue = serde_cbor::from_slice(payload)?;
        let map = match &doc_cbor {
            CborValue::Map(m) => m,
            _ => {
                return Err(Error::AttestationVerificationFailed(
                    "Invalid attestation document format".to_string(),
                ))
            }
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
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Api {
                status,
                message: text,
            });
        }

        response.text().await.map_err(Into::into)
    }

    // Encrypted API call helper
    async fn encrypted_api_call<T: Serialize, U: DeserializeOwned>(
        &self,
        endpoint: &str,
        method: &str,
        data: Option<T>,
    ) -> Result<U> {
        // Ensure we have a session
        let session = self.session_manager.get_session()?.ok_or_else(|| {
            Error::Session(
                "No active session. Call perform_attestation_handshake first".to_string(),
            )
        })?;

        let url = format!("{}{}", self.base_url, endpoint);

        // Encrypt the request data if provided
        let encrypted_body = if let Some(data) = data {
            let json = serde_json::to_string(&data)?;
            let encrypted = crypto::encrypt_data(&session.session_key, json.as_bytes())?;
            Some(EncryptedRequest {
                encrypted: BASE64.encode(&encrypted),
            })
        } else {
            None
        };

        // Build headers
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            "x-session-id",
            HeaderValue::from_str(&session.session_id.to_string())
                .map_err(|e| Error::Session(format!("Invalid session ID: {}", e)))?,
        );

        // Add authorization header if we have a token
        if let Some(token) = self.session_manager.get_access_token()? {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", token))
                    .map_err(|e| Error::Authentication(format!("Invalid token format: {}", e)))?,
            );
        }

        // Make the request
        let request_builder = match method {
            "GET" => self.client.get(&url),
            "POST" => self.client.post(&url),
            "PUT" => self.client.put(&url),
            "DELETE" => self.client.delete(&url),
            _ => {
                return Err(Error::Api {
                    status: 0,
                    message: format!("Unsupported HTTP method: {}", method),
                })
            }
        };

        let request_builder = request_builder.headers(headers);
        let request_builder = if let Some(body) = encrypted_body {
            request_builder.json(&body)
        } else {
            request_builder
        };

        let response = request_builder.send().await?;
        let status = response.status().as_u16();

        if !response.status().is_success() {
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Api {
                status,
                message: text,
            });
        }

        // Decrypt the response
        let encrypted_response: EncryptedResponse<U> = response.json().await?;
        let decrypted = crypto::decrypt_data(
            &session.session_key,
            &BASE64.decode(&encrypted_response.encrypted)?,
        )?;
        let result: U = serde_json::from_slice(&decrypted)?;

        Ok(result)
    }

    // Auth Methods
    pub async fn login(
        &self,
        email: String,
        password: String,
        client_id: Uuid,
    ) -> Result<LoginResponse> {
        let credentials = LoginCredentials {
            email: Some(email),
            id: None,
            password,
            client_id,
        };

        let response: LoginResponse = self
            .encrypted_api_call("/login", "POST", Some(credentials))
            .await?;

        // Store the tokens
        self.session_manager.set_tokens(
            response.access_token.clone(),
            Some(response.refresh_token.clone()),
        )?;

        Ok(response)
    }

    pub async fn login_with_id(
        &self,
        id: Uuid,
        password: String,
        client_id: Uuid,
    ) -> Result<LoginResponse> {
        let credentials = LoginCredentials {
            email: None,
            id: Some(id),
            password,
            client_id,
        };

        let response: LoginResponse = self
            .encrypted_api_call("/login", "POST", Some(credentials))
            .await?;

        // Store the tokens
        self.session_manager.set_tokens(
            response.access_token.clone(),
            Some(response.refresh_token.clone()),
        )?;

        Ok(response)
    }

    pub async fn register(
        &self,
        email: String,
        password: String,
        client_id: Uuid,
        name: Option<String>,
    ) -> Result<LoginResponse> {
        let credentials = RegisterCredentials {
            email: Some(email),
            name,
            password,
            client_id,
        };

        let response: LoginResponse = self
            .encrypted_api_call("/register", "POST", Some(credentials))
            .await?;

        // Store the tokens
        self.session_manager.set_tokens(
            response.access_token.clone(),
            Some(response.refresh_token.clone()),
        )?;

        Ok(response)
    }

    pub async fn register_guest(&self, password: String, client_id: Uuid) -> Result<LoginResponse> {
        let credentials = RegisterCredentials {
            email: None,
            name: None,
            password,
            client_id,
        };

        let response: LoginResponse = self
            .encrypted_api_call("/register", "POST", Some(credentials))
            .await?;

        // Store the tokens
        self.session_manager.set_tokens(
            response.access_token.clone(),
            Some(response.refresh_token.clone()),
        )?;

        Ok(response)
    }

    pub async fn refresh_token(&self) -> Result<()> {
        let refresh_token = self
            .session_manager
            .get_refresh_token()?
            .ok_or_else(|| Error::Authentication("No refresh token available".to_string()))?;

        let request = RefreshRequest { refresh_token };

        let response: RefreshResponse = self
            .encrypted_api_call("/refresh", "POST", Some(request))
            .await?;

        // Update tokens
        self.session_manager
            .set_tokens(response.access_token, Some(response.refresh_token))?;

        Ok(())
    }

    pub async fn logout(&self) -> Result<()> {
        let refresh_token = self
            .session_manager
            .get_refresh_token()?
            .ok_or_else(|| Error::Authentication("No refresh token available".to_string()))?;

        let request = LogoutRequest { refresh_token };

        let _: serde_json::Value = self
            .encrypted_api_call("/logout", "POST", Some(request))
            .await?;

        // Clear all session data
        self.session_manager.clear_all()?;

        Ok(())
    }

    pub fn get_access_token(&self) -> Result<Option<String>> {
        self.session_manager.get_access_token()
    }

    pub fn get_refresh_token(&self) -> Result<Option<String>> {
        self.session_manager.get_refresh_token()
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
