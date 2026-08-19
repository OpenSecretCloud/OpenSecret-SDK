use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{DateTime, Utc};
use ring::signature::{UnparsedPublicKey, ED25519};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::{collections::HashSet, net::SocketAddr};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialUpdateResponse {
    #[serde(default)]
    pub message: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogoutRequest {
    pub refresh_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push_device_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedResponse<T> {
    pub encrypted: String,
    #[serde(skip)]
    _phantom: std::marker::PhantomData<T>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum NullableField<T> {
    #[default]
    Missing,
    Null,
    Value(T),
}

impl<T> NullableField<T> {
    pub fn is_missing(&self) -> bool {
        matches!(self, Self::Missing)
    }

    pub fn null() -> Self {
        Self::Null
    }

    pub fn value(value: T) -> Self {
        Self::Value(value)
    }
}

impl<T> Serialize for NullableField<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Missing | Self::Null => serializer.serialize_none(),
            Self::Value(value) => value.serialize(serializer),
        }
    }
}

impl<'de, T> Deserialize<'de> for NullableField<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(match Option::<T>::deserialize(deserializer)? {
            Some(value) => Self::Value(value),
            None => Self::Null,
        })
    }
}

// OAuth Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthInitRequest {
    pub client_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invite_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubAuthResponse {
    pub auth_url: String,
    #[serde(alias = "csrf_token")]
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleAuthResponse {
    pub auth_url: String,
    #[serde(alias = "csrf_token")]
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleAuthResponse {
    pub auth_url: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthCallbackRequest {
    pub code: String,
    pub state: String,
    pub invite_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleNativeSignInRequest {
    pub user_identifier: String,
    pub identity_token: String,
    pub client_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invite_code: Option<String>,
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

// Maple Remote Device Types

/// Current version of the Maple remote-device registration contract.
pub const MAPLE_DEVICE_PROTOCOL_VERSION: u16 = 1;

/// Current version of the server-reconstructed canonical registration transcript.
pub const MAPLE_DEVICE_TRANSCRIPT_VERSION: u16 = 1;

/// Default Maple device-directory page size applied by the server.
pub const MAPLE_DEVICE_LIST_DEFAULT_LIMIT: u16 = 25;

/// Largest Maple device-directory page accepted by the Rust SDK.
pub const MAPLE_DEVICE_LIST_MAX_LIMIT: u16 = 100;

/// Largest opaque Maple device-directory cursor accepted from a caller.
pub const MAPLE_DEVICE_LIST_MAX_CURSOR_BYTES: usize = 512;

/// Maximum relay routes carried in one signed Iroh endpoint address.
pub const MAPLE_DEVICE_MAX_RELAY_URLS: usize = 4;

/// Maximum direct socket routes carried in one signed Iroh endpoint address.
pub const MAPLE_DEVICE_MAX_DIRECT_ADDRESSES: usize = 16;

/// Maximum encoded length of one canonical relay URL.
pub const MAPLE_DEVICE_MAX_RELAY_URL_BYTES: usize = 512;

/// Maximum encoded length of one canonical direct `SocketAddr`.
pub const MAPLE_DEVICE_MAX_DIRECT_ADDRESS_BYTES: usize = 64;

/// Bounded, non-secret routing information for one Iroh endpoint.
///
/// This intentionally represents only Iroh relay URLs and direct socket
/// addresses. It cannot carry an endpoint secret key or an opaque/custom Iroh
/// transport address. Direct routes may be public, private, or loopback
/// unicast addresses. The device directory itself is encrypted; callers must
/// treat all routes as sensitive network metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MapleIrohEndpointAddr {
    pub relay_urls: Vec<String>,
    pub direct_addresses: Vec<String>,
}

struct ValidatedMapleDeviceTranscript<'a> {
    identity_public_key: [u8; 32],
    iroh_endpoint_key: [u8; 32],
    relay_urls: Vec<&'a str>,
    direct_addresses: Vec<&'a str>,
    capabilities: Vec<&'a str>,
}

impl MapleIrohEndpointAddr {
    /// Builds the canonical JSON form used by registration and list records.
    pub fn canonical_v1(
        relay_urls: Vec<String>,
        direct_addresses: Vec<String>,
    ) -> crate::Result<Self> {
        let candidate = Self {
            relay_urls,
            direct_addresses,
        };
        let (relay_urls, direct_addresses) = candidate.canonical_parts()?;
        Ok(Self {
            relay_urls: relay_urls.into_iter().map(str::to_owned).collect(),
            direct_addresses: direct_addresses.into_iter().map(str::to_owned).collect(),
        })
    }

    /// Validates that the DTO contains only bounded, non-secret Iroh routes.
    pub fn validate(&self) -> crate::Result<()> {
        let (relay_urls, direct_addresses) = self.canonical_parts()?;
        if relay_urls.as_slice() != self.relay_urls
            || direct_addresses.as_slice() != self.direct_addresses
        {
            return Err(crate::Error::Configuration(
                "Maple Iroh endpoint routes must use canonical sorted order".to_string(),
            ));
        }
        Ok(())
    }

    fn canonical_parts(&self) -> crate::Result<(Vec<&str>, Vec<&str>)> {
        if self.relay_urls.len() > MAPLE_DEVICE_MAX_RELAY_URLS {
            return Err(crate::Error::Configuration(format!(
                "Maple Iroh endpoint supports at most {MAPLE_DEVICE_MAX_RELAY_URLS} relay URLs"
            )));
        }
        if self.direct_addresses.len() > MAPLE_DEVICE_MAX_DIRECT_ADDRESSES {
            return Err(crate::Error::Configuration(format!(
                "Maple Iroh endpoint supports at most {MAPLE_DEVICE_MAX_DIRECT_ADDRESSES} direct addresses"
            )));
        }
        if self.relay_urls.is_empty() && self.direct_addresses.is_empty() {
            return Err(crate::Error::Configuration(
                "Maple Iroh endpoint must contain at least one relay URL or direct address"
                    .to_string(),
            ));
        }

        let mut relay_urls = Vec::with_capacity(self.relay_urls.len());
        let mut seen_relays = HashSet::with_capacity(self.relay_urls.len());
        for relay_url in &self.relay_urls {
            let parsed = reqwest::Url::parse(relay_url).map_err(|_| {
                crate::Error::Configuration("Maple Iroh relay URL is invalid".to_string())
            })?;
            if relay_url.is_empty()
                || relay_url.len() > MAPLE_DEVICE_MAX_RELAY_URL_BYTES
                || parsed.as_str() != relay_url
                || parsed.scheme() != "https"
                || parsed.host().is_none()
                || !parsed.username().is_empty()
                || parsed.password().is_some()
                || parsed.port() == Some(0)
                || parsed.query().is_some()
                || parsed.fragment().is_some()
                || !seen_relays.insert(relay_url.as_str())
            {
                return Err(crate::Error::Configuration(
                    "Maple Iroh relay URLs must be unique canonical HTTPS URLs without credentials, queries, or fragments"
                        .to_string(),
                ));
            }
            relay_urls.push(relay_url.as_str());
        }
        relay_urls.sort_unstable();

        let mut direct_addresses = Vec::with_capacity(self.direct_addresses.len());
        let mut seen_direct = HashSet::with_capacity(self.direct_addresses.len());
        for direct_address in &self.direct_addresses {
            let parsed = direct_address.parse::<SocketAddr>().map_err(|_| {
                crate::Error::Configuration(
                    "Maple Iroh direct address is not a socket address".to_string(),
                )
            })?;
            if direct_address.is_empty()
                || direct_address.len() > MAPLE_DEVICE_MAX_DIRECT_ADDRESS_BYTES
                || parsed.to_string() != *direct_address
                || parsed.port() == 0
                || parsed.ip().is_unspecified()
                || parsed.ip().is_multicast()
                || parsed.ip() == std::net::IpAddr::V4(std::net::Ipv4Addr::BROADCAST)
                || !seen_direct.insert(direct_address.as_str())
            {
                return Err(crate::Error::Configuration(
                    "Maple Iroh direct addresses must be unique canonical unicast SocketAddr values with nonzero ports"
                        .to_string(),
                ));
            }
            direct_addresses.push(direct_address.as_str());
        }
        direct_addresses.sort_unstable();

        Ok((relay_urls, direct_addresses))
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MapleDeviceIdentityAlgorithm {
    Ed25519,
}

impl MapleDeviceIdentityAlgorithm {
    fn as_str(self) -> &'static str {
        match self {
            Self::Ed25519 => "ed25519",
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MapleDevicePlatform {
    Macos,
    Windows,
    Linux,
    Ios,
    Android,
}

impl MapleDevicePlatform {
    fn as_str(self) -> &'static str {
        match self {
            Self::Macos => "macos",
            Self::Windows => "windows",
            Self::Linux => "linux",
            Self::Ios => "ios",
            Self::Android => "android",
        }
    }
}

/// A signed, caller-owned Maple device registration request.
///
/// These structured fields are the input to the server's canonical transcript
/// reconstruction. The SDK does not accept caller-chosen canonical bytes and
/// verifies the completed Ed25519 signature before sending the request.
/// Private-key generation and storage belong to the Maple installation and its
/// platform secure-storage implementation.
///
/// An installation ID whose remote enrollment lineage has been retired is
/// permanently unavailable for registration. Re-enrollment must create a
/// fresh installation ID and identity key/endpoint identity; changing mutable
/// display or routing fields does not revive the retired installation.
/// `operation_id` must be generated once by the caller and retained across
/// ambiguous outcomes; the encrypted client may automatically retry this exact
/// request after refreshing authentication or its attested session.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegisterMapleDeviceRequest {
    pub protocol_version: u16,
    pub transcript_version: u16,
    pub operation_id: Uuid,
    pub device_id: Uuid,
    pub installation_id: Uuid,
    /// Signed compare-and-swap precondition for an existing registration.
    /// `None` creates a registration; `Some(current_revision)` refreshes the
    /// same installation without permitting identity-key replacement.
    pub expected_revision: Option<i64>,
    /// Signed account security-epoch precondition. A newly bootstrapped
    /// account starts at epoch 1; callers must persist the latest verified
    /// epoch and refresh device state when the service rejects a stale value.
    pub known_security_epoch: u64,
    /// Signed stale-state precondition. The server derives account authority
    /// from authentication and rejects a mismatch; this field never selects a
    /// database namespace.
    pub asserted_account_id: Uuid,
    /// Signed public `org_projects.client_id` precondition for the authenticated
    /// OpenSecret project. The server derives project authority independently.
    pub asserted_project_id: Uuid,
    pub identity_algorithm: MapleDeviceIdentityAlgorithm,
    pub identity_public_key: String,
    pub iroh_endpoint_id: String,
    /// Monotonic endpoint lifecycle epoch, not an address-generation counter.
    /// Routine relay/direct address churn keeps this value stable and advances
    /// the registration `revision` through `expected_revision` CAS instead;
    /// v1 updates may retain or advance it for the same immutable identity but
    /// may never decrease it.
    pub endpoint_epoch: u64,
    pub iroh_endpoint_addr: MapleIrohEndpointAddr,
    pub platform: MapleDevicePlatform,
    pub display_name: String,
    pub capabilities: Vec<String>,
    pub signature: String,
}

impl std::fmt::Debug for RegisterMapleDeviceRequest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RegisterMapleDeviceRequest")
            .field("protocol_version", &self.protocol_version)
            .field("transcript_version", &self.transcript_version)
            .field("operation_id", &self.operation_id)
            .field("device_id", &self.device_id)
            .field("installation_id", &self.installation_id)
            .field("expected_revision", &self.expected_revision)
            .field("known_security_epoch", &self.known_security_epoch)
            .field("asserted_account_id", &self.asserted_account_id)
            .field("asserted_project_id", &self.asserted_project_id)
            .field("identity_algorithm", &self.identity_algorithm.as_str())
            .field("identity_public_key", &"<redacted>")
            .field("iroh_endpoint_id", &"<redacted>")
            .field("endpoint_epoch", &self.endpoint_epoch)
            .field("relay_url_count", &self.iroh_endpoint_addr.relay_urls.len())
            .field(
                "direct_address_count",
                &self.iroh_endpoint_addr.direct_addresses.len(),
            )
            .field("platform", &self.platform.as_str())
            .field("display_name", &"<redacted>")
            .field("capability_count", &self.capabilities.len())
            .field("signature", &"<redacted>")
            .finish()
    }
}

impl RegisterMapleDeviceRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        operation_id: Uuid,
        device_id: Uuid,
        installation_id: Uuid,
        expected_revision: Option<i64>,
        known_security_epoch: u64,
        asserted_account_id: Uuid,
        asserted_project_id: Uuid,
        identity_public_key: impl Into<String>,
        iroh_endpoint_id: impl Into<String>,
        endpoint_epoch: u64,
        iroh_endpoint_addr: MapleIrohEndpointAddr,
        platform: MapleDevicePlatform,
        display_name: impl Into<String>,
        capabilities: Vec<String>,
        signature: impl Into<String>,
    ) -> Self {
        Self::unsigned_v1(
            operation_id,
            device_id,
            installation_id,
            expected_revision,
            known_security_epoch,
            asserted_account_id,
            asserted_project_id,
            identity_public_key,
            iroh_endpoint_id,
            endpoint_epoch,
            iroh_endpoint_addr,
            platform,
            display_name,
            capabilities,
        )
        .with_signature(signature)
    }

    /// Creates the structured v1 registration before the caller signs it.
    ///
    /// Call [`Self::canonical_transcript`], sign those bytes with the
    /// installation's externally managed key, then attach the public signature
    /// with [`Self::with_signature`].
    #[allow(clippy::too_many_arguments)]
    pub fn unsigned_v1(
        operation_id: Uuid,
        device_id: Uuid,
        installation_id: Uuid,
        expected_revision: Option<i64>,
        known_security_epoch: u64,
        asserted_account_id: Uuid,
        asserted_project_id: Uuid,
        identity_public_key: impl Into<String>,
        iroh_endpoint_id: impl Into<String>,
        endpoint_epoch: u64,
        iroh_endpoint_addr: MapleIrohEndpointAddr,
        platform: MapleDevicePlatform,
        display_name: impl Into<String>,
        capabilities: Vec<String>,
    ) -> Self {
        Self {
            protocol_version: MAPLE_DEVICE_PROTOCOL_VERSION,
            transcript_version: MAPLE_DEVICE_TRANSCRIPT_VERSION,
            operation_id,
            device_id,
            installation_id,
            expected_revision,
            known_security_epoch,
            asserted_account_id,
            asserted_project_id,
            identity_algorithm: MapleDeviceIdentityAlgorithm::Ed25519,
            identity_public_key: identity_public_key.into(),
            iroh_endpoint_id: iroh_endpoint_id.into(),
            endpoint_epoch,
            iroh_endpoint_addr,
            platform,
            display_name: display_name.into(),
            capabilities,
            signature: String::new(),
        }
    }

    pub fn with_signature(mut self, signature: impl Into<String>) -> Self {
        self.signature = signature.into();
        self
    }

    /// Reconstructs the versioned canonical bytes this request must sign.
    ///
    /// This validates every signed public field, sorts capabilities without
    /// changing the caller's request object, and deliberately excludes
    /// `signature`. The OpenSecret service independently reconstructs these
    /// exact bytes after deriving and matching the account and project from the
    /// authenticated request context.
    pub fn canonical_transcript(&self) -> crate::Result<Vec<u8>> {
        let validated = self.validate_transcript_fields()?;
        let mut canonical = MapleCanonicalBytes::new("os.maple-device-registration.v1");
        canonical
            .append_u16(self.protocol_version)
            .append_u16(self.transcript_version)
            .append_uuid(self.asserted_account_id)
            .append_uuid(self.asserted_project_id)
            .append_u64(self.known_security_epoch)
            .append_uuid(self.operation_id)
            .append_uuid(self.device_id)
            .append_uuid(self.installation_id)
            .append_bool(self.expected_revision.is_some());
        if let Some(expected_revision) = self.expected_revision {
            canonical.append_i64(expected_revision);
        }
        canonical
            .append_str(self.identity_algorithm.as_str())
            .append_bytes(&validated.identity_public_key)
            .append_bytes(&validated.iroh_endpoint_key)
            .append_u64(self.endpoint_epoch)
            .append_u16(validated.relay_urls.len() as u16);
        for relay_url in validated.relay_urls {
            canonical.append_str(relay_url);
        }
        canonical.append_u16(validated.direct_addresses.len() as u16);
        for direct_address in validated.direct_addresses {
            canonical.append_str(direct_address);
        }
        canonical
            .append_str(self.platform.as_str())
            .append_str(&self.display_name)
            .append_u16(validated.capabilities.len() as u16);
        for capability in validated.capabilities {
            canonical.append_str(capability);
        }
        Ok(canonical.into_bytes())
    }

    /// Validates the complete signed request without generating or storing keys.
    pub fn validate(&self) -> crate::Result<()> {
        let transcript = self.canonical_transcript()?;
        let validated = self.validate_transcript_fields()?;
        let signature = decode_canonical_base64(&self.signature, "Maple device signature")?;
        if signature.len() != 64 {
            return Err(crate::Error::Configuration(
                "Maple device signature must contain exactly 64 bytes".to_string(),
            ));
        }
        UnparsedPublicKey::new(&ED25519, validated.identity_public_key)
            .verify(&transcript, &signature)
            .map_err(|_| {
                crate::Error::Configuration(
                    "Maple device signature does not verify for the signed registration"
                        .to_string(),
                )
            })?;
        Ok(())
    }

    fn validate_transcript_fields(&self) -> crate::Result<ValidatedMapleDeviceTranscript<'_>> {
        if self.protocol_version != MAPLE_DEVICE_PROTOCOL_VERSION {
            return Err(crate::Error::Configuration(format!(
                "Unsupported Maple device protocol version: {}",
                self.protocol_version
            )));
        }
        if self.transcript_version != MAPLE_DEVICE_TRANSCRIPT_VERSION {
            return Err(crate::Error::Configuration(format!(
                "Unsupported Maple device transcript version: {}",
                self.transcript_version
            )));
        }
        for (name, value) in [
            ("operation_id", self.operation_id),
            ("asserted_account_id", self.asserted_account_id),
            ("asserted_project_id", self.asserted_project_id),
            ("device_id", self.device_id),
            ("installation_id", self.installation_id),
        ] {
            if value.is_nil() {
                return Err(crate::Error::Configuration(format!(
                    "Maple device {name} must not be nil"
                )));
            }
        }
        if self
            .expected_revision
            .is_some_and(|revision| revision <= 0 || revision == i64::MAX)
        {
            return Err(crate::Error::Configuration(
                "Maple device expected revision must be positive and incrementable".to_string(),
            ));
        }
        if self.known_security_epoch == 0 || self.known_security_epoch > i64::MAX as u64 {
            return Err(crate::Error::Configuration(
                "Maple device known security epoch must be nonzero and supported by the service"
                    .to_string(),
            ));
        }
        if self.endpoint_epoch > i64::MAX as u64 {
            return Err(crate::Error::Configuration(
                "Maple device endpoint epoch exceeds the supported range".to_string(),
            ));
        }
        let (relay_urls, direct_addresses) = self.iroh_endpoint_addr.canonical_parts()?;

        let identity_public_key =
            decode_canonical_base64(&self.identity_public_key, "Maple device public key")?;
        let identity_public_key: [u8; 32] = identity_public_key.try_into().map_err(|_| {
            crate::Error::Configuration(
                "Maple device public key must contain exactly 32 bytes".to_string(),
            )
        })?;

        if self.iroh_endpoint_id.len() != 64
            || !self
                .iroh_endpoint_id
                .bytes()
                .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
        {
            return Err(crate::Error::Configuration(
                "Maple Iroh endpoint ID must be canonical lowercase 64-character hex".to_string(),
            ));
        }
        let iroh_endpoint_key: [u8; 32] = hex::decode(&self.iroh_endpoint_id)
            .map_err(|_| {
                crate::Error::Configuration("Maple Iroh endpoint ID is invalid hex".to_string())
            })?
            .try_into()
            .map_err(|_| {
                crate::Error::Configuration(
                    "Maple Iroh endpoint ID must contain exactly 32 bytes".to_string(),
                )
            })?;
        if iroh_endpoint_key != identity_public_key {
            return Err(crate::Error::Configuration(
                "Maple device public key must match the Iroh endpoint ID".to_string(),
            ));
        }

        if self.display_name.is_empty()
            || self.display_name.trim() != self.display_name
            || self.display_name.chars().count() > 80
            || self.display_name.chars().any(char::is_control)
        {
            return Err(crate::Error::Configuration(
                "Maple device display name must be trimmed, contain 1 to 80 characters, and contain no control characters"
                    .to_string(),
            ));
        }

        if self.capabilities.len() > 32 {
            return Err(crate::Error::Configuration(
                "Maple device registration supports at most 32 capabilities".to_string(),
            ));
        }
        let mut unique_capabilities = HashSet::with_capacity(self.capabilities.len());
        for capability in &self.capabilities {
            let valid = !capability.is_empty()
                && capability.len() <= 64
                && capability.bytes().all(|byte| {
                    byte.is_ascii_lowercase()
                        || byte.is_ascii_digit()
                        || matches!(byte, b'.' | b'_' | b':' | b'-')
                });
            if !valid || !unique_capabilities.insert(capability.as_str()) {
                return Err(crate::Error::Configuration(
                    "Maple device capabilities must be unique lowercase tokens of at most 64 bytes"
                        .to_string(),
                ));
            }
        }
        let mut capabilities = unique_capabilities.into_iter().collect::<Vec<_>>();
        capabilities.sort_unstable();

        Ok(ValidatedMapleDeviceTranscript {
            identity_public_key,
            iroh_endpoint_key,
            relay_urls,
            direct_addresses,
            capabilities,
        })
    }
}

fn decode_canonical_base64(value: &str, field_name: &str) -> crate::Result<Vec<u8>> {
    let decoded = BASE64.decode(value).map_err(|_| {
        crate::Error::Configuration(format!("{field_name} must be standard base64"))
    })?;
    if BASE64.encode(&decoded) != value {
        return Err(crate::Error::Configuration(format!(
            "{field_name} must use canonical padded standard base64"
        )));
    }
    Ok(decoded)
}

#[derive(Default)]
struct MapleCanonicalBytes {
    bytes: Vec<u8>,
}

impl MapleCanonicalBytes {
    fn new(domain: &str) -> Self {
        let mut canonical = Self::default();
        canonical.append_str(domain);
        canonical
    }

    fn append_str(&mut self, value: &str) -> &mut Self {
        self.append_field(b's', value.as_bytes())
    }

    fn append_bytes(&mut self, value: &[u8]) -> &mut Self {
        self.append_field(b'b', value)
    }

    fn append_u16(&mut self, value: u16) -> &mut Self {
        self.append_field(b'j', &value.to_be_bytes())
    }

    fn append_bool(&mut self, value: bool) -> &mut Self {
        self.append_field(b'?', &[u8::from(value)])
    }

    fn append_i64(&mut self, value: i64) -> &mut Self {
        self.append_field(b'l', &value.to_be_bytes())
    }

    fn append_u64(&mut self, value: u64) -> &mut Self {
        self.append_field(b'L', &value.to_be_bytes())
    }

    fn append_uuid(&mut self, value: Uuid) -> &mut Self {
        self.append_field(b'u', value.as_bytes())
    }

    fn append_field(&mut self, tag: u8, value: &[u8]) -> &mut Self {
        let len = u32::try_from(value.len()).expect("Maple canonical field length fits in u32");
        self.bytes.push(tag);
        self.bytes.extend_from_slice(&len.to_be_bytes());
        self.bytes.extend_from_slice(value);
        self
    }

    fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

/// Idempotent receipt returned by Maple device registration.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapleDeviceRegistrationResponse {
    pub protocol_version: u16,
    pub operation_id: Uuid,
    pub registration_id: Uuid,
    pub device_id: Uuid,
    pub revision: i64,
    pub accepted_at: DateTime<Utc>,
    pub security_epoch: u64,
    pub revocation_sync: crate::MapleRevocationSyncV1,
}

impl std::fmt::Debug for MapleDeviceRegistrationResponse {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MapleDeviceRegistrationResponse")
            .field("protocol_version", &self.protocol_version)
            .field("revision", &self.revision)
            .field("security_epoch", &self.security_epoch)
            .field("sync_status", &self.revocation_sync.status)
            .field("authority_material", &"[redacted]")
            .finish()
    }
}

/// Durable account-scoped record returned after device-directory decryption.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapleDevice {
    pub registration_id: Uuid,
    pub device_id: Uuid,
    pub installation_id: Uuid,
    pub identity_algorithm: MapleDeviceIdentityAlgorithm,
    pub identity_public_key: String,
    pub iroh_endpoint_id: String,
    pub endpoint_epoch: u64,
    pub iroh_endpoint_addr: MapleIrohEndpointAddr,
    pub platform: MapleDevicePlatform,
    pub display_name: String,
    pub capabilities: Vec<String>,
    pub revision: i64,
}

impl std::fmt::Debug for MapleDevice {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MapleDevice")
            .field("endpoint_epoch", &self.endpoint_epoch)
            .field("platform", &self.platform.as_str())
            .field("capability_count", &self.capabilities.len())
            .field("revision", &self.revision)
            .field("authority_material", &"[redacted]")
            .finish()
    }
}

impl MapleDevice {
    /// Validates a decrypted device-directory record before it is handed to
    /// Maple's transport layer.
    pub fn validate(&self) -> crate::Result<()> {
        if self.registration_id.is_nil()
            || self.device_id.is_nil()
            || self.installation_id.is_nil()
            || self.revision <= 0
            || self.endpoint_epoch > i64::MAX as u64
        {
            return Err(crate::Error::Configuration(
                "Maple device directory record contains invalid identifiers or revision"
                    .to_string(),
            ));
        }

        let identity_public_key =
            decode_canonical_base64(&self.identity_public_key, "Maple device public key")?;
        let identity_public_key: [u8; 32] = identity_public_key.try_into().map_err(|_| {
            crate::Error::Configuration(
                "Maple device public key must contain exactly 32 bytes".to_string(),
            )
        })?;
        if self.iroh_endpoint_id != hex::encode(identity_public_key) {
            return Err(crate::Error::Configuration(
                "Maple device directory public key does not match its Iroh endpoint ID".to_string(),
            ));
        }
        self.iroh_endpoint_addr.validate()?;
        if self.display_name.is_empty()
            || self.display_name.trim() != self.display_name
            || self.display_name.chars().count() > 80
            || self.display_name.chars().any(char::is_control)
        {
            return Err(crate::Error::Configuration(
                "Maple device directory display name must be trimmed, contain 1 to 80 characters, and contain no control characters"
                    .to_string(),
            ));
        }
        if self.capabilities.len() > 32 {
            return Err(crate::Error::Configuration(
                "Maple device directory record supports at most 32 capabilities".to_string(),
            ));
        }
        for capability in &self.capabilities {
            let valid = !capability.is_empty()
                && capability.len() <= 64
                && capability.bytes().all(|byte| {
                    byte.is_ascii_lowercase()
                        || byte.is_ascii_digit()
                        || matches!(byte, b'.' | b'_' | b':' | b'-')
                });
            if !valid {
                return Err(crate::Error::Configuration(
                    "Maple device directory capabilities must be lowercase tokens of at most 64 bytes"
                        .to_string(),
                ));
            }
        }
        if self.capabilities.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(crate::Error::Configuration(
                "Maple device directory capabilities must be in strictly ascending canonical order"
                    .to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct MapleDeviceListParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u16>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapleDeviceListResponse {
    pub protocol_version: u16,
    pub security_epoch: u64,
    pub devices: Vec<MapleDevice>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

impl std::fmt::Debug for MapleDeviceListResponse {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MapleDeviceListResponse")
            .field("protocol_version", &self.protocol_version)
            .field("security_epoch", &self.security_epoch)
            .field("device_count", &self.devices.len())
            .field("has_more", &self.has_more)
            .field("authority_material", &"[redacted]")
            .finish()
    }
}

// Push Notification Types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PushPlatform {
    Ios,
    Android,
}

impl PushPlatform {
    pub fn provider(self) -> PushProvider {
        match self {
            Self::Ios => PushProvider::Apns,
            Self::Android => PushProvider::Fcm,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PushProvider {
    Apns,
    Fcm,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum PushEnvironment {
    Dev,
    #[default]
    Prod,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PushKeyAlgorithm {
    P256EcdhV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegisterPushDeviceRequest {
    pub installation_id: Uuid,
    pub platform: PushPlatform,
    pub provider: PushProvider,
    pub environment: PushEnvironment,
    pub app_id: String,
    pub push_token: String,
    pub notification_public_key: String,
    pub key_algorithm: PushKeyAlgorithm,
    #[serde(default)]
    pub supports_encrypted_preview: bool,
    #[serde(default)]
    pub supports_background_processing: bool,
}

impl RegisterPushDeviceRequest {
    pub fn new(
        installation_id: Uuid,
        platform: PushPlatform,
        environment: PushEnvironment,
        app_id: impl Into<String>,
        push_token: impl Into<String>,
        notification_public_key: impl Into<String>,
    ) -> Self {
        Self {
            installation_id,
            platform,
            provider: platform.provider(),
            environment,
            app_id: app_id.into(),
            push_token: push_token.into(),
            notification_public_key: notification_public_key.into(),
            key_algorithm: PushKeyAlgorithm::P256EcdhV1,
            supports_encrypted_preview: false,
            supports_background_processing: false,
        }
    }

    pub fn supports_encrypted_preview(mut self, supports_encrypted_preview: bool) -> Self {
        self.supports_encrypted_preview = supports_encrypted_preview;
        self
    }

    pub fn supports_background_processing(mut self, supports_background_processing: bool) -> Self {
        self.supports_background_processing = supports_background_processing;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PushDevice {
    pub id: Uuid,
    pub object: String,
    pub installation_id: Uuid,
    pub platform: PushPlatform,
    pub provider: PushProvider,
    pub environment: PushEnvironment,
    pub app_id: String,
    pub key_algorithm: PushKeyAlgorithm,
    pub supports_encrypted_preview: bool,
    pub supports_background_processing: bool,
    pub last_seen_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PushDeviceListResponse {
    pub object: String,
    pub data: Vec<PushDevice>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeletedPushDeviceResponse {
    pub id: Uuid,
    pub object: String,
    pub deleted: bool,
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

// Account Management Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetRequest {
    pub email: String,
    pub hashed_secret: String,
    pub client_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetConfirmRequest {
    pub email: String,
    pub alphanumeric_code: String,
    pub plaintext_secret: String,
    pub new_password: String,
    pub client_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestVerificationCodeRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitiateAccountDeletionRequest {
    pub hashed_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmAccountDeletionRequest {
    pub confirmation_code: String,
    pub plaintext_secret: String,
}

// API Key Management Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyListResponse {
    pub keys: Vec<ApiKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyCreateRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyCreateResponse {
    pub key: String, // UUID format with dashes, only returned on creation
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: Uuid,
    pub object: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<Uuid>,
    pub pinned: bool,
    pub created_at: i64,
    pub last_activity_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConversationCreateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConversationUpdateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    #[serde(default, skip_serializing_if = "NullableField::is_missing")]
    pub project_id: NullableField<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned: Option<bool>,
}

impl ConversationUpdateRequest {
    pub fn is_empty(&self) -> bool {
        self.metadata.is_none() && self.project_id.is_missing() && self.pinned.is_none()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConversationsListParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unassigned_project: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationsListResponse {
    pub object: String,
    pub data: Vec<Conversation>,
    pub first_id: Option<Uuid>,
    pub last_id: Option<Uuid>,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationsDeleteResponse {
    pub object: String,
    pub deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchDeleteConversationsRequest {
    pub ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchDeleteItemResult {
    pub id: String,
    pub object: String,
    pub deleted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchDeleteConversationsResponse {
    pub object: String,
    pub data: Vec<BatchDeleteItemResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchUpdateConversationProjectRequest {
    pub ids: Vec<Uuid>,
    pub project_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchUpdateConversationProjectResponse {
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationProject {
    pub id: Uuid,
    pub object: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationProjectListItem {
    pub id: Uuid,
    pub object: String,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationProjectsListResponse {
    pub object: String,
    pub data: Vec<ConversationProjectListItem>,
    pub first_id: Option<Uuid>,
    pub last_id: Option<Uuid>,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationProjectCreateRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConversationProjectUpdateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "NullableField::is_missing")]
    pub instructions: NullableField<String>,
}

impl ConversationProjectUpdateRequest {
    pub fn is_empty(&self) -> bool {
        self.name.is_none() && self.instructions.is_missing()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConversationProjectListParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
}

// AI/OpenAI API Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    #[serde(default = "default_model_object")]
    pub object: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owned_by: Option<String>,
}

fn default_model_object() -> String {
    "model".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsResponse {
    pub object: String,
    pub data: Vec<Model>,
}

// Tool Calling Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: Function,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub parameters: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: FunctionCall,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    #[serde(default)]
    pub content: Value, // Now accepts both string and array formats
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<StreamOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamOptions {
    pub include_usage: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    pub index: i32,
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}

// Streaming types - transparent Value wrapper for full passthrough of any backend JSON.
// This avoids deserialization failures when LLMs send null fields in streaming tool_call
// deltas or introduce new fields the SDK doesn't know about yet.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ChatCompletionChunk(pub Value);

// Embeddings Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    pub input: EmbeddingInput,
    #[serde(default = "default_embedding_model")]
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

fn default_embedding_model() -> String {
    "nomic-embed-text".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EmbeddingInput {
    Single(String),
    Multiple(Vec<String>),
}

impl From<String> for EmbeddingInput {
    fn from(s: String) -> Self {
        EmbeddingInput::Single(s)
    }
}

impl From<&str> for EmbeddingInput {
    fn from(s: &str) -> Self {
        EmbeddingInput::Single(s.to_string())
    }
}

impl From<Vec<String>> for EmbeddingInput {
    fn from(v: Vec<String>) -> Self {
        EmbeddingInput::Multiple(v)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub object: String,
    pub data: Vec<EmbeddingData>,
    pub model: String,
    pub usage: EmbeddingUsage,
}

/// An embedding vector in the representation requested by `encoding_format`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EmbeddingVector {
    Float(Vec<f64>),
    Base64(String),
}

impl EmbeddingVector {
    pub fn as_floats(&self) -> Option<&[f64]> {
        match self {
            Self::Float(values) => Some(values),
            Self::Base64(_) => None,
        }
    }

    pub fn as_base64(&self) -> Option<&str> {
        match self {
            Self::Float(_) => None,
            Self::Base64(value) => Some(value),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingData {
    pub object: String,
    pub index: i32,
    pub embedding: EmbeddingVector,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingUsage {
    pub prompt_tokens: i32,
    pub total_tokens: i32,
}

// Web API Types

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebSearchWorkflow {
    #[default]
    Search,
    Images,
    Videos,
    News,
    Podcasts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebSearchTimeRelative {
    Day,
    Week,
    Month,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebSearchLens {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sites_included: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sites_excluded: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords_included: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords_excluded: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_after: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_before: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_relative: Option<WebSearchTimeRelative>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_region: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebSearchFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WebSearchRequest {
    pub query: String,
    /// Search result class. The server defaults to [`WebSearchWorkflow::Search`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow: Option<WebSearchWorkflow>,
    /// One-based result page, from 1 through 10.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u8>,
    /// Maximum results to return, from 1 through 50. The server defaults to 10.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u16>,
    /// Whether to omit potentially unsafe content. The server defaults to true.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safe_search: Option<bool>,
    /// Search collection timeout in seconds, from 0.5 through 4.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lens_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lens: Option<WebSearchLens>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<WebSearchFilters>,
}

impl WebSearchRequest {
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            workflow: None,
            page: None,
            limit: None,
            safe_search: None,
            timeout: None,
            lens_id: None,
            lens: None,
            filters: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebSearchResult {
    pub category: String,
    pub url: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebSearchResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    pub results: Vec<WebSearchResult>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WebExtractRequest {
    /// Public HTTPS URLs to extract, from 1 through 10 entries.
    pub urls: Vec<String>,
    /// Bulk extraction timeout in seconds, from 0.5 through 10.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<f64>,
}

impl WebExtractRequest {
    pub fn new(urls: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            urls: urls.into_iter().map(Into::into).collect(),
            timeout: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebExtractPageError {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebExtractPage {
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub markdown: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<WebExtractPageError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebExtractResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    pub pages: Vec<WebExtractPage>,
}

// Agent API Types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentChatRequest {
    pub input: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct InitMainAgentRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitMainAgentResponse {
    pub id: Uuid,
    pub object: String,
    pub kind: String,
    pub conversation_id: Uuid,
    pub display_name: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub messages: Vec<ConversationItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubagentRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    pub purpose: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SetMessageReactionRequest {
    pub emoji: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MainAgentResponse {
    pub id: Uuid,
    pub object: String,
    pub kind: String,
    pub conversation_id: Uuid,
    pub display_name: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentResponse {
    pub id: Uuid,
    pub object: String,
    pub kind: String,
    pub conversation_id: Uuid,
    pub display_name: String,
    pub purpose: String,
    pub created_by: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentItemsListParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListSubagentsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ConversationContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "input_text")]
    InputText { text: String },
    #[serde(rename = "output_text")]
    OutputText { text: String },
    #[serde(rename = "input_image")]
    InputImage { image_url: String },
    #[serde(rename = "input_file")]
    InputFile { filename: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ReasoningContentItem {
    #[serde(rename = "text")]
    Text { text: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ConversationItem {
    #[serde(rename = "message")]
    Message {
        id: Uuid,
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<String>,
        role: String,
        content: Vec<ConversationContent>,
        #[serde(skip_serializing_if = "Option::is_none")]
        reaction: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        created_at: Option<i64>,
    },
    #[serde(rename = "function_call")]
    FunctionToolCall {
        id: Uuid,
        call_id: Uuid,
        name: String,
        arguments: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        created_at: Option<i64>,
    },
    #[serde(rename = "function_call_output")]
    FunctionToolCallOutput {
        id: Uuid,
        call_id: Uuid,
        output: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        created_at: Option<i64>,
    },
    #[serde(rename = "reasoning")]
    Reasoning {
        id: Uuid,
        content: Vec<ReasoningContentItem>,
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        created_at: Option<i64>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentItemsListResponse {
    pub object: String,
    pub data: Vec<ConversationItem>,
    pub first_id: Option<Uuid>,
    pub last_id: Option<Uuid>,
    pub has_more: bool,
}

pub type ConversationItemsResponse = AgentItemsListResponse;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentListResponse {
    pub object: String,
    pub data: Vec<SubagentResponse>,
    pub first_id: Option<Uuid>,
    pub last_id: Option<Uuid>,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletedObjectResponse {
    pub id: Uuid,
    pub object: String,
    pub deleted: bool,
}

// Agent SSE event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessageEvent {
    pub message_id: Uuid,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentReactionEvent {
    pub item_id: Uuid,
    pub emoji: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTypingEvent {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDoneEvent {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentErrorEvent {
    pub error: String,
}

/// Parsed SSE event from the agent chat stream
#[derive(Debug, Clone)]
pub enum AgentSseEvent {
    Message(AgentMessageEvent),
    Reaction(AgentReactionEvent),
    Typing(AgentTypingEvent),
    Done(AgentDoneEvent),
    Error(AgentErrorEvent),
}

#[cfg(test)]
mod tests {
    use super::*;
    use ring::signature::{Ed25519KeyPair, KeyPair};
    use serde_json::json;
    use sha2::{Digest, Sha256};

    fn sign_maple_request(unsigned: RegisterMapleDeviceRequest) -> RegisterMapleDeviceRequest {
        let key = Ed25519KeyPair::from_seed_unchecked(&[17u8; 32]).unwrap();
        assert_eq!(
            key.public_key().as_ref(),
            BASE64.decode(&unsigned.identity_public_key).unwrap()
        );
        let signature = key.sign(&unsigned.canonical_transcript().unwrap());
        unsigned.with_signature(BASE64.encode(signature.as_ref()))
    }

    fn sample_iroh_addr() -> MapleIrohEndpointAddr {
        MapleIrohEndpointAddr::canonical_v1(
            vec![
                "https://use1-1.relay.n0.iroh.link./".to_string(),
                "https://euw1-1.relay.n0.iroh.link./".to_string(),
            ],
            vec![
                "[2001:db8::1]:4433".to_string(),
                "203.0.113.7:4433".to_string(),
            ],
        )
        .unwrap()
    }

    #[test]
    fn maple_device_v1_request_serializes_only_structured_transcript_fields() {
        let identity_public_key =
            hex::decode("d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c9778737")
                .unwrap();
        let identity_public_key_base64 = BASE64.encode(&identity_public_key);
        let iroh_endpoint_id = hex::encode(&identity_public_key);
        let request = sign_maple_request(RegisterMapleDeviceRequest::unsigned_v1(
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440003").unwrap(),
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440004").unwrap(),
            None,
            1,
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap(),
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440002").unwrap(),
            &identity_public_key_base64,
            &iroh_endpoint_id,
            7,
            sample_iroh_addr(),
            MapleDevicePlatform::Macos,
            "MacBook Pro",
            vec!["agent.control".to_string(), "agent.host".to_string()],
        ));
        let signature = request.signature.clone();

        let serialized_string = serde_json::to_string(&request).unwrap();
        let expected_serialized_string = format!(
            concat!(
                "{{\"protocol_version\":1,\"transcript_version\":1,",
                "\"operation_id\":\"550e8400-e29b-41d4-a716-446655440000\",",
                "\"device_id\":\"550e8400-e29b-41d4-a716-446655440003\",",
                "\"installation_id\":\"550e8400-e29b-41d4-a716-446655440004\",",
                "\"expected_revision\":null,\"known_security_epoch\":1,",
                "\"asserted_account_id\":\"550e8400-e29b-41d4-a716-446655440001\",",
                "\"asserted_project_id\":\"550e8400-e29b-41d4-a716-446655440002\",",
                "\"identity_algorithm\":\"ed25519\",",
                "\"identity_public_key\":\"0EqyMnQrtKs6E2i9RhXk5tAiSrcaAWuvhSCjMsl3hzc=\",",
                "\"iroh_endpoint_id\":\"d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c9778737\",",
                "\"endpoint_epoch\":7,",
                "\"iroh_endpoint_addr\":{{\"relay_urls\":[\"https://euw1-1.relay.n0.iroh.link./\",\"https://use1-1.relay.n0.iroh.link./\"],",
                "\"direct_addresses\":[\"203.0.113.7:4433\",\"[2001:db8::1]:4433\"]}},",
                "\"platform\":\"macos\",\"display_name\":\"MacBook Pro\",",
                "\"capabilities\":[\"agent.control\",\"agent.host\"],\"signature\":\"{}\"}}"
            ),
            signature
        );
        assert_eq!(serialized_string, expected_serialized_string);
        let expected_serialized = json!({
            "protocol_version": 1,
            "transcript_version": 1,
            "operation_id": "550e8400-e29b-41d4-a716-446655440000",
            "device_id": "550e8400-e29b-41d4-a716-446655440003",
            "installation_id": "550e8400-e29b-41d4-a716-446655440004",
            "expected_revision": null,
            "known_security_epoch": 1,
            "asserted_account_id": "550e8400-e29b-41d4-a716-446655440001",
            "asserted_project_id": "550e8400-e29b-41d4-a716-446655440002",
            "identity_algorithm": "ed25519",
            "identity_public_key": identity_public_key_base64,
            "iroh_endpoint_id": iroh_endpoint_id,
            "endpoint_epoch": 7,
            "iroh_endpoint_addr": {
                "relay_urls": [
                    "https://euw1-1.relay.n0.iroh.link./",
                    "https://use1-1.relay.n0.iroh.link./"
                ],
                "direct_addresses": ["203.0.113.7:4433", "[2001:db8::1]:4433"]
            },
            "platform": "macos",
            "display_name": "MacBook Pro",
            "capabilities": ["agent.control", "agent.host"],
            "signature": signature
        });
        let serialized: Value = serde_json::from_str(&serialized_string).unwrap();
        assert_eq!(serialized, expected_serialized);
        assert!(serialized.get("canonical_transcript").is_none());
        assert!(serialized.get("private_key").is_none());
    }

    #[test]
    fn maple_device_v1_canonical_transcript_matches_frozen_epoch_vectors() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../tests/fixtures/maple_pairing_v1_vectors.json"
        ))
        .unwrap();
        for (request_name, transcript_name, digest_name) in [
            (
                "register_device_request_epoch_1",
                "register_device_request_epoch_1_transcript_hex",
                "register_device_request_epoch_1_digest",
            ),
            (
                "register_device_request_epoch_4",
                "register_device_request_epoch_4_transcript_hex",
                "register_device_request_epoch_4_digest",
            ),
        ] {
            let request: RegisterMapleDeviceRequest =
                serde_json::from_value(fixture[request_name].clone()).unwrap();
            request.validate().unwrap();
            let transcript = request.canonical_transcript().unwrap();
            assert_eq!(
                hex::encode(&transcript),
                fixture[transcript_name].as_str().unwrap()
            );
            assert_eq!(
                BASE64.encode(Sha256::digest(&transcript)),
                fixture[digest_name].as_str().unwrap()
            );

            let mut tampered_epoch = request;
            tampered_epoch.known_security_epoch += 1;
            assert!(matches!(
                tampered_epoch.validate(),
                Err(crate::Error::Configuration(_))
            ));
        }
    }

    #[test]
    fn maple_device_v1_validation_rejects_identity_and_metadata_ambiguity() {
        let identity_public_key =
            hex::decode("d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c9778737")
                .unwrap();
        let valid = sign_maple_request(RegisterMapleDeviceRequest::unsigned_v1(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            1,
            Uuid::new_v4(),
            Uuid::new_v4(),
            BASE64.encode(&identity_public_key),
            hex::encode(&identity_public_key),
            1,
            sample_iroh_addr(),
            MapleDevicePlatform::Ios,
            "iPhone",
            vec!["agent.control".to_string()],
        ));
        valid.validate().unwrap();

        let invalid_requests = [
            RegisterMapleDeviceRequest {
                iroh_endpoint_id: hex::encode([8u8; 32]),
                ..valid.clone()
            },
            RegisterMapleDeviceRequest {
                display_name: " iPhone".to_string(),
                ..valid.clone()
            },
            RegisterMapleDeviceRequest {
                capabilities: vec!["agent.control".to_string(), "agent.control".to_string()],
                ..valid.clone()
            },
            RegisterMapleDeviceRequest {
                signature: BASE64.encode([9u8; 63]),
                ..valid
            },
        ];
        for request in invalid_requests {
            assert!(matches!(
                request.validate(),
                Err(crate::Error::Configuration(_))
            ));
        }
    }

    #[test]
    fn maple_device_registration_debug_redacts_identity_endpoint_routes_name_and_signature() {
        let identity_public_key =
            hex::decode("d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c9778737")
                .unwrap();
        let request = sign_maple_request(RegisterMapleDeviceRequest::unsigned_v1(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            1,
            Uuid::new_v4(),
            Uuid::new_v4(),
            BASE64.encode(&identity_public_key),
            hex::encode(&identity_public_key),
            1,
            sample_iroh_addr(),
            MapleDevicePlatform::Macos,
            "Sensitive Maple Host Name",
            vec!["agent.host".to_string()],
        ));

        let debug = format!("{request:?}");
        assert!(debug.contains("<redacted>"));
        assert!(!debug.contains(&request.identity_public_key));
        assert!(!debug.contains(&request.iroh_endpoint_id));
        assert!(!debug.contains(&request.display_name));
        assert!(!debug.contains(&request.signature));
        assert!(!request
            .iroh_endpoint_addr
            .relay_urls
            .iter()
            .any(|relay_url| debug.contains(relay_url)));
        assert!(!request
            .iroh_endpoint_addr
            .direct_addresses
            .iter()
            .any(|address| debug.contains(address)));
    }

    #[test]
    fn maple_device_directory_debug_redacts_identity_routes_names_and_cursor() {
        let identity_public_key =
            hex::decode("d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c9778737")
                .unwrap();
        let device = MapleDevice {
            registration_id: Uuid::new_v4(),
            device_id: Uuid::new_v4(),
            installation_id: Uuid::new_v4(),
            identity_algorithm: MapleDeviceIdentityAlgorithm::Ed25519,
            identity_public_key: BASE64.encode(&identity_public_key),
            iroh_endpoint_id: hex::encode(&identity_public_key),
            endpoint_epoch: 7,
            iroh_endpoint_addr: sample_iroh_addr(),
            platform: MapleDevicePlatform::Macos,
            display_name: "Sensitive Maple Host Name".to_string(),
            capabilities: vec!["agent.host".to_string()],
            revision: 3,
        };
        let cursor = "sensitive-device-directory-cursor".to_string();
        let page = MapleDeviceListResponse {
            protocol_version: 1,
            security_epoch: 4,
            devices: vec![device.clone()],
            next_cursor: Some(cursor.clone()),
            has_more: true,
        };

        for debug in [format!("{device:?}"), format!("{page:?}")] {
            assert!(debug.contains("[redacted]"));
            for secret in [
                device.registration_id.to_string(),
                device.device_id.to_string(),
                device.installation_id.to_string(),
                device.identity_public_key.clone(),
                device.iroh_endpoint_id.clone(),
                device.display_name.clone(),
                cursor.clone(),
            ] {
                assert!(!debug.contains(&secret));
            }
            assert!(!device
                .iroh_endpoint_addr
                .relay_urls
                .iter()
                .any(|route| debug.contains(route)));
            assert!(!device
                .iroh_endpoint_addr
                .direct_addresses
                .iter()
                .any(|route| debug.contains(route)));
        }
    }

    #[test]
    fn maple_iroh_endpoint_addr_is_canonical_bounded_and_contains_no_secret_material() {
        let canonical = sample_iroh_addr();
        assert_eq!(
            serde_json::to_value(&canonical).unwrap(),
            json!({
                "relay_urls": [
                    "https://euw1-1.relay.n0.iroh.link./",
                    "https://use1-1.relay.n0.iroh.link./"
                ],
                "direct_addresses": ["203.0.113.7:4433", "[2001:db8::1]:4433"]
            })
        );

        let unsorted = MapleIrohEndpointAddr {
            relay_urls: vec![
                "https://use1-1.relay.n0.iroh.link./".to_string(),
                "https://euw1-1.relay.n0.iroh.link./".to_string(),
            ],
            direct_addresses: vec![],
        };
        assert!(unsorted.validate().is_err());
        assert_eq!(
            MapleIrohEndpointAddr::canonical_v1(unsorted.relay_urls, unsorted.direct_addresses)
                .unwrap()
                .relay_urls,
            vec![
                "https://euw1-1.relay.n0.iroh.link./",
                "https://use1-1.relay.n0.iroh.link./"
            ]
        );

        for invalid in [
            MapleIrohEndpointAddr {
                relay_urls: vec![],
                direct_addresses: vec![],
            },
            MapleIrohEndpointAddr {
                relay_urls: vec!["http://relay.example/".to_string()],
                direct_addresses: vec![],
            },
            MapleIrohEndpointAddr {
                relay_urls: vec!["https://user@relay.example/".to_string()],
                direct_addresses: vec![],
            },
            MapleIrohEndpointAddr {
                relay_urls: vec![],
                direct_addresses: vec!["0.0.0.0:4433".to_string()],
            },
            MapleIrohEndpointAddr {
                relay_urls: vec![],
                direct_addresses: vec!["255.255.255.255:4433".to_string()],
            },
            MapleIrohEndpointAddr {
                relay_urls: vec![],
                direct_addresses: vec!["2001:db8::1:4433".to_string()],
            },
        ] {
            assert!(invalid.validate().is_err());
        }

        let literal: MapleIrohEndpointAddr = serde_json::from_value(json!({
            "relay_urls": ["https://relay.example/"],
            "direct_addresses": ["192.0.2.7:7777"]
        }))
        .unwrap();
        literal.validate().unwrap();
        assert!(serde_json::from_value::<MapleIrohEndpointAddr>(json!({
            "relay_urls": ["https://relay.example/"],
            "direct_addresses": [],
            "private_key": "must-not-fit-this-schema"
        }))
        .is_err());
    }

    #[test]
    fn maple_device_literal_json_preserves_endpoint_address_contract() {
        let literal = json!({
            "registration_id": "550e8400-e29b-41d4-a716-446655440005",
            "device_id": "550e8400-e29b-41d4-a716-446655440003",
            "installation_id": "550e8400-e29b-41d4-a716-446655440004",
            "identity_algorithm": "ed25519",
            "identity_public_key": "0EqyMnQrtKs6E2i9RhXk5tAiSrcaAWuvhSCjMsl3hzc=",
            "iroh_endpoint_id": "d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c9778737",
            "endpoint_epoch": 7,
            "iroh_endpoint_addr": {
                "relay_urls": ["https://relay.example/"],
                "direct_addresses": ["192.0.2.7:7777"]
            },
            "platform": "macos",
            "display_name": "MacBook Pro",
            "capabilities": ["agent.host"],
            "revision": 2
        });
        let device: MapleDevice = serde_json::from_value(literal.clone()).unwrap();
        device.validate().unwrap();
        assert_eq!(serde_json::to_value(device).unwrap(), literal);
    }

    #[test]
    fn maple_device_directory_capabilities_require_strict_ascending_order() {
        let identity_public_key =
            hex::decode("d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c9778737")
                .unwrap();
        let valid = MapleDevice {
            registration_id: Uuid::new_v4(),
            device_id: Uuid::new_v4(),
            installation_id: Uuid::new_v4(),
            identity_algorithm: MapleDeviceIdentityAlgorithm::Ed25519,
            identity_public_key: BASE64.encode(&identity_public_key),
            iroh_endpoint_id: hex::encode(&identity_public_key),
            endpoint_epoch: 1,
            iroh_endpoint_addr: sample_iroh_addr(),
            platform: MapleDevicePlatform::Macos,
            display_name: "Maple Host".to_string(),
            capabilities: vec!["agent.control".to_string(), "agent.host".to_string()],
            revision: 1,
        };
        valid.validate().unwrap();

        for capabilities in [
            vec!["agent.host".to_string(), "agent.control".to_string()],
            vec!["agent.control".to_string(), "agent.control".to_string()],
        ] {
            let invalid = MapleDevice {
                capabilities,
                ..valid.clone()
            };
            assert!(matches!(
                invalid.validate(),
                Err(crate::Error::Configuration(_))
            ));
        }
    }

    #[test]
    fn nullable_field_request_serialization_distinguishes_missing_and_null() {
        let conversation_update = ConversationUpdateRequest::default();
        assert!(conversation_update.is_empty());
        assert_eq!(
            serde_json::to_value(&conversation_update).unwrap(),
            json!({})
        );

        let conversation_update = ConversationUpdateRequest {
            project_id: NullableField::null(),
            ..Default::default()
        };
        assert!(!conversation_update.is_empty());
        assert_eq!(
            serde_json::to_value(&conversation_update).unwrap(),
            json!({ "project_id": null })
        );

        let project_update = ConversationProjectUpdateRequest {
            instructions: NullableField::null(),
            ..Default::default()
        };
        assert!(!project_update.is_empty());
        assert_eq!(
            serde_json::to_value(&project_update).unwrap(),
            json!({ "instructions": null })
        );
    }

    #[test]
    fn batch_update_project_request_serializes_none_as_explicit_null() {
        let request = BatchUpdateConversationProjectRequest {
            ids: vec![Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()],
            project_id: None,
        };

        assert_eq!(
            serde_json::to_value(&request).unwrap(),
            json!({
                "ids": ["550e8400-e29b-41d4-a716-446655440000"],
                "project_id": null
            })
        );
    }

    #[test]
    fn credential_update_response_tolerates_missing_message() {
        let response: CredentialUpdateResponse =
            serde_json::from_value(json!({ "access_token": "new-access" })).unwrap();

        assert_eq!(response.message, "");
        assert_eq!(response.access_token.as_deref(), Some("new-access"));
        assert_eq!(response.refresh_token, None);
    }

    #[test]
    fn web_requests_omit_unspecified_options() {
        let search = WebSearchRequest::new("maple privacy");
        assert_eq!(
            serde_json::to_value(search).unwrap(),
            json!({ "query": "maple privacy" })
        );

        let extract = WebExtractRequest::new(["https://example.com/article"]);
        assert_eq!(
            serde_json::to_value(extract).unwrap(),
            json!({ "urls": ["https://example.com/article"] })
        );
    }

    #[test]
    fn web_responses_tolerate_absent_optional_fields() {
        let search: WebSearchResponse = serde_json::from_value(json!({
            "results": [{
                "category": "search",
                "url": "https://example.com",
                "title": "Example"
            }]
        }))
        .unwrap();
        assert_eq!(search.trace_id, None);
        assert_eq!(search.results[0].snippet, None);
        assert_eq!(search.results[0].published_at, None);

        let extract: WebExtractResponse = serde_json::from_value(json!({
            "pages": [{ "url": "https://example.com" }]
        }))
        .unwrap();
        assert_eq!(extract.trace_id, None);
        assert_eq!(extract.pages[0].markdown, None);
        assert_eq!(extract.pages[0].error, None);
    }

    #[test]
    fn embedding_response_deserializes_float_vectors() {
        let response: EmbeddingResponse = serde_json::from_value(json!({
            "object": "list",
            "data": [{
                "object": "embedding",
                "index": 0,
                "embedding": [0.1, 0.2, 0.3]
            }],
            "model": "nomic-embed-text",
            "usage": { "prompt_tokens": 4, "total_tokens": 4 }
        }))
        .unwrap();

        assert_eq!(
            response.data[0].embedding,
            EmbeddingVector::Float(vec![0.1, 0.2, 0.3])
        );
        assert_eq!(
            response.data[0].embedding.as_floats(),
            Some([0.1, 0.2, 0.3].as_slice())
        );
        assert_eq!(response.data[0].embedding.as_base64(), None);
        assert_eq!(
            serde_json::to_value(&response).unwrap()["data"][0]["embedding"],
            json!([0.1, 0.2, 0.3])
        );
    }

    #[test]
    fn embedding_response_deserializes_base64_vectors() {
        let response: EmbeddingResponse = serde_json::from_value(json!({
            "object": "list",
            "data": [{
                "object": "embedding",
                "index": 0,
                "embedding": "AQIDBA=="
            }],
            "model": "nomic-embed-text",
            "usage": { "prompt_tokens": 4, "total_tokens": 4 }
        }))
        .unwrap();

        assert_eq!(
            response.data[0].embedding,
            EmbeddingVector::Base64("AQIDBA==".to_string())
        );
        assert_eq!(response.data[0].embedding.as_floats(), None);
        assert_eq!(response.data[0].embedding.as_base64(), Some("AQIDBA=="));
        assert_eq!(
            serde_json::to_value(&response).unwrap()["data"][0]["embedding"],
            json!("AQIDBA==")
        );
    }

    #[test]
    fn embedding_response_rejects_unknown_vector_representations() {
        let response = serde_json::from_value::<EmbeddingResponse>(json!({
            "object": "list",
            "data": [{
                "object": "embedding",
                "index": 0,
                "embedding": { "values": [0.1, 0.2, 0.3] }
            }],
            "model": "nomic-embed-text",
            "usage": { "prompt_tokens": 4, "total_tokens": 4 }
        }));

        assert!(response.is_err());
    }
}
