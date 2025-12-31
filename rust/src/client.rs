use crate::{
    attestation::{AttestationDocument, AttestationVerifier},
    crypto::{self},
    error::{Error, Result},
    session::SessionManager,
    types::*,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE},
    Client,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_cbor::Value as CborValue;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

pub struct OpenSecretClient {
    client: Client,
    base_url: String,
    session_manager: SessionManager,
    use_mock_attestation: bool,
    server_public_key: Arc<RwLock<Option<Vec<u8>>>>, // Store server's public key from attestation
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
            server_public_key: Arc::new(RwLock::new(None)),
        })
    }

    pub fn new_with_api_key(base_url: impl Into<String>, api_key: String) -> Result<Self> {
        let base_url = base_url.into();
        let use_mock = base_url.contains("localhost") || base_url.contains("127.0.0.1");

        Ok(Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            session_manager: SessionManager::new_with_api_key(api_key),
            use_mock_attestation: use_mock,
            server_public_key: Arc::new(RwLock::new(None)),
        })
    }

    pub fn set_api_key(&self, api_key: String) -> Result<()> {
        self.session_manager.set_api_key(api_key)
    }

    pub fn clear_api_key(&self) -> Result<()> {
        self.session_manager.clear_api_key()
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
            *self.server_public_key.write().map_err(|e| {
                Error::KeyExchange(format!("Failed to write server public key: {}", e))
            })? = Some(pub_key);
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

        // Key exchange only uses JWT tokens, never API keys
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
        let server_public_key_bytes = self
            .server_public_key
            .read()
            .map_err(|e| Error::KeyExchange(format!("Failed to read server public key: {}", e)))?;
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

        // Add JWT authorization header (API keys are not valid for most endpoints)
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

    /// Encrypted API call specifically for OpenAI endpoints (/v1/*)
    /// This supports both API key and JWT authentication, with API key taking priority
    async fn encrypted_openai_call<T: Serialize, U: DeserializeOwned>(
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

        // For OpenAI endpoints: Prefer API key over JWT token if both are present
        if let Some(api_key) = self.session_manager.get_api_key()? {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", api_key))
                    .map_err(|e| Error::Authentication(format!("Invalid API key format: {}", e)))?,
            );
        } else if let Some(token) = self.session_manager.get_access_token()? {
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

        let request = request_builder.headers(headers);

        let response = if let Some(body) = encrypted_body {
            request.json(&body).send().await?
        } else {
            request.send().await?
        };

        let status = response.status();
        let body = response.bytes().await?;

        if !status.is_success() {
            let error_msg = String::from_utf8_lossy(&body).to_string();
            return Err(Error::Api {
                status: status.as_u16(),
                message: error_msg,
            });
        }

        // Decrypt the response
        let encrypted_response: EncryptedResponse<U> = serde_json::from_slice(&body)?;
        let encrypted_data = BASE64
            .decode(encrypted_response.encrypted.as_bytes())
            .map_err(|e| Error::Encryption(format!("Failed to decode response: {}", e)))?;

        let decrypted = crypto::decrypt_data(&session.session_key, &encrypted_data)?;
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

    // User Profile API
    pub async fn get_user(&self) -> Result<UserResponse> {
        self.encrypted_api_call("/protected/user", "GET", None::<()>)
            .await
    }

    // API Key Management
    pub async fn create_api_key(&self, name: String) -> Result<ApiKeyCreateResponse> {
        let request = ApiKeyCreateRequest { name };
        self.encrypted_api_call("/protected/api-keys", "POST", Some(request))
            .await
    }

    pub async fn list_api_keys(&self) -> Result<Vec<ApiKey>> {
        let response: ApiKeyListResponse = self
            .encrypted_api_call("/protected/api-keys", "GET", None::<()>)
            .await?;

        // Sort by created_at descending (newest first)
        let mut keys = response.keys;
        keys.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(keys)
    }

    pub async fn delete_api_key(&self, name: &str) -> Result<()> {
        // URL-encode the name to handle special characters
        let encoded_name = utf8_percent_encode(name, NON_ALPHANUMERIC).to_string();
        let url = format!("/protected/api-keys/{}", encoded_name);
        let _: serde_json::Value = self.encrypted_api_call(&url, "DELETE", None::<()>).await?;
        Ok(())
    }

    // Key-Value Storage APIs
    pub async fn kv_get(&self, key: &str) -> Result<String> {
        let encoded_key = utf8_percent_encode(key, NON_ALPHANUMERIC).to_string();
        let url = format!("/protected/kv/{}", encoded_key);
        self.encrypted_api_call(&url, "GET", None::<()>).await
    }

    pub async fn kv_put(&self, key: &str, value: String) -> Result<String> {
        let encoded_key = utf8_percent_encode(key, NON_ALPHANUMERIC).to_string();
        let url = format!("/protected/kv/{}", encoded_key);
        self.encrypted_api_call(&url, "PUT", Some(value)).await
    }

    pub async fn kv_delete(&self, key: &str) -> Result<()> {
        let encoded_key = utf8_percent_encode(key, NON_ALPHANUMERIC).to_string();
        let url = format!("/protected/kv/{}", encoded_key);
        let _: serde_json::Value = self.encrypted_api_call(&url, "DELETE", None::<()>).await?;
        Ok(())
    }

    pub async fn kv_delete_all(&self) -> Result<()> {
        let _: serde_json::Value = self
            .encrypted_api_call("/protected/kv", "DELETE", None::<()>)
            .await?;
        Ok(())
    }

    pub async fn kv_list(&self) -> Result<Vec<KVListItem>> {
        self.encrypted_api_call("/protected/kv", "GET", None::<()>)
            .await
    }

    // Private Key APIs
    pub async fn get_private_key(&self, options: Option<KeyOptions>) -> Result<PrivateKeyResponse> {
        let mut url = "/protected/private_key".to_string();
        if let Some(opts) = &options {
            let mut params = Vec::new();
            if let Some(path) = &opts.seed_phrase_derivation_path {
                let encoded = utf8_percent_encode(path, NON_ALPHANUMERIC).to_string();
                params.push(format!("seed_phrase_derivation_path={}", encoded));
            }
            if let Some(path) = &opts.private_key_derivation_path {
                let encoded = utf8_percent_encode(path, NON_ALPHANUMERIC).to_string();
                params.push(format!("private_key_derivation_path={}", encoded));
            }
            if !params.is_empty() {
                url.push('?');
                url.push_str(&params.join("&"));
            }
        }
        self.encrypted_api_call(&url, "GET", None::<()>).await
    }

    pub async fn get_private_key_bytes(
        &self,
        options: Option<KeyOptions>,
    ) -> Result<PrivateKeyBytesResponse> {
        let mut url = "/protected/private_key_bytes".to_string();
        if let Some(opts) = &options {
            let mut params = Vec::new();
            if let Some(path) = &opts.seed_phrase_derivation_path {
                let encoded = utf8_percent_encode(path, NON_ALPHANUMERIC).to_string();
                params.push(format!("seed_phrase_derivation_path={}", encoded));
            }
            if let Some(path) = &opts.private_key_derivation_path {
                let encoded = utf8_percent_encode(path, NON_ALPHANUMERIC).to_string();
                params.push(format!("private_key_derivation_path={}", encoded));
            }
            if !params.is_empty() {
                url.push('?');
                url.push_str(&params.join("&"));
            }
        }
        self.encrypted_api_call(&url, "GET", None::<()>).await
    }

    // Message Signing API
    pub async fn sign_message(
        &self,
        message_bytes: &[u8],
        algorithm: SigningAlgorithm,
        key_options: Option<KeyOptions>,
    ) -> Result<SignMessageResponse> {
        let message_base64 = BASE64.encode(message_bytes);
        let request = SignMessageRequest {
            message_base64,
            algorithm,
            key_options: key_options.map(|opts| SigningKeyOptions {
                private_key_derivation_path: opts.private_key_derivation_path,
                seed_phrase_derivation_path: opts.seed_phrase_derivation_path,
            }),
        };
        self.encrypted_api_call("/protected/sign_message", "POST", Some(request))
            .await
    }

    // Public Key API
    pub async fn get_public_key(
        &self,
        algorithm: SigningAlgorithm,
        key_options: Option<KeyOptions>,
    ) -> Result<PublicKeyResponse> {
        let mut url = format!(
            "/protected/public_key?algorithm={}",
            match algorithm {
                SigningAlgorithm::Schnorr => "schnorr",
                SigningAlgorithm::Ecdsa => "ecdsa",
            }
        );
        if let Some(opts) = key_options {
            if let Some(path) = &opts.private_key_derivation_path {
                let encoded = utf8_percent_encode(path, NON_ALPHANUMERIC).to_string();
                url.push_str(&format!("&private_key_derivation_path={}", encoded));
            }
            if let Some(path) = &opts.seed_phrase_derivation_path {
                let encoded = utf8_percent_encode(path, NON_ALPHANUMERIC).to_string();
                url.push_str(&format!("&seed_phrase_derivation_path={}", encoded));
            }
        }
        self.encrypted_api_call(&url, "GET", None::<()>).await
    }

    // Third Party Token API
    pub async fn generate_third_party_token(
        &self,
        audience: Option<String>,
    ) -> Result<ThirdPartyTokenResponse> {
        let request = ThirdPartyTokenRequest { audience };
        self.encrypted_api_call("/protected/third_party_token", "POST", Some(request))
            .await
    }

    // Encryption/Decryption APIs
    pub async fn encrypt_data(
        &self,
        data: String,
        key_options: Option<KeyOptions>,
    ) -> Result<EncryptDataResponse> {
        let request = EncryptDataRequest {
            data,
            key_options: key_options.map(|opts| EncryptionKeyOptions {
                private_key_derivation_path: opts.private_key_derivation_path,
                seed_phrase_derivation_path: opts.seed_phrase_derivation_path,
            }),
        };
        self.encrypted_api_call("/protected/encrypt", "POST", Some(request))
            .await
    }

    pub async fn decrypt_data(
        &self,
        encrypted_data: String,
        key_options: Option<KeyOptions>,
    ) -> Result<String> {
        let request = DecryptDataRequest {
            encrypted_data,
            key_options: key_options.map(|opts| EncryptionKeyOptions {
                private_key_derivation_path: opts.private_key_derivation_path,
                seed_phrase_derivation_path: opts.seed_phrase_derivation_path,
            }),
        };
        self.encrypted_api_call("/protected/decrypt", "POST", Some(request))
            .await
    }

    // Account Management APIs

    /// Changes the password for the currently authenticated user
    pub async fn change_password(
        &self,
        current_password: String,
        new_password: String,
    ) -> Result<()> {
        let request = ChangePasswordRequest {
            current_password,
            new_password,
        };
        let _: serde_json::Value = self
            .encrypted_api_call("/protected/change_password", "POST", Some(request))
            .await?;
        Ok(())
    }

    /// Requests a password reset for the given email
    /// Note: This does not require authentication but still uses encryption
    pub async fn request_password_reset(
        &self,
        email: String,
        hashed_secret: String,
        client_id: Uuid,
    ) -> Result<()> {
        let request = PasswordResetRequest {
            email,
            hashed_secret,
            client_id,
        };
        let _: serde_json::Value = self
            .encrypted_api_call("/password-reset/request", "POST", Some(request))
            .await?;
        Ok(())
    }

    /// Confirms a password reset with the code from email
    /// Note: This does not require authentication but still uses encryption
    pub async fn confirm_password_reset(
        &self,
        email: String,
        alphanumeric_code: String,
        plaintext_secret: String,
        new_password: String,
        client_id: Uuid,
    ) -> Result<()> {
        let request = PasswordResetConfirmRequest {
            email,
            alphanumeric_code,
            plaintext_secret,
            new_password,
            client_id,
        };
        let _: serde_json::Value = self
            .encrypted_api_call("/password-reset/confirm", "POST", Some(request))
            .await?;
        Ok(())
    }

    /// Converts a guest account to an email account
    pub async fn convert_guest_to_email(
        &self,
        email: String,
        password: String,
        name: Option<String>,
    ) -> Result<()> {
        let request = ConvertGuestToEmailRequest {
            email,
            password,
            name,
        };
        let _: serde_json::Value = self
            .encrypted_api_call("/protected/convert_guest", "POST", Some(request))
            .await?;
        Ok(())
    }

    /// Verifies an email address with the code from the verification email
    /// Note: This does not require authentication but still uses encryption
    pub async fn verify_email(&self, code: String) -> Result<()> {
        let _: serde_json::Value = self
            .encrypted_api_call(&format!("/verify-email/{}", code), "GET", None::<()>)
            .await?;
        Ok(())
    }

    /// Requests a new email verification code
    pub async fn request_new_verification_code(&self) -> Result<()> {
        let request = RequestVerificationCodeRequest {};
        let _: serde_json::Value = self
            .encrypted_api_call("/protected/request_verification", "POST", Some(request))
            .await?;
        Ok(())
    }

    /// Initiates the account deletion process
    pub async fn request_account_deletion(&self, hashed_secret: String) -> Result<()> {
        let request = InitiateAccountDeletionRequest { hashed_secret };
        let _: serde_json::Value = self
            .encrypted_api_call("/protected/delete-account/request", "POST", Some(request))
            .await?;
        Ok(())
    }

    /// Confirms account deletion with the code from email
    pub async fn confirm_account_deletion(
        &self,
        confirmation_code: String,
        plaintext_secret: String,
    ) -> Result<()> {
        let request = ConfirmAccountDeletionRequest {
            confirmation_code,
            plaintext_secret,
        };
        let _: serde_json::Value = self
            .encrypted_api_call("/protected/delete-account/confirm", "POST", Some(request))
            .await?;
        Ok(())
    }

    // AI/OpenAI API Methods

    /// Deletes all conversations
    pub async fn delete_conversations(&self) -> Result<ConversationsDeleteResponse> {
        self.encrypted_api_call("/v1/conversations", "DELETE", None::<()>)
            .await
    }

    /// Batch deletes multiple conversations by their IDs
    pub async fn batch_delete_conversations(
        &self,
        ids: Vec<String>,
    ) -> Result<BatchDeleteConversationsResponse> {
        let request = BatchDeleteConversationsRequest { ids };
        self.encrypted_api_call("/v1/conversations/batch-delete", "POST", Some(request))
            .await
    }

    /// Fetches available AI models
    pub async fn get_models(&self) -> Result<ModelsResponse> {
        self.encrypted_openai_call("/v1/models", "GET", None::<()>)
            .await
    }

    /// Creates embeddings for the given input text(s)
    ///
    /// # Example
    /// ```ignore
    /// let request = EmbeddingRequest {
    ///     input: "Hello, world!".into(),
    ///     model: "nomic-embed-text".to_string(),
    ///     encoding_format: None,
    ///     dimensions: None,
    ///     user: None,
    /// };
    /// let response = client.create_embeddings(request).await?;
    /// ```
    pub async fn create_embeddings(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse> {
        self.encrypted_openai_call("/v1/embeddings", "POST", Some(request))
            .await
    }

    /// Creates a chat completion (non-streaming)
    pub async fn create_chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse> {
        let mut modified_request = request;
        modified_request.stream = Some(false);
        self.encrypted_openai_call("/v1/chat/completions", "POST", Some(modified_request))
            .await
    }

    /// Creates a streaming chat completion
    pub async fn create_chat_completion_stream(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<std::pin::Pin<Box<dyn futures::Stream<Item = Result<ChatCompletionChunk>> + Send>>>
    {
        use eventsource_stream::Eventsource;
        use futures::StreamExt;

        let mut modified_request = request;
        modified_request.stream = Some(true);
        modified_request.stream_options = Some(StreamOptions {
            include_usage: true,
        });

        // Get session for encryption
        let session = self.session_manager.get_session()?.ok_or_else(|| {
            Error::Session(
                "No active session. Call perform_attestation_handshake first".to_string(),
            )
        })?;

        let url = format!("{}/v1/chat/completions", self.base_url);

        // Encrypt the request
        let json = serde_json::to_string(&modified_request)?;
        let encrypted = crypto::encrypt_data(&session.session_key, json.as_bytes())?;
        let encrypted_request = EncryptedRequest {
            encrypted: BASE64.encode(&encrypted),
        };

        // Build headers
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert("accept", HeaderValue::from_static("text/event-stream"));
        headers.insert(
            "x-session-id",
            HeaderValue::from_str(&session.session_id.to_string())
                .map_err(|e| Error::Session(format!("Invalid session ID: {}", e)))?,
        );

        // Add authorization header if we have a token or API key
        // Prefer API key over JWT token if both are present
        if let Some(api_key) = self.session_manager.get_api_key()? {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", api_key))
                    .map_err(|e| Error::Authentication(format!("Invalid API key format: {}", e)))?,
            );
        } else if let Some(token) = self.session_manager.get_access_token()? {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", token))
                    .map_err(|e| Error::Authentication(format!("Invalid token format: {}", e)))?,
            );
        }

        // Make the streaming request
        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&encrypted_request)
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

        // Convert response to stream of bytes
        let stream = response
            .bytes_stream()
            .map(|result| result.map_err(std::io::Error::other));

        // Parse SSE events and decrypt
        let session_key = session.session_key;
        let event_stream = stream.eventsource().filter_map(move |event| {
            let session_key = session_key;
            async move {
                match event {
                    Ok(event) => {
                        // Check if this is the [DONE] event
                        if event.data == "[DONE]" {
                            return None;
                        }

                        // Decrypt the event data - server sends base64 encrypted chunks
                        match BASE64.decode(&event.data) {
                            Ok(encrypted_bytes) => {
                                match crypto::decrypt_data(&session_key, &encrypted_bytes) {
                                    Ok(decrypted) => match String::from_utf8(decrypted) {
                                        Ok(json_str) => {
                                            match serde_json::from_str::<ChatCompletionChunk>(
                                                &json_str,
                                            ) {
                                                Ok(chunk) => Some(Ok(chunk)),
                                                Err(e) => Some(Err(Error::Api {
                                                    status: 0,
                                                    message: format!(
                                                        "Failed to parse chunk: {}",
                                                        e
                                                    ),
                                                })),
                                            }
                                        }
                                        Err(e) => Some(Err(Error::Api {
                                            status: 0,
                                            message: format!(
                                                "Invalid UTF-8 in decrypted data: {}",
                                                e
                                            ),
                                        })),
                                    },
                                    Err(e) => Some(Err(Error::Decryption(format!(
                                        "Failed to decrypt chunk: {}",
                                        e
                                    )))),
                                }
                            }
                            Err(e) => Some(Err(Error::Api {
                                status: 0,
                                message: format!("Failed to decode base64: {}", e),
                            })),
                        }
                    }
                    Err(e) => Some(Err(Error::Api {
                        status: 0,
                        message: format!("SSE error: {}", e),
                    })),
                }
            }
        });

        Ok(Box::pin(event_stream))
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
