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

fn append_query_param(query: &mut Vec<String>, key: &str, value: impl ToString) {
    let encoded = utf8_percent_encode(&value.to_string(), NON_ALPHANUMERIC).to_string();
    query.push(format!("{}={}", key, encoded));
}

fn build_agent_items_endpoint(base: &str, params: Option<&AgentItemsListParams>) -> String {
    let mut endpoint = base.to_string();
    let mut query = Vec::new();

    if let Some(params) = params {
        if let Some(limit) = params.limit {
            append_query_param(&mut query, "limit", limit);
        }
        if let Some(after) = params.after {
            append_query_param(&mut query, "after", after);
        }
        if let Some(order) = &params.order {
            append_query_param(&mut query, "order", order);
        }
        if let Some(include) = &params.include {
            for include_value in include {
                append_query_param(&mut query, "include", include_value);
            }
        }
    }

    if !query.is_empty() {
        endpoint.push('?');
        endpoint.push_str(&query.join("&"));
    }

    endpoint
}

fn build_subagents_endpoint(params: Option<&ListSubagentsParams>) -> String {
    let mut endpoint = "/v1/agent/subagents".to_string();
    let mut query = Vec::new();

    if let Some(params) = params {
        if let Some(limit) = params.limit {
            append_query_param(&mut query, "limit", limit);
        }
        if let Some(after) = params.after {
            append_query_param(&mut query, "after", after);
        }
        if let Some(order) = &params.order {
            append_query_param(&mut query, "order", order);
        }
        if let Some(created_by) = &params.created_by {
            append_query_param(&mut query, "created_by", created_by);
        }
    }

    if !query.is_empty() {
        endpoint.push('?');
        endpoint.push_str(&query.join("&"));
    }

    endpoint
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AuthHeaderMode {
    None,
    Jwt,
    ApiKeyOrJwt,
}

impl OpenSecretClient {
    pub fn new(base_url: impl Into<String>) -> Result<Self> {
        let base_url = base_url.into();
        let use_mock = base_url.contains("localhost")
            || base_url.contains("127.0.0.1")
            || base_url.contains("0.0.0.0")
            || base_url.contains("10.0.2.2");

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
        let use_mock = base_url.contains("localhost")
            || base_url.contains("127.0.0.1")
            || base_url.contains("0.0.0.0")
            || base_url.contains("10.0.2.2");

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

    async fn encrypted_api_call<T: Serialize + Clone, U: DeserializeOwned>(
        &self,
        endpoint: &str,
        method: &str,
        data: Option<T>,
    ) -> Result<U> {
        self.retry_encrypted_json_call_without_refresh(endpoint, method, data, AuthHeaderMode::None)
            .await
    }

    async fn authenticated_api_call<T: Serialize + Clone, U: DeserializeOwned>(
        &self,
        endpoint: &str,
        method: &str,
        data: Option<T>,
    ) -> Result<U> {
        self.retry_encrypted_json_call(endpoint, method, data, AuthHeaderMode::Jwt, true)
            .await
    }

    async fn retry_encrypted_json_call_without_refresh<
        T: Serialize + Clone,
        U: DeserializeOwned,
    >(
        &self,
        endpoint: &str,
        method: &str,
        data: Option<T>,
        auth_mode: AuthHeaderMode,
    ) -> Result<U> {
        let mut retried_attestation = false;

        loop {
            match self
                .encrypted_json_call_inner(endpoint, method, data.clone(), auth_mode)
                .await
            {
                Ok(result) => return Ok(result),
                Err(error) if !retried_attestation && Self::is_attestation_retryable(&error) => {
                    self.perform_attestation_handshake().await?;
                    retried_attestation = true;
                }
                Err(error) => return Err(error),
            }
        }
    }

    async fn retry_encrypted_json_call<T: Serialize + Clone, U: DeserializeOwned>(
        &self,
        endpoint: &str,
        method: &str,
        data: Option<T>,
        auth_mode: AuthHeaderMode,
        allow_refresh: bool,
    ) -> Result<U> {
        let mut retried_attestation = false;
        let mut retried_refresh = false;

        loop {
            match self
                .encrypted_json_call_inner(endpoint, method, data.clone(), auth_mode)
                .await
            {
                Ok(result) => return Ok(result),
                Err(error) if !retried_attestation && Self::is_attestation_retryable(&error) => {
                    self.perform_attestation_handshake().await?;
                    retried_attestation = true;
                }
                Err(Error::Api { status: 401, .. })
                    if allow_refresh && !retried_refresh && !self.using_api_key(auth_mode)? =>
                {
                    self.refresh_token().await?;
                    retried_refresh = true;
                }
                Err(error) => return Err(error),
            }
        }
    }

    async fn encrypted_json_call_inner<T: Serialize, U: DeserializeOwned>(
        &self,
        endpoint: &str,
        method: &str,
        data: Option<T>,
        auth_mode: AuthHeaderMode,
    ) -> Result<U> {
        let (response, session_key) = self
            .send_encrypted_request(endpoint, method, data, auth_mode, false)
            .await?;
        let encrypted_response: EncryptedResponse<U> = response.json().await?;
        let decrypted =
            crypto::decrypt_data(&session_key, &BASE64.decode(&encrypted_response.encrypted)?)?;
        let result: U = serde_json::from_slice(&decrypted)?;

        Ok(result)
    }

    /// Encrypted API call specifically for OpenAI endpoints (/v1/*)
    /// This supports both API key and JWT authentication, with API key taking priority
    async fn encrypted_openai_call<T: Serialize + Clone, U: DeserializeOwned>(
        &self,
        endpoint: &str,
        method: &str,
        data: Option<T>,
    ) -> Result<U> {
        self.retry_encrypted_json_call(endpoint, method, data, AuthHeaderMode::ApiKeyOrJwt, true)
            .await
    }

    async fn retry_encrypted_stream_call<T: Serialize + Clone>(
        &self,
        endpoint: &str,
        method: &str,
        data: Option<T>,
        auth_mode: AuthHeaderMode,
        allow_refresh: bool,
    ) -> Result<(reqwest::Response, [u8; 32])> {
        let mut retried_attestation = false;
        let mut retried_refresh = false;

        loop {
            match self
                .send_encrypted_request(endpoint, method, data.clone(), auth_mode, true)
                .await
            {
                Ok(response) => return Ok(response),
                Err(error) if !retried_attestation && Self::is_attestation_retryable(&error) => {
                    self.perform_attestation_handshake().await?;
                    retried_attestation = true;
                }
                Err(Error::Api { status: 401, .. })
                    if allow_refresh && !retried_refresh && !self.using_api_key(auth_mode)? =>
                {
                    self.refresh_token().await?;
                    retried_refresh = true;
                }
                Err(error) => return Err(error),
            }
        }
    }

    async fn send_encrypted_request<T: Serialize>(
        &self,
        endpoint: &str,
        method: &str,
        data: Option<T>,
        auth_mode: AuthHeaderMode,
        accept_sse: bool,
    ) -> Result<(reqwest::Response, [u8; 32])> {
        let session = self.session_manager.get_session()?.ok_or_else(|| {
            Error::Session(
                "No active session. Call perform_attestation_handshake first".to_string(),
            )
        })?;

        let url = format!("{}{}", self.base_url, endpoint);

        let encrypted_body = if let Some(data) = data {
            let json = serde_json::to_string(&data)?;
            let encrypted = crypto::encrypt_data(&session.session_key, json.as_bytes())?;
            Some(EncryptedRequest {
                encrypted: BASE64.encode(&encrypted),
            })
        } else {
            None
        };

        let headers = self.build_encrypted_headers(&session, auth_mode, accept_sse)?;
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
        let response = if let Some(body) = encrypted_body {
            request_builder.json(&body).send().await?
        } else {
            request_builder.send().await?
        };

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_msg = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Api {
                status,
                message: error_msg,
            });
        }

        Ok((response, session.session_key))
    }

    fn build_encrypted_headers(
        &self,
        session: &crate::types::SessionState,
        auth_mode: AuthHeaderMode,
        accept_sse: bool,
    ) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        if accept_sse {
            headers.insert("accept", HeaderValue::from_static("text/event-stream"));
        }

        headers.insert(
            "x-session-id",
            HeaderValue::from_str(&session.session_id.to_string())
                .map_err(|e| Error::Session(format!("Invalid session ID: {}", e)))?,
        );

        if let Some(token) = self.resolve_auth_token(auth_mode)? {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", token)).map_err(|e| {
                    Error::Authentication(format!("Invalid authorization credential format: {}", e))
                })?,
            );
        }

        Ok(headers)
    }

    fn resolve_auth_token(&self, auth_mode: AuthHeaderMode) -> Result<Option<String>> {
        match auth_mode {
            AuthHeaderMode::None => Ok(None),
            AuthHeaderMode::Jwt => self.session_manager.get_access_token(),
            AuthHeaderMode::ApiKeyOrJwt => {
                if let Some(api_key) = self.session_manager.get_api_key()? {
                    Ok(Some(api_key))
                } else {
                    self.session_manager.get_access_token()
                }
            }
        }
    }

    fn using_api_key(&self, auth_mode: AuthHeaderMode) -> Result<bool> {
        match auth_mode {
            AuthHeaderMode::ApiKeyOrJwt => Ok(self.session_manager.get_api_key()?.is_some()),
            _ => Ok(false),
        }
    }

    fn is_attestation_retryable(error: &Error) -> bool {
        matches!(
            error,
            Error::Session(_)
                | Error::Api { status: 400, .. }
                | Error::Encryption(_)
                | Error::Decryption(_)
        )
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

    // OAuth Methods

    pub async fn initiate_github_auth(
        &self,
        client_id: Uuid,
        invite_code: Option<String>,
    ) -> Result<GithubAuthResponse> {
        let request = OAuthInitRequest {
            client_id,
            invite_code,
        };
        self.encrypted_api_call("/auth/github", "POST", Some(request))
            .await
    }

    pub async fn handle_github_callback(
        &self,
        code: String,
        state: String,
        invite_code: String,
    ) -> Result<LoginResponse> {
        let request = OAuthCallbackRequest {
            code,
            state,
            invite_code,
        };

        let response: LoginResponse = self
            .encrypted_api_call("/auth/github/callback", "POST", Some(request))
            .await?;

        self.session_manager.set_tokens(
            response.access_token.clone(),
            Some(response.refresh_token.clone()),
        )?;

        Ok(response)
    }

    pub async fn initiate_google_auth(
        &self,
        client_id: Uuid,
        invite_code: Option<String>,
    ) -> Result<GoogleAuthResponse> {
        let request = OAuthInitRequest {
            client_id,
            invite_code,
        };
        self.encrypted_api_call("/auth/google", "POST", Some(request))
            .await
    }

    pub async fn handle_google_callback(
        &self,
        code: String,
        state: String,
        invite_code: String,
    ) -> Result<LoginResponse> {
        let request = OAuthCallbackRequest {
            code,
            state,
            invite_code,
        };

        let response: LoginResponse = self
            .encrypted_api_call("/auth/google/callback", "POST", Some(request))
            .await?;

        self.session_manager.set_tokens(
            response.access_token.clone(),
            Some(response.refresh_token.clone()),
        )?;

        Ok(response)
    }

    pub async fn initiate_apple_auth(
        &self,
        client_id: Uuid,
        invite_code: Option<String>,
    ) -> Result<AppleAuthResponse> {
        let request = OAuthInitRequest {
            client_id,
            invite_code,
        };
        self.encrypted_api_call("/auth/apple", "POST", Some(request))
            .await
    }

    pub async fn handle_apple_callback(
        &self,
        code: String,
        state: String,
        invite_code: String,
    ) -> Result<LoginResponse> {
        let request = OAuthCallbackRequest {
            code,
            state,
            invite_code,
        };

        let response: LoginResponse = self
            .encrypted_api_call("/auth/apple/callback", "POST", Some(request))
            .await?;

        self.session_manager.set_tokens(
            response.access_token.clone(),
            Some(response.refresh_token.clone()),
        )?;

        Ok(response)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn handle_apple_native_sign_in(
        &self,
        user_identifier: String,
        identity_token: String,
        client_id: Uuid,
        email: Option<String>,
        given_name: Option<String>,
        family_name: Option<String>,
        nonce: Option<String>,
        invite_code: Option<String>,
    ) -> Result<LoginResponse> {
        let request = AppleNativeSignInRequest {
            user_identifier,
            identity_token,
            client_id,
            email,
            given_name,
            family_name,
            nonce,
            invite_code,
        };

        let response: LoginResponse = self
            .encrypted_api_call("/auth/apple/native", "POST", Some(request))
            .await?;

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

    pub fn set_tokens(&self, access_token: String, refresh_token: Option<String>) -> Result<()> {
        self.session_manager.clear_session()?;
        self.session_manager.set_tokens(access_token, refresh_token)
    }

    // User Profile API
    pub async fn get_user(&self) -> Result<UserResponse> {
        self.authenticated_api_call("/protected/user", "GET", None::<()>)
            .await
    }

    // API Key Management
    pub async fn create_api_key(&self, name: String) -> Result<ApiKeyCreateResponse> {
        let request = ApiKeyCreateRequest { name };
        self.authenticated_api_call("/protected/api-keys", "POST", Some(request))
            .await
    }

    pub async fn list_api_keys(&self) -> Result<Vec<ApiKey>> {
        let response: ApiKeyListResponse = self
            .authenticated_api_call("/protected/api-keys", "GET", None::<()>)
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
        let _: serde_json::Value = self
            .authenticated_api_call(&url, "DELETE", None::<()>)
            .await?;
        Ok(())
    }

    // Key-Value Storage APIs
    pub async fn kv_get(&self, key: &str) -> Result<String> {
        let encoded_key = utf8_percent_encode(key, NON_ALPHANUMERIC).to_string();
        let url = format!("/protected/kv/{}", encoded_key);
        self.authenticated_api_call(&url, "GET", None::<()>).await
    }

    pub async fn kv_put(&self, key: &str, value: String) -> Result<String> {
        let encoded_key = utf8_percent_encode(key, NON_ALPHANUMERIC).to_string();
        let url = format!("/protected/kv/{}", encoded_key);
        self.authenticated_api_call(&url, "PUT", Some(value)).await
    }

    pub async fn kv_delete(&self, key: &str) -> Result<()> {
        let encoded_key = utf8_percent_encode(key, NON_ALPHANUMERIC).to_string();
        let url = format!("/protected/kv/{}", encoded_key);
        let _: serde_json::Value = self
            .authenticated_api_call(&url, "DELETE", None::<()>)
            .await?;
        Ok(())
    }

    pub async fn kv_delete_all(&self) -> Result<()> {
        let _: serde_json::Value = self
            .authenticated_api_call("/protected/kv", "DELETE", None::<()>)
            .await?;
        Ok(())
    }

    pub async fn kv_list(&self) -> Result<Vec<KVListItem>> {
        self.authenticated_api_call("/protected/kv", "GET", None::<()>)
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
        self.authenticated_api_call(&url, "GET", None::<()>).await
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
        self.authenticated_api_call(&url, "GET", None::<()>).await
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
        self.authenticated_api_call("/protected/sign_message", "POST", Some(request))
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
        self.authenticated_api_call(&url, "GET", None::<()>).await
    }

    // Third Party Token API
    pub async fn generate_third_party_token(
        &self,
        audience: Option<String>,
    ) -> Result<ThirdPartyTokenResponse> {
        let request = ThirdPartyTokenRequest { audience };
        self.authenticated_api_call("/protected/third_party_token", "POST", Some(request))
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
        self.authenticated_api_call("/protected/encrypt", "POST", Some(request))
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
        self.authenticated_api_call("/protected/decrypt", "POST", Some(request))
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
            .authenticated_api_call("/protected/change_password", "POST", Some(request))
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
            .authenticated_api_call("/protected/convert_guest", "POST", Some(request))
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
            .authenticated_api_call("/protected/request_verification", "POST", Some(request))
            .await?;
        Ok(())
    }

    /// Initiates the account deletion process
    pub async fn request_account_deletion(&self, hashed_secret: String) -> Result<()> {
        let request = InitiateAccountDeletionRequest { hashed_secret };
        let _: serde_json::Value = self
            .authenticated_api_call("/protected/delete-account/request", "POST", Some(request))
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
            .authenticated_api_call("/protected/delete-account/confirm", "POST", Some(request))
            .await?;
        Ok(())
    }

    // AI/OpenAI API Methods

    /// Deletes all conversations
    pub async fn delete_conversations(&self) -> Result<ConversationsDeleteResponse> {
        self.authenticated_api_call("/v1/conversations", "DELETE", None::<()>)
            .await
    }

    /// Batch deletes multiple conversations by their IDs
    pub async fn batch_delete_conversations(
        &self,
        ids: Vec<String>,
    ) -> Result<BatchDeleteConversationsResponse> {
        let request = BatchDeleteConversationsRequest { ids };
        self.authenticated_api_call("/v1/conversations/batch-delete", "POST", Some(request))
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

        let (response, session_key) = self
            .retry_encrypted_stream_call(
                "/v1/chat/completions",
                "POST",
                Some(modified_request),
                AuthHeaderMode::ApiKeyOrJwt,
                true,
            )
            .await?;

        let stream = response
            .bytes_stream()
            .map(|result| result.map_err(std::io::Error::other));

        let event_stream = stream.eventsource().filter_map(move |event| {
            let session_key = session_key;
            async move {
                match event {
                    Ok(event) => {
                        // Check if this is the [DONE] event
                        if event.data == "[DONE]" {
                            return None;
                        }

                        // Decrypt the event data - server sends base64 encrypted chunks.
                        // Skip non-base64 events (heartbeats, retries, etc.) to match TS SDK.
                        let encrypted_bytes = match BASE64.decode(&event.data) {
                            Ok(bytes) => bytes,
                            Err(_) => return None,
                        };
                        match crypto::decrypt_data(&session_key, &encrypted_bytes) {
                            Ok(decrypted) => match String::from_utf8(decrypted) {
                                Ok(json_str) => {
                                    match serde_json::from_str::<ChatCompletionChunk>(&json_str) {
                                        Ok(chunk) => Some(Ok(chunk)),
                                        Err(e) => Some(Err(Error::Api {
                                            status: 0,
                                            message: format!("Failed to parse chunk: {}", e),
                                        })),
                                    }
                                }
                                Err(e) => Some(Err(Error::Api {
                                    status: 0,
                                    message: format!("Invalid UTF-8 in decrypted data: {}", e),
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
                        message: format!("SSE error: {}", e),
                    })),
                }
            }
        });

        Ok(Box::pin(event_stream))
    }

    async fn agent_chat_stream(
        &self,
        endpoint: String,
        input: &str,
    ) -> Result<std::pin::Pin<Box<dyn futures::Stream<Item = Result<AgentSseEvent>> + Send>>> {
        use eventsource_stream::Eventsource;
        use futures::StreamExt;

        let request = AgentChatRequest {
            input: input.to_string(),
        };

        let (response, session_key) = self
            .retry_encrypted_stream_call(
                &endpoint,
                "POST",
                Some(request),
                AuthHeaderMode::Jwt,
                true,
            )
            .await?;

        let stream = response
            .bytes_stream()
            .map(|result| result.map_err(std::io::Error::other));

        let event_stream = stream.eventsource().filter_map(move |event| {
            let session_key = session_key;
            async move {
                match event {
                    Ok(event) => {
                        if event.data == "[DONE]" {
                            return None;
                        }

                        // Skip non-base64 events (heartbeats, retries, etc.)
                        let encrypted_bytes = match BASE64.decode(&event.data) {
                            Ok(bytes) => bytes,
                            Err(_) => return None,
                        };
                        match crypto::decrypt_data(&session_key, &encrypted_bytes) {
                            Ok(decrypted) => match String::from_utf8(decrypted) {
                                Ok(json_str) => {
                                    let event_type = event.event.as_str();
                                    match event_type {
                                        "agent.message" => {
                                            match serde_json::from_str::<AgentMessageEvent>(
                                                &json_str,
                                            ) {
                                                Ok(msg) => Some(Ok(AgentSseEvent::Message(msg))),
                                                Err(e) => Some(Err(Error::Api {
                                                    status: 0,
                                                    message: format!(
                                                        "Failed to parse agent message: {}",
                                                        e
                                                    ),
                                                })),
                                            }
                                        }
                                        "agent.typing" => {
                                            match serde_json::from_str::<AgentTypingEvent>(
                                                &json_str,
                                            ) {
                                                Ok(typing) => {
                                                    Some(Ok(AgentSseEvent::Typing(typing)))
                                                }
                                                Err(e) => Some(Err(Error::Api {
                                                    status: 0,
                                                    message: format!(
                                                        "Failed to parse agent typing: {}",
                                                        e
                                                    ),
                                                })),
                                            }
                                        }
                                        "agent.done" => {
                                            match serde_json::from_str::<AgentDoneEvent>(&json_str)
                                            {
                                                Ok(done) => Some(Ok(AgentSseEvent::Done(done))),
                                                Err(e) => Some(Err(Error::Api {
                                                    status: 0,
                                                    message: format!(
                                                        "Failed to parse agent done: {}",
                                                        e
                                                    ),
                                                })),
                                            }
                                        }
                                        "agent.error" => {
                                            match serde_json::from_str::<AgentErrorEvent>(&json_str)
                                            {
                                                Ok(err) => Some(Ok(AgentSseEvent::Error(err))),
                                                Err(e) => Some(Err(Error::Api {
                                                    status: 0,
                                                    message: format!(
                                                        "Failed to parse agent error: {}",
                                                        e
                                                    ),
                                                })),
                                            }
                                        }
                                        _ => None,
                                    }
                                }
                                Err(e) => Some(Err(Error::Api {
                                    status: 0,
                                    message: format!("Invalid UTF-8 in decrypted data: {}", e),
                                })),
                            },
                            Err(e) => Some(Err(Error::Decryption(format!(
                                "Failed to decrypt agent event: {}",
                                e
                            )))),
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

    // Agent API Methods

    /// Fetches the current user's main agent.
    pub async fn get_main_agent(&self) -> Result<MainAgentResponse> {
        self.authenticated_api_call("/v1/agent", "GET", None::<()>)
            .await
    }

    /// Deletes the current user's main agent and resets shared agent state.
    pub async fn delete_main_agent(&self) -> Result<DeletedObjectResponse> {
        self.authenticated_api_call("/v1/agent", "DELETE", None::<()>)
            .await
    }

    /// Lists items in the main agent conversation.
    pub async fn list_main_agent_items(
        &self,
        params: Option<AgentItemsListParams>,
    ) -> Result<AgentItemsListResponse> {
        let endpoint = build_agent_items_endpoint("/v1/agent/items", params.as_ref());
        self.authenticated_api_call(&endpoint, "GET", None::<()>)
            .await
    }

    /// Fetches a single item from the main agent conversation.
    pub async fn get_main_agent_item(&self, item_id: Uuid) -> Result<ConversationItem> {
        self.authenticated_api_call(&format!("/v1/agent/items/{}", item_id), "GET", None::<()>)
            .await
    }

    /// Sends a message to the main agent and returns a stream of SSE events.
    pub async fn agent_chat(
        &self,
        input: &str,
    ) -> Result<std::pin::Pin<Box<dyn futures::Stream<Item = Result<AgentSseEvent>> + Send>>> {
        self.agent_chat_stream("/v1/agent/chat".to_string(), input)
            .await
    }

    /// Creates a new subagent for the current user.
    pub async fn create_subagent(
        &self,
        request: CreateSubagentRequest,
    ) -> Result<SubagentResponse> {
        self.authenticated_api_call("/v1/agent/subagents", "POST", Some(request))
            .await
    }

    /// Lists subagents for the current user with pagination and filtering.
    pub async fn list_subagents(
        &self,
        params: Option<ListSubagentsParams>,
    ) -> Result<SubagentListResponse> {
        let endpoint = build_subagents_endpoint(params.as_ref());
        self.authenticated_api_call(&endpoint, "GET", None::<()>)
            .await
    }

    /// Fetches a single subagent by UUID.
    pub async fn get_subagent(&self, id: Uuid) -> Result<SubagentResponse> {
        self.authenticated_api_call(&format!("/v1/agent/subagents/{}", id), "GET", None::<()>)
            .await
    }

    /// Sends a message to a specific subagent and returns a stream of SSE events.
    pub async fn subagent_chat(
        &self,
        id: Uuid,
        input: &str,
    ) -> Result<std::pin::Pin<Box<dyn futures::Stream<Item = Result<AgentSseEvent>> + Send>>> {
        self.agent_chat_stream(format!("/v1/agent/subagents/{}/chat", id), input)
            .await
    }

    /// Lists items in a subagent conversation.
    pub async fn list_subagent_items(
        &self,
        id: Uuid,
        params: Option<AgentItemsListParams>,
    ) -> Result<AgentItemsListResponse> {
        let endpoint = build_agent_items_endpoint(
            &format!("/v1/agent/subagents/{}/items", id),
            params.as_ref(),
        );
        self.authenticated_api_call(&endpoint, "GET", None::<()>)
            .await
    }

    /// Fetches a single item from a subagent conversation.
    pub async fn get_subagent_item(&self, id: Uuid, item_id: Uuid) -> Result<ConversationItem> {
        self.authenticated_api_call(
            &format!("/v1/agent/subagents/{}/items/{}", id, item_id),
            "GET",
            None::<()>,
        )
        .await
    }

    /// Deletes a subagent by UUID.
    pub async fn delete_subagent(&self, id: Uuid) -> Result<DeletedObjectResponse> {
        self.authenticated_api_call(&format!("/v1/agent/subagents/{}", id), "DELETE", None::<()>)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_cbor::Value as CborValue;
    use serde_json::json;
    use std::collections::BTreeMap;
    use wiremock::{
        matchers::{header, method, path},
        Match, Mock, MockServer, Request, Respond, ResponseTemplate,
    };

    struct MissingHeaderMatcher(&'static str);

    impl Match for MissingHeaderMatcher {
        fn matches(&self, request: &Request) -> bool {
            !request.headers.contains_key(self.0)
        }
    }

    struct PathPrefixMatcher(&'static str);

    impl Match for PathPrefixMatcher {
        fn matches(&self, request: &Request) -> bool {
            request.url.path().starts_with(self.0)
        }
    }

    struct AttestationResponder {
        server_public_key: [u8; 32],
    }

    impl Respond for AttestationResponder {
        fn respond(&self, request: &Request) -> ResponseTemplate {
            let nonce = request.url.path().rsplit('/').next().unwrap_or_default();
            let attestation_document =
                build_mock_attestation_document(nonce, &self.server_public_key);

            ResponseTemplate::new(200)
                .set_body_json(json!({ "attestation_document": attestation_document }))
        }
    }

    struct KeyExchangeResponder {
        server_secret_key: [u8; 32],
        session_key: [u8; 32],
        session_id: String,
    }

    impl Respond for KeyExchangeResponder {
        fn respond(&self, request: &Request) -> ResponseTemplate {
            let body: KeyExchangeRequest = serde_json::from_slice(request.body.as_ref()).unwrap();
            let client_public_bytes = BASE64.decode(body.client_public_key.as_bytes()).unwrap();
            let client_public_key = x25519_dalek::PublicKey::from(
                <[u8; 32]>::try_from(client_public_bytes.as_slice()).unwrap(),
            );
            let server_secret = x25519_dalek::StaticSecret::from(self.server_secret_key);
            let shared_secret =
                crypto::perform_static_key_exchange(&server_secret, &client_public_key);
            let encrypted_session_key = BASE64
                .encode(crypto::encrypt_data(shared_secret.as_bytes(), &self.session_key).unwrap());

            ResponseTemplate::new(200).set_body_json(json!({
                "encrypted_session_key": encrypted_session_key,
                "session_id": self.session_id,
            }))
        }
    }

    fn build_mock_attestation_document(nonce: &str, server_public_key: &[u8; 32]) -> String {
        let mut payload = BTreeMap::new();
        payload.insert(
            CborValue::Text("public_key".to_string()),
            CborValue::Bytes(server_public_key.to_vec()),
        );
        payload.insert(
            CborValue::Text("nonce".to_string()),
            CborValue::Bytes(nonce.as_bytes().to_vec()),
        );

        let payload = serde_cbor::to_vec(&CborValue::Map(payload)).unwrap();
        let cose_sign1 = CborValue::Array(vec![
            CborValue::Bytes(vec![]),
            CborValue::Map(BTreeMap::new()),
            CborValue::Bytes(payload),
            CborValue::Bytes(vec![]),
        ]);

        BASE64.encode(serde_cbor::to_vec(&cose_sign1).unwrap())
    }

    fn encrypted_response<T: Serialize>(session_key: &[u8; 32], payload: &T) -> serde_json::Value {
        let plaintext = serde_json::to_vec(payload).unwrap();
        let encrypted = crypto::encrypt_data(session_key, &plaintext).unwrap();
        json!({ "encrypted": BASE64.encode(encrypted) })
    }

    #[tokio::test]
    async fn test_client_creation() {
        let client = OpenSecretClient::new("http://localhost:3000").unwrap();
        assert_eq!(client.base_url, "http://localhost:3000");
        assert!(client.use_mock_attestation);
    }

    #[tokio::test]
    async fn test_authenticated_calls_refresh_and_retry_seamlessly() {
        let mock_server = MockServer::start().await;
        let client = OpenSecretClient::new(mock_server.uri()).unwrap();
        let session_id = Uuid::new_v4();
        let session_key = [7u8; 32];
        let expired_access = "expired_access";
        let new_access = "new_access";
        let new_refresh = "new_refresh";
        let expired_header = format!("Bearer {}", expired_access);
        let fresh_header = format!("Bearer {}", new_access);

        client
            .session_manager
            .set_session(session_id, session_key)
            .unwrap();
        client
            .session_manager
            .set_tokens(
                expired_access.to_string(),
                Some("refresh_token".to_string()),
            )
            .unwrap();

        Mock::given(method("GET"))
            .and(path("/protected/user"))
            .and(header("authorization", &expired_header))
            .and(header("x-session-id", session_id.to_string()))
            .respond_with(
                ResponseTemplate::new(401).set_body_json(json!({ "message": "jwt expired" })),
            )
            .expect(1)
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/refresh"))
            .and(MissingHeaderMatcher("authorization"))
            .and(header("x-session-id", session_id.to_string()))
            .respond_with(ResponseTemplate::new(200).set_body_json(encrypted_response(
                &session_key,
                &json!({
                    "access_token": new_access,
                    "refresh_token": new_refresh,
                }),
            )))
            .expect(1)
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/protected/user"))
            .and(header("authorization", &fresh_header))
            .and(header("x-session-id", session_id.to_string()))
            .respond_with(ResponseTemplate::new(200).set_body_json(encrypted_response(
                &session_key,
                &json!({
                    "user": {
                        "id": Uuid::new_v4(),
                        "name": null,
                        "email": "sdk@test.dev",
                        "email_verified": true,
                        "login_method": "email",
                        "created_at": "2024-01-01T00:00:00Z",
                        "updated_at": "2024-01-01T00:00:00Z"
                    }
                }),
            )))
            .expect(1)
            .mount(&mock_server)
            .await;

        let response = client.get_user().await.unwrap();

        assert_eq!(response.user.email.as_deref(), Some("sdk@test.dev"));
        assert_eq!(
            client.get_access_token().unwrap().as_deref(),
            Some(new_access)
        );
        assert_eq!(
            client.get_refresh_token().unwrap().as_deref(),
            Some(new_refresh)
        );
    }

    #[tokio::test]
    async fn test_corrupted_access_token_recovers_via_refresh_on_next_call() {
        let mock_server = MockServer::start().await;
        let client = OpenSecretClient::new(mock_server.uri()).unwrap();
        let session_id = Uuid::new_v4();
        let session_key = [5u8; 32];
        let original_access = "valid_access";
        let original_refresh = "valid_refresh";
        let corrupted_access = "malformed_access";
        let refreshed_access = "refreshed_access";
        let refreshed_refresh = "refreshed_refresh";

        client
            .session_manager
            .set_session(session_id, session_key)
            .unwrap();

        Mock::given(method("POST"))
            .and(path("/login"))
            .and(MissingHeaderMatcher("authorization"))
            .and(header("x-session-id", session_id.to_string()))
            .respond_with(ResponseTemplate::new(200).set_body_json(encrypted_response(
                &session_key,
                &json!({
                    "id": Uuid::new_v4(),
                    "email": "sdk@test.dev",
                    "access_token": original_access,
                    "refresh_token": original_refresh,
                }),
            )))
            .expect(1)
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/protected/user"))
            .and(header(
                "authorization",
                format!("Bearer {}", original_access),
            ))
            .and(header("x-session-id", session_id.to_string()))
            .respond_with(ResponseTemplate::new(200).set_body_json(encrypted_response(
                &session_key,
                &json!({
                    "user": {
                        "id": Uuid::new_v4(),
                        "name": null,
                        "email": "sdk@test.dev",
                        "email_verified": true,
                        "login_method": "email",
                        "created_at": "2024-01-01T00:00:00Z",
                        "updated_at": "2024-01-01T00:00:00Z"
                    }
                }),
            )))
            .expect(1)
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/protected/user"))
            .and(header(
                "authorization",
                format!("Bearer {}", corrupted_access),
            ))
            .and(header("x-session-id", session_id.to_string()))
            .respond_with(
                ResponseTemplate::new(401).set_body_json(json!({ "message": "invalid jwt" })),
            )
            .expect(1)
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/refresh"))
            .and(MissingHeaderMatcher("authorization"))
            .and(header("x-session-id", session_id.to_string()))
            .respond_with(ResponseTemplate::new(200).set_body_json(encrypted_response(
                &session_key,
                &json!({
                    "access_token": refreshed_access,
                    "refresh_token": refreshed_refresh,
                }),
            )))
            .expect(1)
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/protected/user"))
            .and(header(
                "authorization",
                format!("Bearer {}", refreshed_access),
            ))
            .and(header("x-session-id", session_id.to_string()))
            .respond_with(ResponseTemplate::new(200).set_body_json(encrypted_response(
                &session_key,
                &json!({
                    "user": {
                        "id": Uuid::new_v4(),
                        "name": null,
                        "email": "sdk@test.dev",
                        "email_verified": true,
                        "login_method": "email",
                        "created_at": "2024-01-01T00:00:00Z",
                        "updated_at": "2024-01-01T00:00:00Z"
                    }
                }),
            )))
            .expect(1)
            .mount(&mock_server)
            .await;

        client
            .login(
                "sdk@test.dev".to_string(),
                "password".to_string(),
                Uuid::new_v4(),
            )
            .await
            .unwrap();

        let initial_user = client.get_user().await.unwrap();
        assert_eq!(initial_user.user.email.as_deref(), Some("sdk@test.dev"));

        client
            .session_manager
            .update_access_token(corrupted_access.to_string())
            .unwrap();

        let recovered_user = client.get_user().await.unwrap();

        assert_eq!(recovered_user.user.email.as_deref(), Some("sdk@test.dev"));
        assert_eq!(
            client.get_access_token().unwrap().as_deref(),
            Some(refreshed_access)
        );
        assert_eq!(
            client.get_refresh_token().unwrap().as_deref(),
            Some(refreshed_refresh)
        );
    }

    #[tokio::test]
    async fn test_refresh_reestablishes_attestation_without_sending_auth_headers() {
        let mock_server = MockServer::start().await;
        let client = OpenSecretClient::new(mock_server.uri()).unwrap();
        let server_secret_key = [11u8; 32];
        let server_public_key =
            x25519_dalek::PublicKey::from(&x25519_dalek::StaticSecret::from(server_secret_key));
        let session_key = [9u8; 32];
        let session_id = Uuid::new_v4().to_string();
        let refreshed_access = "refreshed_access";
        let refreshed_refresh = "refreshed_refresh";

        client
            .session_manager
            .set_tokens(
                "expired_access".to_string(),
                Some("refresh_token".to_string()),
            )
            .unwrap();

        Mock::given(method("GET"))
            .and(PathPrefixMatcher("/attestation/"))
            .respond_with(AttestationResponder {
                server_public_key: server_public_key.to_bytes(),
            })
            .expect(1)
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/key_exchange"))
            .and(MissingHeaderMatcher("authorization"))
            .respond_with(KeyExchangeResponder {
                server_secret_key,
                session_key,
                session_id: session_id.clone(),
            })
            .expect(1)
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/refresh"))
            .and(MissingHeaderMatcher("authorization"))
            .and(header("x-session-id", session_id.clone()))
            .respond_with(ResponseTemplate::new(200).set_body_json(encrypted_response(
                &session_key,
                &json!({
                    "access_token": refreshed_access,
                    "refresh_token": refreshed_refresh,
                }),
            )))
            .expect(1)
            .mount(&mock_server)
            .await;

        client.refresh_token().await.unwrap();

        assert_eq!(
            client.get_session_id().unwrap(),
            Some(Uuid::parse_str(&session_id).unwrap())
        );
        assert_eq!(
            client.get_access_token().unwrap().as_deref(),
            Some(refreshed_access)
        );
        assert_eq!(
            client.get_refresh_token().unwrap().as_deref(),
            Some(refreshed_refresh)
        );
    }
}
