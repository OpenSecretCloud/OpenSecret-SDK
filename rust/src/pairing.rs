//! Maple's encrypted, account-scoped remote pairing control-plane contract.
//!
//! The service mediates explicit, one-way controller-to-host approval. It does
//! not participate in the Iroh data path and its artifacts do not become
//! authority until their issuer signatures have been checked against an
//! explicitly supplied trusted key set. Private device keys remain owned by
//! the calling Maple installation and are never accepted by this module.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::{Error, Result};

pub const MAPLE_PAIRING_PROTOCOL_VERSION: u16 = 1;
pub const MAPLE_PAIRING_TRANSCRIPT_VERSION: u16 = 1;
pub const MAPLE_PAIRING_ARTIFACT_VERSION: u16 = 1;
pub const MAPLE_PAIRING_LIST_DEFAULT_LIMIT: u16 = 25;
pub const MAPLE_PAIRING_LIST_MAX_LIMIT: u16 = 100;
pub const MAPLE_PAIRING_CURSOR_MAX_BYTES: usize = 512;
pub const MAPLE_PAIRING_MAX_TICKET_TTL_MS: i64 = 10 * 60 * 1000;
pub const MAPLE_PAIRING_MAX_CLOCK_SKEW_MS: i64 = 30 * 1000;
pub const MAPLE_RESET_CLEAR_MAX_ADMISSIONS: u16 = 128;

const ED25519_PUBLIC_KEY_BYTES: usize = 32;
const ED25519_SIGNATURE_BYTES: usize = 64;
const NONCE_BYTES: usize = 32;
const DIGEST_BYTES: usize = 32;
const MAX_REASON_CODE_BYTES: usize = 64;
const MAX_ISSUER_KEY_ID_BYTES: usize = 64;
const RESET_CLEAR_ADMISSION_SET_DOMAIN: &str = "os.maple-reset-clear-admission-set.v1";
const RESET_CLEAR_INSTRUCTION_MATERIAL_DOMAIN: &str =
    "os.maple-reset-clear-instruction-material.v1";
const RESET_CLEAR_CHAIN_DOMAIN: &str = "os.maple-reset-clear-chain.v1";
const RESET_CLEAR_REQUIRED_DOMAIN: &str = "os.maple-reset-clear-required.v1";

macro_rules! redacted_debug {
    ($type:ty) => {
        impl std::fmt::Debug for $type {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter
                    .debug_struct(stringify!($type))
                    .field("authority_material", &"[redacted]")
                    .finish()
            }
        }
    };
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MaplePairingDirection {
    ControllerToHost,
}

impl MaplePairingDirection {
    fn as_str(self) -> &'static str {
        match self {
            Self::ControllerToHost => "controller_to_host",
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MaplePairingState {
    Pending,
    AwaitingHostCommit,
    Active,
    Expired,
    Revoked,
}

impl MaplePairingState {
    fn canonical_order(self) -> u8 {
        match self {
            Self::Pending => 0,
            Self::AwaitingHostCommit => 1,
            Self::Active => 2,
            Self::Expired => 3,
            Self::Revoked => 4,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::AwaitingHostCommit => "awaiting_host_commit",
            Self::Active => "active",
            Self::Expired => "expired",
            Self::Revoked => "revoked",
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MaplePairingRole {
    Controller,
    Host,
}

impl MaplePairingRole {
    fn as_str(self) -> &'static str {
        match self {
            Self::Controller => "controller",
            Self::Host => "host",
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MaplePairingIdentityAlgorithm {
    Ed25519,
}

impl MaplePairingIdentityAlgorithm {
    fn as_str(self) -> &'static str {
        match self {
            Self::Ed25519 => "ed25519",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CreateMaplePairingRequest {
    pub protocol_version: u16,
    pub transcript_version: u16,
    pub operation_id: Uuid,
    pub asserted_account_id: Uuid,
    pub asserted_project_id: Uuid,
    pub controller_registration_id: Uuid,
    pub controller_device_id: Uuid,
    pub controller_installation_id: Uuid,
    pub controller_endpoint_id: String,
    pub controller_endpoint_epoch: u64,
    pub host_registration_id: Uuid,
    pub host_device_id: Uuid,
    pub host_installation_id: Uuid,
    pub host_endpoint_id: String,
    pub host_endpoint_epoch: u64,
    pub direction: MaplePairingDirection,
    pub execution_target_id: Uuid,
    pub pairing_request_nonce: String,
    pub protocol_min: u16,
    pub protocol_max: u16,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ListMaplePairingsRequest {
    pub protocol_version: u16,
    pub transcript_version: u16,
    pub query_id: Uuid,
    pub asserted_account_id: Uuid,
    pub asserted_project_id: Uuid,
    pub actor_registration_id: Uuid,
    pub role: MaplePairingRole,
    pub states: Vec<MaplePairingState>,
    pub cursor: Option<String>,
    pub limit: Option<u16>,
    pub signature: String,
}

impl ListMaplePairingsRequest {
    pub fn canonical_states(mut states: Vec<MaplePairingState>) -> Result<Vec<MaplePairingState>> {
        if states.is_empty() {
            return Err(Error::Configuration(
                "Maple pairing state filter must not be empty".to_string(),
            ));
        }
        states.sort_by_key(|state| state.canonical_order());
        if states.windows(2).any(|states| states[0] == states[1]) {
            return Err(Error::Configuration(
                "Maple pairing state filter must not contain duplicates".to_string(),
            ));
        }
        Ok(states)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MaplePairingStatusRequest {
    pub protocol_version: u16,
    pub transcript_version: u16,
    pub query_id: Uuid,
    pub asserted_account_id: Uuid,
    pub asserted_project_id: Uuid,
    pub actor_registration_id: Uuid,
    pub pair_id: Uuid,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ApproveMaplePairingRequest {
    pub protocol_version: u16,
    pub transcript_version: u16,
    pub operation_id: Uuid,
    pub asserted_account_id: Uuid,
    pub asserted_project_id: Uuid,
    pub host_registration_id: Uuid,
    pub pairing_request_id: Uuid,
    pub pair_id: Uuid,
    pub expected_pairing_revision: i64,
    pub pairing_incarnation: u64,
    pub revocation_stream_id: Uuid,
    pub revocation_stream_generation: u64,
    pub request_ticket_digest: String,
    pub host_approval_nonce: String,
    pub approved_protocol_min: u16,
    pub approved_protocol_max: u16,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ConfirmMaplePairingRequest {
    pub protocol_version: u16,
    pub transcript_version: u16,
    pub operation_id: Uuid,
    pub asserted_account_id: Uuid,
    pub asserted_project_id: Uuid,
    pub host_registration_id: Uuid,
    pub pairing_request_id: Uuid,
    pub pair_id: Uuid,
    pub expected_pairing_revision: i64,
    pub pairing_incarnation: u64,
    pub pair_authorization_digest: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RevokeMaplePairingRequest {
    pub protocol_version: u16,
    pub transcript_version: u16,
    pub operation_id: Uuid,
    pub asserted_account_id: Uuid,
    pub asserted_project_id: Uuid,
    pub actor_registration_id: Uuid,
    pub actor_role: MaplePairingRole,
    pub pairing_request_id: Uuid,
    pub pair_id: Uuid,
    pub expected_pairing_revision: i64,
    pub pairing_incarnation: u64,
    pub revocation_stream_id: Uuid,
    pub revocation_stream_generation: u64,
    pub reason_code: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ListMaplePairingRevocationsRequest {
    pub protocol_version: u16,
    pub transcript_version: u16,
    pub query_id: Uuid,
    pub asserted_account_id: Uuid,
    pub asserted_project_id: Uuid,
    pub host_registration_id: Uuid,
    pub revocation_stream_id: Uuid,
    pub revocation_stream_generation: u64,
    pub after_issuer_sequence: u64,
    pub limit: Option<u16>,
    pub signature: String,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AckMaplePairingRevocationRequest {
    pub protocol_version: u16,
    pub transcript_version: u16,
    pub operation_id: Uuid,
    pub asserted_account_id: Uuid,
    pub asserted_project_id: Uuid,
    pub host_registration_id: Uuid,
    pub revocation_stream_id: Uuid,
    pub revocation_stream_generation: u64,
    pub event_id: Uuid,
    pub issuer_sequence: u64,
    pub event_digest: String,
    pub expected_previous_issuer_sequence: u64,
    pub signature: String,
}

impl std::fmt::Debug for AckMaplePairingRevocationRequest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AckMaplePairingRevocationRequest")
            .field("protocol_version", &self.protocol_version)
            .field("transcript_version", &self.transcript_version)
            .field("issuer_sequence", &self.issuer_sequence)
            .field(
                "expected_previous_issuer_sequence",
                &self.expected_previous_issuer_sequence,
            )
            .field("authority_material", &"[redacted]")
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MaplePairingDeviceClaimV1 {
    pub registration_id: Uuid,
    pub device_id: Uuid,
    pub installation_id: Uuid,
    pub identity_algorithm: MaplePairingIdentityAlgorithm,
    pub identity_public_key: String,
    pub endpoint_id: String,
    pub endpoint_epoch: u64,
}

/// Public wire name used by the OpenSecret service. The older SDK-specific
/// name remains available for source compatibility while the pairing API is
/// still unreleased.
pub type MapleDeviceClaimV1 = MaplePairingDeviceClaimV1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MaplePairRequestTicketV1 {
    pub artifact_version: u16,
    pub subject_account_id: Uuid,
    pub subject_project_id: Uuid,
    pub pairing_request_id: Uuid,
    pub pair_id: Uuid,
    pub direction: MaplePairingDirection,
    pub execution_target_id: Uuid,
    pub controller: MaplePairingDeviceClaimV1,
    pub host: MaplePairingDeviceClaimV1,
    pub pairing_request_nonce: String,
    pub controller_request_operation_id: Uuid,
    pub controller_request_digest: String,
    pub controller_request_signature: String,
    pub pairing_incarnation: u64,
    pub protocol_min: u16,
    pub protocol_max: u16,
    pub created_at_unix_ms: i64,
    pub expires_at_unix_ms: i64,
    pub issuer_key_id: String,
    pub issuer_signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MaplePairAuthorizationV1 {
    pub artifact_version: u16,
    pub subject_account_id: Uuid,
    pub subject_project_id: Uuid,
    pub pairing_request_id: Uuid,
    pub pair_id: Uuid,
    pub direction: MaplePairingDirection,
    pub execution_target_id: Uuid,
    pub controller: MaplePairingDeviceClaimV1,
    pub host: MaplePairingDeviceClaimV1,
    pub pairing_request_nonce: String,
    pub controller_request_operation_id: Uuid,
    pub controller_request_digest: String,
    pub controller_request_signature: String,
    pub request_ticket_digest: String,
    pub host_approval_operation_id: Uuid,
    pub host_approval_expected_pairing_revision: i64,
    pub host_approval_nonce: String,
    pub host_approval_digest: String,
    pub host_approval_signature: String,
    pub pairing_incarnation: u64,
    pub revocation_stream_id: Uuid,
    pub revocation_stream_generation: u64,
    pub protocol_min: u16,
    pub protocol_max: u16,
    pub approved_at_unix_ms: i64,
    pub issuer_key_id: String,
    pub issuer_signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MaplePairRevocationV1 {
    pub artifact_version: u16,
    pub event_id: Uuid,
    pub subject_account_id: Uuid,
    pub subject_project_id: Uuid,
    pub recipient_host_registration_id: Uuid,
    pub issuer_sequence: u64,
    pub revocation_stream_id: Uuid,
    pub revocation_stream_generation: u64,
    pub pairing_request_id: Uuid,
    pub pair_id: Uuid,
    pub direction: MaplePairingDirection,
    pub execution_target_id: Uuid,
    pub controller: MaplePairingDeviceClaimV1,
    pub host: MaplePairingDeviceClaimV1,
    pub pairing_incarnation: u64,
    pub pair_authorization_digest: String,
    pub revoked_by_registration_id: Uuid,
    pub revoked_by_role: MaplePairingRole,
    pub reason_code: String,
    pub revoked_at_unix_ms: i64,
    pub issuer_key_id: String,
    pub issuer_signature: String,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MapleResetClearScopeV1 {
    AllPairAuthorizationsForAccountProjectHostInstallation,
}

impl MapleResetClearScopeV1 {
    fn as_str(self) -> &'static str {
        match self {
            Self::AllPairAuthorizationsForAccountProjectHostInstallation => {
                "all_pair_authorizations_for_account_project_host_installation"
            }
        }
    }
}

/// One private admission-set leaf supplied only to the canonical digest
/// helper. Reset-clear artifacts expose only the bounded count and digest.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MapleResetClearAdmissionLeafV1 {
    pub pair_id: Uuid,
    pub pairing_incarnation: u64,
    pub pair_authorization_digest: [u8; 32],
}

impl std::fmt::Debug for MapleResetClearAdmissionLeafV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MapleResetClearAdmissionLeafV1")
            .field("authority_material", &"[redacted]")
            .finish()
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MapleResetClearRequiredV1 {
    pub artifact_version: u16,
    pub event_id: Uuid,
    pub reset_id: Uuid,
    pub reset_generation: u64,
    pub cumulative_reset_count: u64,
    pub source_security_epoch: u64,
    pub security_epoch: u64,
    pub subject_account_id: Uuid,
    pub subject_project_id: Uuid,
    pub recipient_host_registration_id: Uuid,
    pub host: MapleDeviceClaimV1,
    pub issuer_sequence: u64,
    pub source_revocation_stream_id: Uuid,
    pub source_revocation_stream_generation: u64,
    pub revocation_stream_id: Uuid,
    pub revocation_stream_generation: u64,
    pub clear_scope: MapleResetClearScopeV1,
    pub admission_count: u16,
    pub admission_set_digest: String,
    pub previous_reset_clear_event_id: Option<Uuid>,
    pub previous_instruction_material_digest: Option<String>,
    pub previous_chain_digest: Option<String>,
    pub reset_at_unix_ms: i64,
    pub instruction_material_digest: String,
    pub chain_digest: String,
    pub issuer_key_id: String,
    pub issuer_signature: String,
}

impl std::fmt::Debug for MapleResetClearRequiredV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MapleResetClearRequiredV1")
            .field("artifact_version", &self.artifact_version)
            .field("reset_generation", &self.reset_generation)
            .field("cumulative_reset_count", &self.cumulative_reset_count)
            .field("source_security_epoch", &self.source_security_epoch)
            .field("security_epoch", &self.security_epoch)
            .field("admission_count", &self.admission_count)
            .field(
                "has_previous_reset",
                &self.previous_reset_clear_event_id.is_some(),
            )
            .field("authority_material", &"[redacted]")
            .finish()
    }
}

#[non_exhaustive]
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "event_type", content = "event", rename_all = "snake_case")]
pub enum MapleRevocationStreamEventV1 {
    PairRevocation(MaplePairRevocationV1),
    ResetClearRequired(MapleResetClearRequiredV1),
}

impl std::fmt::Debug for MapleRevocationStreamEventV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let event_type = match self {
            Self::PairRevocation(_) => "pair_revocation",
            Self::ResetClearRequired(_) => "reset_clear_required",
        };
        formatter
            .debug_struct("MapleRevocationStreamEventV1")
            .field("event_type", &event_type)
            .field("authority_material", &"[redacted]")
            .finish()
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MapleRevocationSyncStatusV1 {
    Ready,
    RevocationsPending,
    ResetClearRequired,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MapleRevocationSyncV1 {
    pub security_epoch: u64,
    pub status: MapleRevocationSyncStatusV1,
    pub stream_checkpoint: MapleRevocationStreamCheckpointV1,
    pub reset_clear_instruction: Option<MapleResetClearRequiredV1>,
}

impl std::fmt::Debug for MapleRevocationSyncV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MapleRevocationSyncV1")
            .field("security_epoch", &self.security_epoch)
            .field("status", &self.status)
            .field(
                "has_reset_clear_instruction",
                &self.reset_clear_instruction.is_some(),
            )
            .field("authority_material", &"[redacted]")
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MaplePairingStatusV1 {
    pub pairing_request_id: Uuid,
    pub pair_id: Uuid,
    pub state: MaplePairingState,
    pub revision: i64,
    pub pairing_incarnation: u64,
    pub revocation_stream_id: Option<Uuid>,
    pub revocation_stream_generation: Option<u64>,
    pub direction: MaplePairingDirection,
    pub execution_target_id: Uuid,
    pub controller_registration_id: Uuid,
    pub host_registration_id: Uuid,
    pub created_at_unix_ms: i64,
    pub expires_at_unix_ms: i64,
    pub approved_at_unix_ms: Option<i64>,
    pub activated_at_unix_ms: Option<i64>,
    pub revoked_at_unix_ms: Option<i64>,
    pub request_ticket: Option<MaplePairRequestTicketV1>,
    pub pair_authorization: Option<MaplePairAuthorizationV1>,
    pub revocation: Option<MaplePairRevocationV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MaplePairingMutationResponse {
    pub protocol_version: u16,
    pub operation_id: Uuid,
    pub pairing: MaplePairingStatusV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MaplePairingListResponse {
    pub protocol_version: u16,
    pub query_id: Uuid,
    pub role: MaplePairingRole,
    pub pairings: Vec<MaplePairingStatusV1>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MaplePairingStatusResponse {
    pub protocol_version: u16,
    pub query_id: Uuid,
    pub pairing: MaplePairingStatusV1,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MaplePairingRevocationListResponse {
    pub protocol_version: u16,
    pub query_id: Uuid,
    pub revocation_sync: MapleRevocationSyncV1,
    pub events: Vec<MapleRevocationStreamEventV1>,
    pub next_after_issuer_sequence: u64,
    pub has_more: bool,
}

impl std::fmt::Debug for MaplePairingRevocationListResponse {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MaplePairingRevocationListResponse")
            .field("protocol_version", &self.protocol_version)
            .field("event_count", &self.events.len())
            .field("has_more", &self.has_more)
            .field("authority_material", &"[redacted]")
            .finish()
    }
}

/// Wire receipt for one accepted revocation acknowledgement operation.
///
/// Its signed checkpoint is bound to the operation at acceptance time. It is
/// not a current readiness snapshot: an exact operation replay can return this
/// historical receipt after newer revocations have been issued.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MaplePairingRevocationAckResponse {
    pub protocol_version: u16,
    pub operation_id: Uuid,
    pub host_registration_id: Uuid,
    pub stream_checkpoint: MapleRevocationStreamCheckpointV1,
    pub event_id: Uuid,
    pub issuer_sequence: u64,
    pub last_acked_issuer_sequence: u64,
    pub accepted_at_unix_ms: i64,
}

impl std::fmt::Debug for MaplePairingRevocationAckResponse {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MaplePairingRevocationAckResponse")
            .field("protocol_version", &self.protocol_version)
            .field("issuer_sequence", &self.issuer_sequence)
            .field(
                "last_acked_issuer_sequence",
                &self.last_acked_issuer_sequence,
            )
            .field("authority_material", &"[redacted]")
            .finish()
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MapleRevocationStreamCheckpointV1 {
    pub artifact_version: u16,
    pub subject_account_id: Uuid,
    pub subject_project_id: Uuid,
    pub host: MaplePairingDeviceClaimV1,
    pub security_epoch: u64,
    pub revocation_stream_id: Uuid,
    pub revocation_stream_generation: u64,
    pub last_issued_issuer_sequence: u64,
    pub last_acked_issuer_sequence: u64,
    pub issuer_key_id: String,
    pub issuer_signature: String,
}

impl std::fmt::Debug for MapleRevocationStreamCheckpointV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MapleRevocationStreamCheckpointV1")
            .field("artifact_version", &self.artifact_version)
            .field("security_epoch", &self.security_epoch)
            .field(
                "last_issued_issuer_sequence",
                &self.last_issued_issuer_sequence,
            )
            .field(
                "last_acked_issuer_sequence",
                &self.last_acked_issuer_sequence,
            )
            .field("authority_material", &"[redacted]")
            .finish()
    }
}

/// Opaque approval request prepared from a verified ticket and a durably
/// reconciled host revocation namespace. Only this type is accepted by the SDK
/// approval transport.
#[derive(Clone)]
pub struct PreparedMaplePairingApprovalV1 {
    request: ApproveMaplePairingRequest,
}

redacted_debug!(PreparedMaplePairingApprovalV1);

impl PreparedMaplePairingApprovalV1 {
    pub fn as_inner(&self) -> &ApproveMaplePairingRequest {
        &self.request
    }

    pub fn canonical_transcript(&self) -> Result<Vec<u8>> {
        self.request.canonical_transcript()
    }

    pub fn with_signature(mut self, signature: impl Into<String>) -> Self {
        self.request.signature = signature.into();
        self
    }
}

/// Opaque confirmation request prepared only after a verified host status and
/// caller-asserted durable local authorization commit.
#[derive(Clone)]
pub struct PreparedMaplePairingHostCommitV1 {
    request: ConfirmMaplePairingRequest,
}

redacted_debug!(PreparedMaplePairingHostCommitV1);

impl PreparedMaplePairingHostCommitV1 {
    pub fn as_inner(&self) -> &ConfirmMaplePairingRequest {
        &self.request
    }

    pub fn canonical_transcript(&self) -> Result<Vec<u8>> {
        self.request.canonical_transcript()
    }

    pub fn with_signature(mut self, signature: impl Into<String>) -> Self {
        self.request.signature = signature.into();
        self
    }
}

/// Opaque ACK request prepared only from a revocation bound to the host's
/// verified authorization and durably reconciled namespace after local apply.
#[derive(Clone)]
pub struct PreparedMaplePairingRevocationAckV1 {
    request: AckMaplePairingRevocationRequest,
}

redacted_debug!(PreparedMaplePairingRevocationAckV1);

impl PreparedMaplePairingRevocationAckV1 {
    pub fn as_inner(&self) -> &AckMaplePairingRevocationRequest {
        &self.request
    }

    pub fn canonical_transcript(&self) -> Result<Vec<u8>> {
        self.request.canonical_transcript()
    }

    pub fn with_signature(mut self, signature: impl Into<String>) -> Self {
        self.request.signature = signature.into();
        self
    }
}

/// Explicitly supplied trust anchors for OpenSecret pairing artifacts.
///
/// Constructing this set never fetches or derives trust from an artifact. Key
/// rotation is an application/configuration concern and unknown key IDs fail
/// closed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MaplePairingIssuerKeyV1 {
    pub key_id: String,
    pub algorithm: MaplePairingIdentityAlgorithm,
    pub public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MaplePairingIssuerKeySetV1 {
    pub version: u16,
    pub keys: Vec<MaplePairingIssuerKeyV1>,
}

#[derive(Debug, Clone)]
pub struct MaplePairingIssuerKeySet {
    keys: HashMap<String, [u8; ED25519_PUBLIC_KEY_BYTES]>,
}

/// Signing abstraction used by literal cross-implementation vector tests and
/// trusted service integrations. Ordinary SDK clients only need the public
/// [`MaplePairingIssuerKeySet`] verifier.
pub trait MaplePairingIssuer: Send + Sync {
    fn key_id(&self) -> &str;
    fn public_key_bytes(&self) -> [u8; ED25519_PUBLIC_KEY_BYTES];
    fn sign(&self, transcript: &[u8]) -> Result<[u8; ED25519_SIGNATURE_BYTES]>;
}

impl MaplePairingIssuerKeySet {
    pub fn from_v1(keyset: MaplePairingIssuerKeySetV1) -> Result<Self> {
        if keyset.version != 1 || keyset.keys.is_empty() {
            return Err(Error::Configuration(
                "Unsupported or empty Maple pairing issuer key set".to_string(),
            ));
        }
        let mut previous_key_id: Option<&str> = None;
        let mut trusted = HashMap::with_capacity(keyset.keys.len());
        let mut public_keys = HashSet::with_capacity(keyset.keys.len());
        for key in &keyset.keys {
            validate_token(&key.key_id, MAX_ISSUER_KEY_ID_BYTES, "issuer key ID")?;
            if previous_key_id.is_some_and(|previous| previous >= key.key_id.as_str()) {
                return Err(Error::Configuration(
                    "Maple pairing issuer keys must have unique ascending key IDs".to_string(),
                ));
            }
            previous_key_id = Some(&key.key_id);
            let public_key = decode_exact_base64(
                &key.public_key,
                ED25519_PUBLIC_KEY_BYTES,
                "Maple pairing issuer public key",
                ErrorClass::Configuration,
            )?;
            let public_key: [u8; ED25519_PUBLIC_KEY_BYTES] = public_key
                .try_into()
                .expect("exact public-key length was checked");
            VerifyingKey::from_bytes(&public_key).map_err(|_| {
                Error::Configuration(
                    "Maple pairing issuer public key is not a valid Ed25519 point".to_string(),
                )
            })?;
            if !public_keys.insert(public_key) {
                return Err(Error::Configuration(
                    "Maple pairing issuer public keys must be unique".to_string(),
                ));
            }
            trusted.insert(key.key_id.clone(), public_key);
        }
        Ok(Self { keys: trusted })
    }

    pub fn new<I, S>(keys: I) -> Result<Self>
    where
        I: IntoIterator<Item = (S, [u8; ED25519_PUBLIC_KEY_BYTES])>,
        S: Into<String>,
    {
        let mut trusted = HashMap::new();
        let mut public_keys = HashSet::new();
        for (key_id, public_key) in keys {
            let key_id = key_id.into();
            validate_token(&key_id, MAX_ISSUER_KEY_ID_BYTES, "issuer key ID")?;
            VerifyingKey::from_bytes(&public_key).map_err(|_| {
                Error::Configuration(
                    "Maple pairing issuer public key is not a valid Ed25519 point".to_string(),
                )
            })?;
            if !public_keys.insert(public_key) {
                return Err(Error::Configuration(
                    "Maple pairing issuer public keys must be unique".to_string(),
                ));
            }
            if trusted.insert(key_id, public_key).is_some() {
                return Err(Error::Configuration(
                    "Maple pairing issuer key IDs must be unique".to_string(),
                ));
            }
        }
        if trusted.is_empty() {
            return Err(Error::Configuration(
                "Maple pairing trust set must not be empty".to_string(),
            ));
        }
        Ok(Self { keys: trusted })
    }

    fn verify(&self, key_id: &str, transcript: &[u8], signature: &str) -> Result<()> {
        let public_key = self.keys.get(key_id).ok_or_else(|| {
            Error::InvalidResponse(
                "Maple pairing artifact uses an untrusted issuer key".to_string(),
            )
        })?;
        let signature = decode_exact_base64(
            signature,
            ED25519_SIGNATURE_BYTES,
            "Maple pairing issuer signature",
            ErrorClass::InvalidResponse,
        )?;
        let signature: [u8; ED25519_SIGNATURE_BYTES] = signature
            .try_into()
            .expect("exact signature length was checked");
        let public_key = VerifyingKey::from_bytes(public_key).map_err(|_| {
            Error::InvalidResponse(
                "Maple pairing artifact issuer key is not a valid Ed25519 point".to_string(),
            )
        })?;
        public_key
            .verify_strict(transcript, &Signature::from_bytes(&signature))
            .map_err(|_| {
                Error::InvalidResponse(
                    "Maple pairing artifact issuer signature does not verify".to_string(),
                )
            })
    }
}

impl TryFrom<MaplePairingIssuerKeySetV1> for MaplePairingIssuerKeySet {
    type Error = Error;

    fn try_from(value: MaplePairingIssuerKeySetV1) -> Result<Self> {
        Self::from_v1(value)
    }
}

fn validate_protocol_header(protocol_version: u16, transcript_version: u16) -> Result<()> {
    if protocol_version != MAPLE_PAIRING_PROTOCOL_VERSION
        || transcript_version != MAPLE_PAIRING_TRANSCRIPT_VERSION
    {
        return Err(Error::Configuration(
            "Unsupported Maple pairing protocol or transcript version".to_string(),
        ));
    }
    Ok(())
}

fn validate_non_nil(ids: &[(&str, Uuid)]) -> Result<()> {
    if let Some((name, _)) = ids.iter().find(|(_, id)| id.is_nil()) {
        return Err(Error::Configuration(format!(
            "Maple pairing {name} must not be nil"
        )));
    }
    Ok(())
}

fn validate_hex_key(value: &str, field_name: &str, class: ErrorClass) -> Result<[u8; 32]> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(class.error(format!(
            "{field_name} must be canonical lowercase 32-byte hex"
        )));
    }
    let decoded: [u8; 32] = hex::decode(value)
        .map_err(|_| class.error(format!("{field_name} is invalid hex")))?
        .try_into()
        .map_err(|_| class.error(format!("{field_name} must contain 32 bytes")))?;
    VerifyingKey::from_bytes(&decoded).map_err(|_| {
        class.error(format!(
            "{field_name} must encode a valid Ed25519 public-key point"
        ))
    })?;
    Ok(decoded)
}

fn validate_token(value: &str, max_len: usize, field_name: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > max_len
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'.' | b'_' | b':' | b'-')
        })
    {
        return Err(Error::Configuration(format!(
            "Maple pairing {field_name} must be a 1 to {max_len} byte lowercase token"
        )));
    }
    Ok(())
}

fn validate_response_token(value: &str, max_len: usize, field_name: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > max_len
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'.' | b'_' | b':' | b'-')
        })
    {
        return Err(Error::InvalidResponse(format!(
            "Maple pairing {field_name} must be a 1 to {max_len} byte lowercase token"
        )));
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum ErrorClass {
    Configuration,
    InvalidResponse,
}

impl ErrorClass {
    fn error(self, message: String) -> Error {
        match self {
            Self::Configuration => Error::Configuration(message),
            Self::InvalidResponse => Error::InvalidResponse(message),
        }
    }
}

fn decode_exact_base64(
    value: &str,
    expected_len: usize,
    field_name: &str,
    class: ErrorClass,
) -> Result<Vec<u8>> {
    let decoded = BASE64
        .decode(value)
        .map_err(|_| class.error(format!("{field_name} must be standard base64")))?;
    if decoded.len() != expected_len || BASE64.encode(&decoded) != value {
        return Err(class.error(format!(
            "{field_name} must be canonical padded standard base64 encoding {expected_len} bytes"
        )));
    }
    Ok(decoded)
}

fn validate_cursor(cursor: Option<&str>) -> Result<()> {
    if cursor.is_some_and(|cursor| {
        cursor.is_empty() || cursor.len() > MAPLE_PAIRING_CURSOR_MAX_BYTES || !cursor.is_ascii()
    }) {
        return Err(Error::Configuration(format!(
            "Maple pairing cursor must contain 1 to {MAPLE_PAIRING_CURSOR_MAX_BYTES} bytes"
        )));
    }
    Ok(())
}

fn validate_limit(limit: Option<u16>) -> Result<u16> {
    let limit = limit.unwrap_or(MAPLE_PAIRING_LIST_DEFAULT_LIMIT);
    if limit == 0 || limit > MAPLE_PAIRING_LIST_MAX_LIMIT {
        return Err(Error::Configuration(format!(
            "Maple pairing page limit must be between 1 and {MAPLE_PAIRING_LIST_MAX_LIMIT}"
        )));
    }
    Ok(limit)
}

fn verify_device_signature(
    public_key: &[u8; 32],
    transcript: &[u8],
    signature: &str,
) -> Result<()> {
    let signature = decode_exact_base64(
        signature,
        ED25519_SIGNATURE_BYTES,
        "Maple pairing request signature",
        ErrorClass::Configuration,
    )?;
    let signature: [u8; ED25519_SIGNATURE_BYTES] = signature
        .try_into()
        .expect("exact signature length was checked");
    let public_key = VerifyingKey::from_bytes(public_key).map_err(|_| {
        Error::Configuration(
            "Maple pairing request public key is not a valid Ed25519 point".to_string(),
        )
    })?;
    public_key
        .verify_strict(transcript, &Signature::from_bytes(&signature))
        .map_err(|_| {
            Error::Configuration(
                "Maple pairing request signature does not verify for its transcript".to_string(),
            )
        })
}

#[derive(Default)]
struct CanonicalBytes {
    bytes: Vec<u8>,
}

impl CanonicalBytes {
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

    fn append_u64(&mut self, value: u64) -> &mut Self {
        self.append_field(b'L', &value.to_be_bytes())
    }

    fn append_i64(&mut self, value: i64) -> &mut Self {
        self.append_field(b'l', &value.to_be_bytes())
    }

    fn append_bool(&mut self, value: bool) -> &mut Self {
        self.append_field(b'?', &[u8::from(value)])
    }

    fn append_uuid(&mut self, value: Uuid) -> &mut Self {
        self.append_field(b'u', value.as_bytes())
    }

    fn append_field(&mut self, tag: u8, value: &[u8]) -> &mut Self {
        let length = u32::try_from(value.len()).expect("bounded Maple pairing canonical field");
        self.bytes.push(tag);
        self.bytes.extend_from_slice(&length.to_be_bytes());
        self.bytes.extend_from_slice(value);
        self
    }

    fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

fn sha256_base64(bytes: &[u8]) -> String {
    BASE64.encode(Sha256::digest(bytes))
}

fn validate_versioned_request(protocol_version: u16, transcript_version: u16) -> Result<()> {
    validate_protocol_header(protocol_version, transcript_version)
}

fn validate_positive_revision(revision: i64) -> Result<()> {
    if revision <= 0 || revision == i64::MAX {
        return Err(Error::Configuration(
            "Maple pairing expected revision must be positive and incrementable".to_string(),
        ));
    }
    Ok(())
}

fn validate_incarnation(incarnation: u64) -> Result<()> {
    if incarnation == 0 || incarnation > i64::MAX as u64 {
        return Err(Error::Configuration(
            "Maple pairing incarnation must be nonzero and supported by the service".to_string(),
        ));
    }
    Ok(())
}

fn validate_revocation_stream(stream_id: Uuid, generation: u64, class: ErrorClass) -> Result<()> {
    if stream_id.is_nil() || generation == 0 || generation > i64::MAX as u64 {
        return Err(class.error(
            "Maple revocation stream requires a non-nil ID and supported nonzero generation"
                .to_string(),
        ));
    }
    Ok(())
}

fn validate_protocol_range(minimum: u16, maximum: u16) -> Result<()> {
    if minimum == 0 || minimum > maximum {
        return Err(Error::Configuration(
            "Maple pairing protocol range must be nonzero and ordered".to_string(),
        ));
    }
    Ok(())
}

fn validate_timestamp(timestamp: i64, field_name: &str, class: ErrorClass) -> Result<()> {
    if timestamp <= 0 {
        return Err(class.error(format!(
            "Maple pairing {field_name} must be positive Unix milliseconds"
        )));
    }
    Ok(())
}

fn validate_device_claim(
    claim: &MaplePairingDeviceClaimV1,
    class: ErrorClass,
) -> Result<([u8; 32], [u8; 32])> {
    if claim.registration_id.is_nil() || claim.device_id.is_nil() || claim.installation_id.is_nil()
    {
        return Err(
            class.error("Maple pairing artifact device claim contains a nil ID".to_string())
        );
    }
    if claim.endpoint_epoch > i64::MAX as u64 {
        return Err(class.error("Maple pairing artifact endpoint epoch is unsupported".to_string()));
    }
    let identity_key = decode_exact_base64(
        &claim.identity_public_key,
        ED25519_PUBLIC_KEY_BYTES,
        "Maple pairing artifact identity public key",
        class,
    )?;
    let identity_key: [u8; 32] = identity_key
        .try_into()
        .expect("exact key length was checked");
    VerifyingKey::from_bytes(&identity_key).map_err(|_| {
        class.error("Maple pairing artifact identity key is not a valid Ed25519 point".to_string())
    })?;
    let endpoint_key = validate_hex_key(
        &claim.endpoint_id,
        "Maple pairing artifact endpoint ID",
        class,
    )?;
    // Maple pairing v1 intentionally uses the same durable Ed25519 key as the
    // installation's Iroh endpoint identity. A separately hardware-backed
    // companion proof key remains a future hardening option, not a v1 field.
    if identity_key != endpoint_key {
        return Err(class.error(
            "Maple pairing artifact identity public key must match its Iroh endpoint ID"
                .to_string(),
        ));
    }
    Ok((identity_key, endpoint_key))
}

fn validate_directed_device_claims(
    controller: &MaplePairingDeviceClaimV1,
    host: &MaplePairingDeviceClaimV1,
    execution_target_id: Uuid,
    class: ErrorClass,
) -> Result<()> {
    if controller.registration_id == host.registration_id
        || controller.device_id == host.device_id
        || controller.installation_id == host.installation_id
        || controller.identity_public_key == host.identity_public_key
        || controller.endpoint_id == host.endpoint_id
        || execution_target_id != host.registration_id
    {
        return Err(class.error(
            "Maple pairing must bind two distinct installations in one controller-to-host direction"
                .to_string(),
        ));
    }
    Ok(())
}

fn device_claim_is_same_identity_at_or_before(
    historical: &MaplePairingDeviceClaimV1,
    current: &MaplePairingDeviceClaimV1,
) -> bool {
    historical.registration_id == current.registration_id
        && historical.device_id == current.device_id
        && historical.installation_id == current.installation_id
        && historical.identity_algorithm == current.identity_algorithm
        && historical.identity_public_key == current.identity_public_key
        && historical.endpoint_id == current.endpoint_id
        && historical.endpoint_epoch <= current.endpoint_epoch
}

fn append_device_claim(
    canonical: &mut CanonicalBytes,
    claim: &MaplePairingDeviceClaimV1,
    class: ErrorClass,
) -> Result<()> {
    let (identity_key, endpoint_key) = validate_device_claim(claim, class)?;
    canonical
        .append_uuid(claim.registration_id)
        .append_uuid(claim.device_id)
        .append_uuid(claim.installation_id)
        .append_str(claim.identity_algorithm.as_str())
        .append_bytes(&identity_key)
        .append_bytes(&endpoint_key)
        .append_u64(claim.endpoint_epoch);
    Ok(())
}

fn validate_reset_counter(value: u64, field_name: &str, class: ErrorClass) -> Result<()> {
    if value == 0 || value > i64::MAX as u64 {
        return Err(class.error(format!(
            "Maple reset-clear {field_name} must be nonzero and supported by the service"
        )));
    }
    Ok(())
}

fn decode_digest_for_class(value: &str, field_name: &str, class: ErrorClass) -> Result<Vec<u8>> {
    decode_exact_base64(value, DIGEST_BYTES, field_name, class)
}

/// Canonicalizes the complete private admission set used to derive the public
/// reset-clear count and digest. Leaves are sorted by their canonical tuple;
/// duplicate tuples fail closed.
pub fn reset_clear_admission_set_transcript(
    artifact_version: u16,
    admissions: &[MapleResetClearAdmissionLeafV1],
) -> Result<Vec<u8>> {
    if artifact_version != MAPLE_PAIRING_ARTIFACT_VERSION {
        return Err(Error::Configuration(
            "Unsupported Maple reset-clear artifact version".to_string(),
        ));
    }
    let admission_count = u16::try_from(admissions.len()).map_err(|_| {
        Error::Configuration("Maple reset-clear admission count is unsupported".to_string())
    })?;
    if admission_count > MAPLE_RESET_CLEAR_MAX_ADMISSIONS {
        return Err(Error::Configuration(format!(
            "Maple reset-clear admission set supports at most {MAPLE_RESET_CLEAR_MAX_ADMISSIONS} leaves"
        )));
    }
    let mut canonical_leaves = Vec::with_capacity(admissions.len());
    for admission in admissions {
        if admission.pair_id.is_nil() {
            return Err(Error::Configuration(
                "Maple reset-clear admission leaves require non-nil pair IDs".to_string(),
            ));
        }
        validate_incarnation(admission.pairing_incarnation)?;
        canonical_leaves.push(admission.clone());
    }
    canonical_leaves.sort_unstable();
    if canonical_leaves.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(Error::Configuration(
            "Maple reset-clear admission leaves must be unique".to_string(),
        ));
    }

    let mut canonical = CanonicalBytes::new(RESET_CLEAR_ADMISSION_SET_DOMAIN);
    canonical
        .append_u16(artifact_version)
        .append_u16(admission_count);
    for admission in canonical_leaves {
        canonical
            .append_uuid(admission.pair_id)
            .append_u64(admission.pairing_incarnation)
            .append_bytes(&admission.pair_authorization_digest);
    }
    Ok(canonical.into_bytes())
}

pub fn reset_clear_admission_set_digest(
    artifact_version: u16,
    admissions: &[MapleResetClearAdmissionLeafV1],
) -> Result<[u8; 32]> {
    Ok(Sha256::digest(reset_clear_admission_set_transcript(
        artifact_version,
        admissions,
    )?)
    .into())
}

fn validate_reset_clear_material_fields(
    reset: &MapleResetClearRequiredV1,
    class: ErrorClass,
) -> Result<()> {
    if reset.artifact_version != MAPLE_PAIRING_ARTIFACT_VERSION {
        return Err(class.error("Unsupported Maple reset-clear artifact version".to_string()));
    }
    for (name, id) in [
        ("event ID", reset.event_id),
        ("reset ID", reset.reset_id),
        ("subject account ID", reset.subject_account_id),
        ("subject project ID", reset.subject_project_id),
        (
            "recipient host registration ID",
            reset.recipient_host_registration_id,
        ),
        (
            "source revocation stream ID",
            reset.source_revocation_stream_id,
        ),
        ("revocation stream ID", reset.revocation_stream_id),
    ] {
        if id.is_nil() {
            return Err(class.error(format!("Maple reset-clear {name} must not be nil")));
        }
    }
    validate_reset_counter(reset.reset_generation, "generation", class)?;
    validate_reset_counter(reset.cumulative_reset_count, "cumulative count", class)?;
    validate_reset_counter(reset.source_security_epoch, "source security epoch", class)?;
    validate_reset_counter(reset.security_epoch, "security epoch", class)?;
    if reset.reset_generation != reset.cumulative_reset_count
        || reset.source_security_epoch.checked_add(1) != Some(reset.security_epoch)
    {
        return Err(class.error(
            "Maple reset-clear generation/count or security-epoch progression is invalid"
                .to_string(),
        ));
    }
    validate_device_claim(&reset.host, class)?;
    if reset.recipient_host_registration_id != reset.host.registration_id {
        return Err(
            class.error("Maple reset-clear recipient does not match its host claim".to_string())
        );
    }
    validate_revocation_stream(
        reset.source_revocation_stream_id,
        reset.source_revocation_stream_generation,
        class,
    )?;
    validate_revocation_stream(
        reset.revocation_stream_id,
        reset.revocation_stream_generation,
        class,
    )?;
    if reset.revocation_stream_id == reset.source_revocation_stream_id
        || reset.source_revocation_stream_generation.checked_add(1)
            != Some(reset.revocation_stream_generation)
    {
        return Err(class.error(
            "Maple reset-clear target revocation namespace must be fresh and advance exactly once"
                .to_string(),
        ));
    }
    if reset.issuer_sequence != 1 {
        return Err(class.error(
            "Maple reset-clear instruction must be event sequence 1 in its target namespace"
                .to_string(),
        ));
    }
    if reset.admission_count > MAPLE_RESET_CLEAR_MAX_ADMISSIONS {
        return Err(class.error(format!(
            "Maple reset-clear admission count exceeds {MAPLE_RESET_CLEAR_MAX_ADMISSIONS}"
        )));
    }
    decode_digest_for_class(
        &reset.admission_set_digest,
        "Maple reset-clear admission-set digest",
        class,
    )?;
    if reset.reset_at_unix_ms < 0 {
        return Err(class.error(
            "Maple reset-clear reset time must be nonnegative Unix milliseconds".to_string(),
        ));
    }

    let predecessor_fields = [
        reset.previous_reset_clear_event_id.is_some(),
        reset.previous_instruction_material_digest.is_some(),
        reset.previous_chain_digest.is_some(),
    ];
    let predecessor_present = predecessor_fields.iter().all(|present| *present);
    if predecessor_fields.iter().any(|present| *present) && !predecessor_present {
        return Err(class.error(
            "Maple reset-clear predecessor fields must be all present or all absent".to_string(),
        ));
    }
    if reset.reset_generation == 1 {
        if predecessor_present {
            return Err(
                class.error("Maple reset-clear genesis must not contain a predecessor".to_string())
            );
        }
    } else if !predecessor_present {
        return Err(class.error(
            "Maple reset-clear successor must contain its complete predecessor commitment"
                .to_string(),
        ));
    }
    if let Some(event_id) = reset.previous_reset_clear_event_id {
        if event_id.is_nil() || event_id == reset.event_id {
            return Err(
                class.error("Maple reset-clear predecessor event ID is invalid".to_string())
            );
        }
    }
    if let Some(digest) = &reset.previous_instruction_material_digest {
        decode_digest_for_class(
            digest,
            "Maple reset-clear previous instruction-material digest",
            class,
        )?;
    }
    if let Some(digest) = &reset.previous_chain_digest {
        decode_digest_for_class(digest, "Maple reset-clear previous chain digest", class)?;
    }
    Ok(())
}

fn append_reset_clear_instruction_material_fields(
    canonical: &mut CanonicalBytes,
    reset: &MapleResetClearRequiredV1,
    class: ErrorClass,
) -> Result<()> {
    validate_reset_clear_material_fields(reset, class)?;
    let admission_set_digest = decode_digest_for_class(
        &reset.admission_set_digest,
        "Maple reset-clear admission-set digest",
        class,
    )?;
    canonical
        .append_u16(reset.artifact_version)
        .append_uuid(reset.event_id)
        .append_uuid(reset.reset_id)
        .append_u64(reset.reset_generation)
        .append_u64(reset.cumulative_reset_count)
        .append_u64(reset.source_security_epoch)
        .append_u64(reset.security_epoch)
        .append_uuid(reset.subject_account_id)
        .append_uuid(reset.subject_project_id)
        .append_uuid(reset.recipient_host_registration_id);
    append_device_claim(canonical, &reset.host, class)?;
    canonical
        .append_u64(reset.issuer_sequence)
        .append_uuid(reset.source_revocation_stream_id)
        .append_u64(reset.source_revocation_stream_generation)
        .append_uuid(reset.revocation_stream_id)
        .append_u64(reset.revocation_stream_generation)
        .append_str(reset.clear_scope.as_str())
        .append_u16(reset.admission_count)
        .append_bytes(&admission_set_digest)
        .append_bool(reset.previous_reset_clear_event_id.is_some());
    if let (Some(previous_event_id), Some(previous_material_digest), Some(previous_chain_digest)) = (
        reset.previous_reset_clear_event_id,
        reset.previous_instruction_material_digest.as_ref(),
        reset.previous_chain_digest.as_ref(),
    ) {
        canonical
            .append_uuid(previous_event_id)
            .append_bytes(&decode_digest_for_class(
                previous_material_digest,
                "Maple reset-clear previous instruction-material digest",
                class,
            )?)
            .append_bytes(&decode_digest_for_class(
                previous_chain_digest,
                "Maple reset-clear previous chain digest",
                class,
            )?);
    }
    canonical.append_i64(reset.reset_at_unix_ms);
    Ok(())
}

pub fn reset_clear_instruction_material_transcript(
    reset: &MapleResetClearRequiredV1,
) -> Result<Vec<u8>> {
    let mut canonical = CanonicalBytes::new(RESET_CLEAR_INSTRUCTION_MATERIAL_DOMAIN);
    append_reset_clear_instruction_material_fields(
        &mut canonical,
        reset,
        ErrorClass::InvalidResponse,
    )?;
    Ok(canonical.into_bytes())
}

pub fn reset_clear_chain_transcript(reset: &MapleResetClearRequiredV1) -> Result<Vec<u8>> {
    validate_reset_clear_material_fields(reset, ErrorClass::InvalidResponse)?;
    let instruction_material_digest = decode_digest_for_class(
        &reset.instruction_material_digest,
        "Maple reset-clear instruction-material digest",
        ErrorClass::InvalidResponse,
    )?;
    let mut canonical = CanonicalBytes::new(RESET_CLEAR_CHAIN_DOMAIN);
    canonical
        .append_u16(reset.artifact_version)
        .append_bool(reset.previous_reset_clear_event_id.is_some());
    if let (
        Some(previous_chain_digest),
        Some(previous_event_id),
        Some(previous_instruction_material_digest),
    ) = (
        reset.previous_chain_digest.as_ref(),
        reset.previous_reset_clear_event_id,
        reset.previous_instruction_material_digest.as_ref(),
    ) {
        canonical
            .append_bytes(&decode_digest_for_class(
                previous_chain_digest,
                "Maple reset-clear previous chain digest",
                ErrorClass::InvalidResponse,
            )?)
            .append_uuid(previous_event_id)
            .append_bytes(&decode_digest_for_class(
                previous_instruction_material_digest,
                "Maple reset-clear previous instruction-material digest",
                ErrorClass::InvalidResponse,
            )?);
    }
    canonical
        .append_uuid(reset.reset_id)
        .append_uuid(reset.event_id)
        .append_u64(reset.reset_generation)
        .append_bytes(&instruction_material_digest)
        .append_u64(reset.cumulative_reset_count);
    Ok(canonical.into_bytes())
}

pub fn reset_clear_required_transcript(reset: &MapleResetClearRequiredV1) -> Result<Vec<u8>> {
    validate_reset_clear_unsigned(reset)?;
    let instruction_material_digest = decode_digest_for_class(
        &reset.instruction_material_digest,
        "Maple reset-clear instruction-material digest",
        ErrorClass::InvalidResponse,
    )?;
    let chain_digest = decode_digest_for_class(
        &reset.chain_digest,
        "Maple reset-clear chain digest",
        ErrorClass::InvalidResponse,
    )?;
    let mut canonical = CanonicalBytes::new(RESET_CLEAR_REQUIRED_DOMAIN);
    append_reset_clear_instruction_material_fields(
        &mut canonical,
        reset,
        ErrorClass::InvalidResponse,
    )?;
    canonical
        .append_bytes(&instruction_material_digest)
        .append_u64(reset.cumulative_reset_count)
        .append_bytes(&chain_digest)
        .append_str(&reset.issuer_key_id);
    Ok(canonical.into_bytes())
}

fn validate_reset_clear_unsigned(reset: &MapleResetClearRequiredV1) -> Result<()> {
    validate_reset_clear_material_fields(reset, ErrorClass::InvalidResponse)?;
    validate_response_token(
        &reset.issuer_key_id,
        MAX_ISSUER_KEY_ID_BYTES,
        "reset-clear issuer key ID",
    )?;
    decode_digest_for_class(
        &reset.instruction_material_digest,
        "Maple reset-clear instruction-material digest",
        ErrorClass::InvalidResponse,
    )?;
    decode_digest_for_class(
        &reset.chain_digest,
        "Maple reset-clear chain digest",
        ErrorClass::InvalidResponse,
    )?;
    let material_digest = sha256_base64(&reset_clear_instruction_material_transcript(reset)?);
    if reset.instruction_material_digest != material_digest {
        return Err(Error::InvalidResponse(
            "Maple reset-clear instruction-material digest does not match its fields".to_string(),
        ));
    }
    let chain_digest = sha256_base64(&reset_clear_chain_transcript(reset)?);
    if reset.chain_digest != chain_digest {
        return Err(Error::InvalidResponse(
            "Maple reset-clear chain digest does not match its fields".to_string(),
        ));
    }
    Ok(())
}

fn validate_request_signature(
    public_key: &[u8; 32],
    transcript: &[u8],
    signature: &str,
) -> Result<()> {
    verify_device_signature(public_key, transcript, signature)
}

impl CreateMaplePairingRequest {
    pub fn canonical_transcript(&self) -> Result<Vec<u8>> {
        validate_versioned_request(self.protocol_version, self.transcript_version)?;
        validate_non_nil(&[
            ("operation ID", self.operation_id),
            ("asserted account ID", self.asserted_account_id),
            ("asserted project ID", self.asserted_project_id),
            (
                "controller registration ID",
                self.controller_registration_id,
            ),
            ("controller device ID", self.controller_device_id),
            (
                "controller installation ID",
                self.controller_installation_id,
            ),
            ("host registration ID", self.host_registration_id),
            ("host device ID", self.host_device_id),
            ("host installation ID", self.host_installation_id),
            ("execution target ID", self.execution_target_id),
        ])?;
        if self.execution_target_id != self.host_registration_id {
            return Err(Error::Configuration(
                "Maple pairing execution target must equal the host registration ID".to_string(),
            ));
        }
        if self.controller_registration_id == self.host_registration_id
            || self.controller_device_id == self.host_device_id
            || self.controller_installation_id == self.host_installation_id
            || self.controller_endpoint_id == self.host_endpoint_id
        {
            return Err(Error::Configuration(
                "Maple pairing controller and host must be distinct installations".to_string(),
            ));
        }
        if self.controller_endpoint_epoch > i64::MAX as u64
            || self.host_endpoint_epoch > i64::MAX as u64
        {
            return Err(Error::Configuration(
                "Maple pairing endpoint epoch is unsupported".to_string(),
            ));
        }
        validate_protocol_range(self.protocol_min, self.protocol_max)?;
        let controller_endpoint = validate_hex_key(
            &self.controller_endpoint_id,
            "Maple pairing controller endpoint ID",
            ErrorClass::Configuration,
        )?;
        let host_endpoint = validate_hex_key(
            &self.host_endpoint_id,
            "Maple pairing host endpoint ID",
            ErrorClass::Configuration,
        )?;
        let nonce = decode_exact_base64(
            &self.pairing_request_nonce,
            NONCE_BYTES,
            "Maple pairing request nonce",
            ErrorClass::Configuration,
        )?;
        let mut canonical = CanonicalBytes::new("os.maple-pair-request.v1");
        canonical
            .append_u16(self.protocol_version)
            .append_u16(self.transcript_version)
            .append_uuid(self.asserted_account_id)
            .append_uuid(self.asserted_project_id)
            .append_uuid(self.operation_id)
            .append_uuid(self.controller_registration_id)
            .append_uuid(self.controller_device_id)
            .append_uuid(self.controller_installation_id)
            .append_bytes(&controller_endpoint)
            .append_u64(self.controller_endpoint_epoch)
            .append_uuid(self.host_registration_id)
            .append_uuid(self.host_device_id)
            .append_uuid(self.host_installation_id)
            .append_bytes(&host_endpoint)
            .append_u64(self.host_endpoint_epoch)
            .append_str(self.direction.as_str())
            .append_uuid(self.execution_target_id)
            .append_bytes(&nonce)
            .append_u16(self.protocol_min)
            .append_u16(self.protocol_max);
        Ok(canonical.into_bytes())
    }

    pub fn validate(&self) -> Result<()> {
        let transcript = self.canonical_transcript()?;
        let controller_key = validate_hex_key(
            &self.controller_endpoint_id,
            "Maple pairing controller endpoint ID",
            ErrorClass::Configuration,
        )?;
        validate_request_signature(&controller_key, &transcript, &self.signature)
    }

    pub fn transcript_digest(&self) -> Result<String> {
        Ok(sha256_base64(&self.canonical_transcript()?))
    }
}

impl ListMaplePairingsRequest {
    pub fn canonical_transcript(&self) -> Result<Vec<u8>> {
        validate_versioned_request(self.protocol_version, self.transcript_version)?;
        validate_non_nil(&[
            ("query ID", self.query_id),
            ("asserted account ID", self.asserted_account_id),
            ("asserted project ID", self.asserted_project_id),
            ("actor registration ID", self.actor_registration_id),
        ])?;
        let states = Self::canonical_states(self.states.clone())?;
        if states != self.states {
            return Err(Error::Configuration(
                "Maple pairing states must use canonical enum order".to_string(),
            ));
        }
        validate_cursor(self.cursor.as_deref())?;
        let effective_limit = validate_limit(self.limit)?;
        let mut canonical = CanonicalBytes::new("os.maple-pair-list.v1");
        canonical
            .append_u16(self.protocol_version)
            .append_u16(self.transcript_version)
            .append_uuid(self.asserted_account_id)
            .append_uuid(self.asserted_project_id)
            .append_uuid(self.query_id)
            .append_uuid(self.actor_registration_id)
            .append_str(self.role.as_str())
            .append_u16(states.len() as u16);
        for state in states {
            canonical.append_str(state.as_str());
        }
        canonical.append_bool(self.cursor.is_some());
        if let Some(cursor) = &self.cursor {
            canonical.append_str(cursor);
        }
        canonical.append_u16(effective_limit);
        Ok(canonical.into_bytes())
    }

    pub fn validate_with_signing_key(&self, public_key: &[u8; 32]) -> Result<()> {
        validate_request_signature(public_key, &self.canonical_transcript()?, &self.signature)
    }
}

impl MaplePairingStatusRequest {
    pub fn canonical_transcript(&self) -> Result<Vec<u8>> {
        validate_versioned_request(self.protocol_version, self.transcript_version)?;
        validate_non_nil(&[
            ("query ID", self.query_id),
            ("asserted account ID", self.asserted_account_id),
            ("asserted project ID", self.asserted_project_id),
            ("actor registration ID", self.actor_registration_id),
            ("pair ID", self.pair_id),
        ])?;
        let mut canonical = CanonicalBytes::new("os.maple-pair-status.v1");
        canonical
            .append_u16(self.protocol_version)
            .append_u16(self.transcript_version)
            .append_uuid(self.asserted_account_id)
            .append_uuid(self.asserted_project_id)
            .append_uuid(self.query_id)
            .append_uuid(self.actor_registration_id)
            .append_uuid(self.pair_id);
        Ok(canonical.into_bytes())
    }

    pub fn validate_with_signing_key(&self, public_key: &[u8; 32]) -> Result<()> {
        validate_request_signature(public_key, &self.canonical_transcript()?, &self.signature)
    }
}

impl ApproveMaplePairingRequest {
    /// Builds an unsigned v1 approval from a currently valid issuer-verified
    /// request ticket.
    ///
    /// Pairing v1 does not permit protocol narrowing: the approved range is
    /// copied exactly from the controller's ticket. The host-signed approval
    /// also binds the namespace obtained from signed discovery and reconciled
    /// to durable local admission state. The caller owns the host private key
    /// and must attach its signature separately.
    pub fn unsigned_v1_from_verified_ticket(
        operation_id: Uuid,
        host_approval_nonce: impl Into<String>,
        ticket: &VerifiedMaplePairRequestTicketV1,
        revocation_stream: &DurablyReconciledMapleRevocationStreamV1,
    ) -> Result<PreparedMaplePairingApprovalV1> {
        let ticket = ticket.as_inner();
        let checkpoint = revocation_stream.as_inner();
        if checkpoint.subject_account_id != ticket.subject_account_id
            || checkpoint.subject_project_id != ticket.subject_project_id
            || !device_claim_is_same_identity_at_or_before(&ticket.host, &checkpoint.host)
        {
            return Err(Error::Configuration(
                "Maple approval revocation stream does not match the verified host ticket"
                    .to_string(),
            ));
        }
        let request = Self {
            protocol_version: MAPLE_PAIRING_PROTOCOL_VERSION,
            transcript_version: MAPLE_PAIRING_TRANSCRIPT_VERSION,
            operation_id,
            asserted_account_id: ticket.subject_account_id,
            asserted_project_id: ticket.subject_project_id,
            host_registration_id: ticket.host.registration_id,
            pairing_request_id: ticket.pairing_request_id,
            pair_id: ticket.pair_id,
            expected_pairing_revision: 1,
            pairing_incarnation: ticket.pairing_incarnation,
            revocation_stream_id: checkpoint.revocation_stream_id,
            revocation_stream_generation: checkpoint.revocation_stream_generation,
            request_ticket_digest: ticket.transcript_digest()?,
            host_approval_nonce: host_approval_nonce.into(),
            approved_protocol_min: ticket.protocol_min,
            approved_protocol_max: ticket.protocol_max,
            signature: String::new(),
        };
        request.canonical_transcript()?;
        Ok(PreparedMaplePairingApprovalV1 { request })
    }

    pub fn canonical_transcript(&self) -> Result<Vec<u8>> {
        validate_versioned_request(self.protocol_version, self.transcript_version)?;
        validate_non_nil(&[
            ("operation ID", self.operation_id),
            ("asserted account ID", self.asserted_account_id),
            ("asserted project ID", self.asserted_project_id),
            ("host registration ID", self.host_registration_id),
            ("pairing request ID", self.pairing_request_id),
            ("pair ID", self.pair_id),
        ])?;
        validate_positive_revision(self.expected_pairing_revision)?;
        validate_incarnation(self.pairing_incarnation)?;
        validate_revocation_stream(
            self.revocation_stream_id,
            self.revocation_stream_generation,
            ErrorClass::Configuration,
        )?;
        validate_protocol_range(self.approved_protocol_min, self.approved_protocol_max)?;
        let ticket_digest = decode_exact_base64(
            &self.request_ticket_digest,
            DIGEST_BYTES,
            "Maple pairing request ticket digest",
            ErrorClass::Configuration,
        )?;
        let approval_nonce = decode_exact_base64(
            &self.host_approval_nonce,
            NONCE_BYTES,
            "Maple pairing host approval nonce",
            ErrorClass::Configuration,
        )?;
        let mut canonical = CanonicalBytes::new("os.maple-pair-approval.v1");
        canonical
            .append_u16(self.protocol_version)
            .append_u16(self.transcript_version)
            .append_uuid(self.asserted_account_id)
            .append_uuid(self.asserted_project_id)
            .append_uuid(self.operation_id)
            .append_uuid(self.host_registration_id)
            .append_uuid(self.pairing_request_id)
            .append_uuid(self.pair_id)
            .append_i64(self.expected_pairing_revision)
            .append_u64(self.pairing_incarnation)
            .append_uuid(self.revocation_stream_id)
            .append_u64(self.revocation_stream_generation)
            .append_bytes(&ticket_digest)
            .append_bytes(&approval_nonce)
            .append_u16(self.approved_protocol_min)
            .append_u16(self.approved_protocol_max);
        Ok(canonical.into_bytes())
    }

    pub fn validate_with_signing_key(&self, public_key: &[u8; 32]) -> Result<()> {
        validate_request_signature(public_key, &self.canonical_transcript()?, &self.signature)
    }

    pub fn transcript_digest(&self) -> Result<String> {
        Ok(sha256_base64(&self.canonical_transcript()?))
    }
}

impl ConfirmMaplePairingRequest {
    /// Builds the unsigned host-commit request after the caller has durably
    /// promoted its local approval record.
    ///
    /// The verified authorization supplies the exact pair lineage and digest;
    /// this constructor does not persist anything and deliberately does not
    /// send or auto-confirm. The caller must sign the returned prepared value's
    /// canonical transcript with the host installation key, attach it through
    /// `PreparedMaplePairingHostCommitV1::with_signature`, and invoke
    /// `OpenSecretClient::confirm_maple_pairing` separately.
    pub fn unsigned_v1_after_durable_commit(
        operation_id: Uuid,
        ready: &HostCommitReadyMaplePairAuthorizationV1,
    ) -> Result<PreparedMaplePairingHostCommitV1> {
        let authorization = ready.authorization().as_inner();
        let request = Self {
            protocol_version: MAPLE_PAIRING_PROTOCOL_VERSION,
            transcript_version: MAPLE_PAIRING_TRANSCRIPT_VERSION,
            operation_id,
            asserted_account_id: authorization.subject_account_id,
            asserted_project_id: authorization.subject_project_id,
            host_registration_id: authorization.host.registration_id,
            pairing_request_id: authorization.pairing_request_id,
            pair_id: authorization.pair_id,
            expected_pairing_revision: ready.expected_pairing_revision(),
            pairing_incarnation: authorization.pairing_incarnation,
            pair_authorization_digest: authorization.transcript_digest()?,
            signature: String::new(),
        };
        request.canonical_transcript()?;
        Ok(PreparedMaplePairingHostCommitV1 { request })
    }

    pub fn canonical_transcript(&self) -> Result<Vec<u8>> {
        validate_versioned_request(self.protocol_version, self.transcript_version)?;
        validate_non_nil(&[
            ("operation ID", self.operation_id),
            ("asserted account ID", self.asserted_account_id),
            ("asserted project ID", self.asserted_project_id),
            ("host registration ID", self.host_registration_id),
            ("pairing request ID", self.pairing_request_id),
            ("pair ID", self.pair_id),
        ])?;
        validate_positive_revision(self.expected_pairing_revision)?;
        validate_incarnation(self.pairing_incarnation)?;
        let authorization_digest = decode_exact_base64(
            &self.pair_authorization_digest,
            DIGEST_BYTES,
            "Maple pair authorization digest",
            ErrorClass::Configuration,
        )?;
        let mut canonical = CanonicalBytes::new("os.maple-pair-host-commit.v1");
        canonical
            .append_u16(self.protocol_version)
            .append_u16(self.transcript_version)
            .append_uuid(self.asserted_account_id)
            .append_uuid(self.asserted_project_id)
            .append_uuid(self.operation_id)
            .append_uuid(self.host_registration_id)
            .append_uuid(self.pairing_request_id)
            .append_uuid(self.pair_id)
            .append_i64(self.expected_pairing_revision)
            .append_u64(self.pairing_incarnation)
            .append_bytes(&authorization_digest);
        Ok(canonical.into_bytes())
    }

    pub fn validate_with_signing_key(&self, public_key: &[u8; 32]) -> Result<()> {
        validate_request_signature(public_key, &self.canonical_transcript()?, &self.signature)
    }
}

impl RevokeMaplePairingRequest {
    pub fn canonical_transcript(&self) -> Result<Vec<u8>> {
        validate_versioned_request(self.protocol_version, self.transcript_version)?;
        validate_non_nil(&[
            ("operation ID", self.operation_id),
            ("asserted account ID", self.asserted_account_id),
            ("asserted project ID", self.asserted_project_id),
            ("actor registration ID", self.actor_registration_id),
            ("pairing request ID", self.pairing_request_id),
            ("pair ID", self.pair_id),
        ])?;
        validate_positive_revision(self.expected_pairing_revision)?;
        validate_incarnation(self.pairing_incarnation)?;
        validate_revocation_stream(
            self.revocation_stream_id,
            self.revocation_stream_generation,
            ErrorClass::Configuration,
        )?;
        validate_token(
            &self.reason_code,
            MAX_REASON_CODE_BYTES,
            "revocation reason code",
        )?;
        let mut canonical = CanonicalBytes::new("os.maple-pair-revocation-request.v1");
        canonical
            .append_u16(self.protocol_version)
            .append_u16(self.transcript_version)
            .append_uuid(self.asserted_account_id)
            .append_uuid(self.asserted_project_id)
            .append_uuid(self.operation_id)
            .append_uuid(self.actor_registration_id)
            .append_str(self.actor_role.as_str())
            .append_uuid(self.pairing_request_id)
            .append_uuid(self.pair_id)
            .append_i64(self.expected_pairing_revision)
            .append_u64(self.pairing_incarnation)
            .append_uuid(self.revocation_stream_id)
            .append_u64(self.revocation_stream_generation)
            .append_str(&self.reason_code);
        Ok(canonical.into_bytes())
    }

    pub fn validate_with_signing_key(&self, public_key: &[u8; 32]) -> Result<()> {
        validate_request_signature(public_key, &self.canonical_transcript()?, &self.signature)
    }
}

impl ListMaplePairingRevocationsRequest {
    /// Builds an unsigned, signed-discovery request for the host's current
    /// revocation namespace. Nil/zero values are valid only for this exact
    /// `after=0` discovery sentinel.
    pub fn unsigned_v1_discovery(
        query_id: Uuid,
        asserted_account_id: Uuid,
        asserted_project_id: Uuid,
        host_registration_id: Uuid,
        limit: Option<u16>,
    ) -> Result<Self> {
        let request = Self {
            protocol_version: MAPLE_PAIRING_PROTOCOL_VERSION,
            transcript_version: MAPLE_PAIRING_TRANSCRIPT_VERSION,
            query_id,
            asserted_account_id,
            asserted_project_id,
            host_registration_id,
            revocation_stream_id: Uuid::nil(),
            revocation_stream_generation: 0,
            after_issuer_sequence: 0,
            limit,
            signature: String::new(),
        };
        request.canonical_transcript()?;
        Ok(request)
    }

    /// Builds an unsigned established-stream page request from the exact
    /// namespace the host has already reconciled to durable admission state.
    pub fn unsigned_v1_from_reconciled_stream(
        query_id: Uuid,
        after_issuer_sequence: u64,
        limit: Option<u16>,
        revocation_stream: &DurablyReconciledMapleRevocationStreamV1,
    ) -> Result<Self> {
        let checkpoint = revocation_stream.as_inner();
        let request = Self {
            protocol_version: MAPLE_PAIRING_PROTOCOL_VERSION,
            transcript_version: MAPLE_PAIRING_TRANSCRIPT_VERSION,
            query_id,
            asserted_account_id: checkpoint.subject_account_id,
            asserted_project_id: checkpoint.subject_project_id,
            host_registration_id: checkpoint.host.registration_id,
            revocation_stream_id: checkpoint.revocation_stream_id,
            revocation_stream_generation: checkpoint.revocation_stream_generation,
            after_issuer_sequence,
            limit,
            signature: String::new(),
        };
        request.canonical_transcript()?;
        Ok(request)
    }

    pub fn with_signature(mut self, signature: impl Into<String>) -> Self {
        self.signature = signature.into();
        self
    }

    pub fn canonical_transcript(&self) -> Result<Vec<u8>> {
        validate_versioned_request(self.protocol_version, self.transcript_version)?;
        validate_non_nil(&[
            ("query ID", self.query_id),
            ("asserted account ID", self.asserted_account_id),
            ("asserted project ID", self.asserted_project_id),
            ("host registration ID", self.host_registration_id),
        ])?;
        let discovery = self.revocation_stream_id.is_nil()
            && self.revocation_stream_generation == 0
            && self.after_issuer_sequence == 0;
        if !discovery {
            validate_revocation_stream(
                self.revocation_stream_id,
                self.revocation_stream_generation,
                ErrorClass::Configuration,
            )?;
        }
        if (self.revocation_stream_id.is_nil() || self.revocation_stream_generation == 0)
            && !discovery
        {
            return Err(Error::Configuration(
                "Maple revocation discovery requires the exact nil/zero/after-zero sentinel"
                    .to_string(),
            ));
        }
        if self.after_issuer_sequence > i64::MAX as u64 {
            return Err(Error::Configuration(
                "Maple revocation pagination cursor exceeds the service sequence range".to_string(),
            ));
        }
        let effective_limit = validate_limit(self.limit)?;
        let mut canonical = CanonicalBytes::new("os.maple-pair-revocation-list.v1");
        canonical
            .append_u16(self.protocol_version)
            .append_u16(self.transcript_version)
            .append_uuid(self.asserted_account_id)
            .append_uuid(self.asserted_project_id)
            .append_uuid(self.query_id)
            .append_uuid(self.host_registration_id)
            .append_uuid(self.revocation_stream_id)
            .append_u64(self.revocation_stream_generation)
            .append_u64(self.after_issuer_sequence)
            .append_u16(effective_limit);
        Ok(canonical.into_bytes())
    }

    pub fn validate_with_signing_key(&self, public_key: &[u8; 32]) -> Result<()> {
        validate_request_signature(public_key, &self.canonical_transcript()?, &self.signature)
    }
}

impl AckMaplePairingRevocationRequest {
    /// Builds an unsigned acknowledgement after the caller durably commits the
    /// verified revocation to the host allowlist.
    ///
    /// This constructor performs no storage and no network request. The caller
    /// must attach a host signature and submit it explicitly.
    pub fn unsigned_v1_after_durable_commit(
        operation_id: Uuid,
        expected_previous_issuer_sequence: u64,
        revocation: &VerifiedMaplePairRevocationV1,
    ) -> Result<PreparedMaplePairingRevocationAckV1> {
        let revocation = revocation.as_inner();
        let request = Self {
            protocol_version: MAPLE_PAIRING_PROTOCOL_VERSION,
            transcript_version: MAPLE_PAIRING_TRANSCRIPT_VERSION,
            operation_id,
            asserted_account_id: revocation.subject_account_id,
            asserted_project_id: revocation.subject_project_id,
            host_registration_id: revocation.recipient_host_registration_id,
            revocation_stream_id: revocation.revocation_stream_id,
            revocation_stream_generation: revocation.revocation_stream_generation,
            event_id: revocation.event_id,
            issuer_sequence: revocation.issuer_sequence,
            event_digest: revocation.transcript_digest()?,
            expected_previous_issuer_sequence,
            signature: String::new(),
        };
        request.canonical_transcript()?;
        Ok(PreparedMaplePairingRevocationAckV1 { request })
    }

    /// Builds a scope-clear ACK only after the caller explicitly promoted the
    /// verified latest reset head following an atomic durable full-scope local
    /// admission clear.
    ///
    /// The returned prepared request may be cloned only for byte-identical
    /// retries with the same operation ID. The service authoritatively rejects
    /// a stale head through its checkpoint compare-and-swap and applies
    /// operation-id idempotency; the SDK does not claim process-global
    /// uniqueness for a signed artifact that can be fetched and verified
    /// again.
    pub fn unsigned_v1_after_durable_reset_clear(
        operation_id: Uuid,
        reset_clear: DurablyClearedMapleResetClearRequiredV1,
    ) -> Result<PreparedMaplePairingRevocationAckV1> {
        let reset = reset_clear.as_inner();
        let request = Self {
            protocol_version: MAPLE_PAIRING_PROTOCOL_VERSION,
            transcript_version: MAPLE_PAIRING_TRANSCRIPT_VERSION,
            operation_id,
            asserted_account_id: reset.subject_account_id,
            asserted_project_id: reset.subject_project_id,
            host_registration_id: reset.recipient_host_registration_id,
            revocation_stream_id: reset.revocation_stream_id,
            revocation_stream_generation: reset.revocation_stream_generation,
            event_id: reset.event_id,
            issuer_sequence: 1,
            event_digest: reset.event_digest()?,
            expected_previous_issuer_sequence: 0,
            signature: String::new(),
        };
        request.canonical_transcript()?;
        Ok(PreparedMaplePairingRevocationAckV1 { request })
    }

    pub fn canonical_transcript(&self) -> Result<Vec<u8>> {
        validate_versioned_request(self.protocol_version, self.transcript_version)?;
        validate_non_nil(&[
            ("operation ID", self.operation_id),
            ("asserted account ID", self.asserted_account_id),
            ("asserted project ID", self.asserted_project_id),
            ("host registration ID", self.host_registration_id),
            ("event ID", self.event_id),
        ])?;
        if self.issuer_sequence == 0
            || self.issuer_sequence > i64::MAX as u64
            || self.expected_previous_issuer_sequence > i64::MAX as u64
            || self.expected_previous_issuer_sequence.checked_add(1) != Some(self.issuer_sequence)
        {
            return Err(Error::Configuration(
                "Maple revocation acknowledgement sequence must immediately follow its durable predecessor"
                    .to_string(),
            ));
        }
        validate_revocation_stream(
            self.revocation_stream_id,
            self.revocation_stream_generation,
            ErrorClass::Configuration,
        )?;
        let event_digest = decode_exact_base64(
            &self.event_digest,
            DIGEST_BYTES,
            "Maple pairing revocation event digest",
            ErrorClass::Configuration,
        )?;
        let mut canonical = CanonicalBytes::new("os.maple-pair-revocation-ack.v1");
        canonical
            .append_u16(self.protocol_version)
            .append_u16(self.transcript_version)
            .append_uuid(self.asserted_account_id)
            .append_uuid(self.asserted_project_id)
            .append_uuid(self.operation_id)
            .append_uuid(self.host_registration_id)
            .append_uuid(self.revocation_stream_id)
            .append_u64(self.revocation_stream_generation)
            .append_uuid(self.event_id)
            .append_u64(self.issuer_sequence)
            .append_bytes(&event_digest)
            .append_u64(self.expected_previous_issuer_sequence);
        Ok(canonical.into_bytes())
    }

    pub fn validate_with_signing_key(&self, public_key: &[u8; 32]) -> Result<()> {
        validate_request_signature(public_key, &self.canonical_transcript()?, &self.signature)
    }
}

impl MapleRevocationStreamCheckpointV1 {
    pub fn canonical_transcript(&self) -> Result<Vec<u8>> {
        self.validate_fields()?;
        let mut canonical = CanonicalBytes::new("os.maple-revocation-stream-checkpoint.v1");
        canonical
            .append_u16(self.artifact_version)
            .append_uuid(self.subject_account_id)
            .append_uuid(self.subject_project_id);
        append_device_claim(&mut canonical, &self.host, ErrorClass::InvalidResponse)?;
        canonical
            .append_u64(self.security_epoch)
            .append_uuid(self.revocation_stream_id)
            .append_u64(self.revocation_stream_generation)
            .append_u64(self.last_issued_issuer_sequence)
            .append_u64(self.last_acked_issuer_sequence)
            .append_str(&self.issuer_key_id);
        Ok(canonical.into_bytes())
    }

    pub fn transcript_digest(&self) -> Result<String> {
        Ok(sha256_base64(&self.canonical_transcript()?))
    }

    pub fn verify(
        &self,
        issuers: &MaplePairingIssuerKeySet,
    ) -> Result<VerifiedMapleRevocationStreamCheckpointV1> {
        let transcript = self.canonical_transcript()?;
        issuers.verify(&self.issuer_key_id, &transcript, &self.issuer_signature)?;
        Ok(VerifiedMapleRevocationStreamCheckpointV1(self.clone()))
    }

    fn validate_fields(&self) -> Result<()> {
        if self.artifact_version != MAPLE_PAIRING_ARTIFACT_VERSION
            || self.subject_account_id.is_nil()
            || self.subject_project_id.is_nil()
            || self.security_epoch == 0
            || self.security_epoch > i64::MAX as u64
            || self.last_issued_issuer_sequence > i64::MAX as u64
            || self.last_acked_issuer_sequence > self.last_issued_issuer_sequence
        {
            return Err(Error::InvalidResponse(
                "Maple revocation stream checkpoint contains invalid scope or sequence fields"
                    .to_string(),
            ));
        }
        validate_device_claim(&self.host, ErrorClass::InvalidResponse)?;
        validate_revocation_stream(
            self.revocation_stream_id,
            self.revocation_stream_generation,
            ErrorClass::InvalidResponse,
        )?;
        validate_response_token(
            &self.issuer_key_id,
            MAX_ISSUER_KEY_ID_BYTES,
            "issuer key ID",
        )
    }
}

impl MapleResetClearRequiredV1 {
    /// Validates the complete reset-clear shape and both locally recomputable
    /// SHA-256 commitments. Issuer trust is checked separately by `verify`.
    pub fn validate(&self) -> Result<()> {
        validate_reset_clear_unsigned(self)?;
        decode_exact_base64(
            &self.issuer_signature,
            ED25519_SIGNATURE_BYTES,
            "Maple reset-clear issuer signature",
            ErrorClass::InvalidResponse,
        )?;
        Ok(())
    }

    pub fn canonical_transcript(&self) -> Result<Vec<u8>> {
        reset_clear_required_transcript(self)
    }

    pub fn event_digest(&self) -> Result<String> {
        Ok(sha256_base64(&self.canonical_transcript()?))
    }

    pub fn verify(
        &self,
        issuers: &MaplePairingIssuerKeySet,
    ) -> Result<VerifiedMapleResetClearRequiredV1> {
        let transcript = self.canonical_transcript()?;
        issuers.verify(&self.issuer_key_id, &transcript, &self.issuer_signature)?;
        Ok(VerifiedMapleResetClearRequiredV1(self.clone()))
    }

    pub fn verify_against_checkpoint(
        &self,
        issuers: &MaplePairingIssuerKeySet,
        checkpoint: &VerifiedMapleRevocationStreamCheckpointV1,
    ) -> Result<VerifiedLatestMapleResetClearRequiredV1> {
        self.verify(issuers)?.bind_to_checkpoint(checkpoint)
    }
}

pub fn sign_reset_clear_required(
    mut instruction: MapleResetClearRequiredV1,
    issuer: &dyn MaplePairingIssuer,
) -> Result<MapleResetClearRequiredV1> {
    instruction.instruction_material_digest.clear();
    instruction.chain_digest.clear();
    instruction.issuer_key_id.clear();
    instruction.issuer_signature.clear();

    let material_digest =
        Sha256::digest(reset_clear_instruction_material_transcript(&instruction)?);
    instruction.instruction_material_digest = BASE64.encode(material_digest);
    let chain_digest = Sha256::digest(reset_clear_chain_transcript(&instruction)?);
    instruction.chain_digest = BASE64.encode(chain_digest);
    instruction.issuer_key_id = issuer.key_id().to_string();
    validate_response_token(
        &instruction.issuer_key_id,
        MAX_ISSUER_KEY_ID_BYTES,
        "reset-clear issuer key ID",
    )?;
    let expected_public_key = issuer.public_key_bytes();
    if VerifyingKey::from_bytes(&expected_public_key).is_err() {
        return Err(Error::Configuration(
            "Maple reset-clear issuer public key is invalid".to_string(),
        ));
    }
    instruction.issuer_signature =
        BASE64.encode(issuer.sign(&instruction.canonical_transcript()?)?);
    instruction.validate()?;
    Ok(instruction)
}

impl MapleRevocationStreamEventV1 {
    pub fn issuer_sequence(&self) -> u64 {
        match self {
            Self::PairRevocation(event) => event.issuer_sequence,
            Self::ResetClearRequired(event) => event.issuer_sequence,
        }
    }

    pub fn event_id(&self) -> Uuid {
        match self {
            Self::PairRevocation(event) => event.event_id,
            Self::ResetClearRequired(event) => event.event_id,
        }
    }

    pub fn event_digest(&self) -> Result<String> {
        match self {
            Self::PairRevocation(event) => event.transcript_digest(),
            Self::ResetClearRequired(event) => event.event_digest(),
        }
    }

    pub fn verify(
        &self,
        issuers: &MaplePairingIssuerKeySet,
    ) -> Result<VerifiedMapleRevocationStreamEventV1> {
        match self {
            Self::PairRevocation(event) => Ok(
                VerifiedMapleRevocationStreamEventV1::PairRevocation(event.verify_issuer(issuers)?),
            ),
            Self::ResetClearRequired(event) => Ok(
                VerifiedMapleRevocationStreamEventV1::ResetClearRequired(event.verify(issuers)?),
            ),
        }
    }
}

impl MapleRevocationSyncV1 {
    pub fn validate(&self) -> Result<()> {
        validate_reset_counter(
            self.security_epoch,
            "sync security epoch",
            ErrorClass::InvalidResponse,
        )?;
        self.stream_checkpoint.validate_fields()?;
        if self.security_epoch != self.stream_checkpoint.security_epoch {
            return Err(Error::InvalidResponse(
                "Maple revocation sync security epoch does not match its checkpoint".to_string(),
            ));
        }
        match (self.status, self.reset_clear_instruction.as_ref()) {
            (MapleRevocationSyncStatusV1::Ready, None)
                if self.stream_checkpoint.last_issued_issuer_sequence
                    == self.stream_checkpoint.last_acked_issuer_sequence => {}
            (MapleRevocationSyncStatusV1::RevocationsPending, None)
                if self.stream_checkpoint.last_issued_issuer_sequence
                    > self.stream_checkpoint.last_acked_issuer_sequence => {}
            (MapleRevocationSyncStatusV1::ResetClearRequired, Some(reset))
                if self.stream_checkpoint.last_issued_issuer_sequence == 1
                    && self.stream_checkpoint.last_acked_issuer_sequence == 0 =>
            {
                reset.validate()?;
                if reset.security_epoch != self.security_epoch
                    || reset.subject_account_id != self.stream_checkpoint.subject_account_id
                    || reset.subject_project_id != self.stream_checkpoint.subject_project_id
                    || reset.recipient_host_registration_id
                        != self.stream_checkpoint.host.registration_id
                    || !device_claim_is_same_identity_at_or_before(
                        &reset.host,
                        &self.stream_checkpoint.host,
                    )
                    || reset.revocation_stream_id != self.stream_checkpoint.revocation_stream_id
                    || reset.revocation_stream_generation
                        != self.stream_checkpoint.revocation_stream_generation
                {
                    return Err(Error::InvalidResponse(
                        "Maple reset-clear sync does not match its target checkpoint".to_string(),
                    ));
                }
            }
            _ => {
                return Err(Error::InvalidResponse(
                    "Maple revocation sync status, checkpoint, and reset instruction disagree"
                        .to_string(),
                ))
            }
        }
        Ok(())
    }

    pub fn verify(
        &self,
        issuers: &MaplePairingIssuerKeySet,
    ) -> Result<VerifiedMapleRevocationSyncV1> {
        self.validate()?;
        let checkpoint = self.stream_checkpoint.verify(issuers)?;
        let reset_clear_instruction = self
            .reset_clear_instruction
            .as_ref()
            .map(|reset| reset.verify_against_checkpoint(issuers, &checkpoint))
            .transpose()?;
        Ok(VerifiedMapleRevocationSyncV1 {
            sync: self.clone(),
            checkpoint,
            reset_clear_instruction,
        })
    }

    /// Verifies this sync against the exact accepted registration request and
    /// receipt identity. The authenticated account/project remain server-owned;
    /// these signed values are stale-state and response-binding preconditions.
    pub fn verify_against_registration(
        &self,
        request: &crate::types::RegisterMapleDeviceRequest,
        registration_id: Uuid,
        response_security_epoch: u64,
        issuers: &MaplePairingIssuerKeySet,
    ) -> Result<VerifiedMapleRevocationSyncV1> {
        let verified = self.verify(issuers)?;
        let checkpoint = verified.checkpoint.as_inner();
        let identity_algorithm_matches = matches!(
            (
                request.identity_algorithm,
                checkpoint.host.identity_algorithm
            ),
            (
                crate::types::MapleDeviceIdentityAlgorithm::Ed25519,
                MaplePairingIdentityAlgorithm::Ed25519
            )
        );
        if registration_id.is_nil()
            || response_security_epoch != request.known_security_epoch
            || self.security_epoch != response_security_epoch
            || checkpoint.subject_account_id != request.asserted_account_id
            || checkpoint.subject_project_id != request.asserted_project_id
            || checkpoint.host.registration_id != registration_id
            || checkpoint.host.device_id != request.device_id
            || checkpoint.host.installation_id != request.installation_id
            || !identity_algorithm_matches
            || checkpoint.host.identity_public_key != request.identity_public_key
            || checkpoint.host.endpoint_id != request.iroh_endpoint_id
            || checkpoint.host.endpoint_epoch != request.endpoint_epoch
        {
            return Err(Error::InvalidResponse(
                "Maple revocation sync does not bind the accepted device registration".to_string(),
            ));
        }
        Ok(verified)
    }
}

impl MaplePairRequestTicketV1 {
    pub fn canonical_transcript(&self) -> Result<Vec<u8>> {
        self.validate_fields()?;
        let request_nonce = decode_exact_base64(
            &self.pairing_request_nonce,
            NONCE_BYTES,
            "Maple pairing artifact request nonce",
            ErrorClass::InvalidResponse,
        )?;
        let request_digest = decode_exact_base64(
            &self.controller_request_digest,
            DIGEST_BYTES,
            "Maple pairing controller request digest",
            ErrorClass::InvalidResponse,
        )?;
        let request_signature = decode_exact_base64(
            &self.controller_request_signature,
            ED25519_SIGNATURE_BYTES,
            "Maple pairing controller request signature",
            ErrorClass::InvalidResponse,
        )?;
        let mut canonical = CanonicalBytes::new("os.maple-pair-request-ticket.v1");
        canonical
            .append_u16(self.artifact_version)
            .append_uuid(self.subject_account_id)
            .append_uuid(self.subject_project_id)
            .append_uuid(self.pairing_request_id)
            .append_uuid(self.pair_id)
            .append_str(self.direction.as_str())
            .append_uuid(self.execution_target_id);
        append_device_claim(
            &mut canonical,
            &self.controller,
            ErrorClass::InvalidResponse,
        )?;
        append_device_claim(&mut canonical, &self.host, ErrorClass::InvalidResponse)?;
        canonical
            .append_bytes(&request_nonce)
            .append_uuid(self.controller_request_operation_id)
            .append_bytes(&request_digest)
            .append_bytes(&request_signature)
            .append_u64(self.pairing_incarnation)
            .append_u16(self.protocol_min)
            .append_u16(self.protocol_max)
            .append_i64(self.created_at_unix_ms)
            .append_i64(self.expires_at_unix_ms)
            .append_str(&self.issuer_key_id);
        Ok(canonical.into_bytes())
    }

    pub fn transcript_digest(&self) -> Result<String> {
        Ok(sha256_base64(&self.canonical_transcript()?))
    }

    pub fn verify_at(
        &self,
        issuers: &MaplePairingIssuerKeySet,
        trusted_now_unix_ms: i64,
        max_clock_skew_ms: i64,
        max_ticket_ttl_ms: i64,
    ) -> Result<VerifiedMaplePairRequestTicketV1> {
        if trusted_now_unix_ms <= 0 {
            return Err(Error::Configuration(
                "Maple pairing ticket verification requires a positive trusted current time"
                    .to_string(),
            ));
        }
        if !(0..=MAPLE_PAIRING_MAX_CLOCK_SKEW_MS).contains(&max_clock_skew_ms) {
            return Err(Error::Configuration(format!(
                "Maple pairing clock skew must be between 0 and {MAPLE_PAIRING_MAX_CLOCK_SKEW_MS} milliseconds"
            )));
        }
        if !(1..=MAPLE_PAIRING_MAX_TICKET_TTL_MS).contains(&max_ticket_ttl_ms) {
            return Err(Error::Configuration(format!(
                "Maple pairing ticket TTL must be between 1 and {MAPLE_PAIRING_MAX_TICKET_TTL_MS} milliseconds"
            )));
        }
        self.verify_signatures(issuers)?;
        if self
            .expires_at_unix_ms
            .checked_sub(self.created_at_unix_ms)
            .is_none_or(|ttl| ttl > max_ticket_ttl_ms)
            || self
                .created_at_unix_ms
                .checked_sub(max_clock_skew_ms)
                .is_none_or(|earliest| trusted_now_unix_ms < earliest)
            || self
                .expires_at_unix_ms
                .checked_add(max_clock_skew_ms)
                .is_none_or(|latest| trusted_now_unix_ms >= latest)
        {
            return Err(Error::InvalidResponse(
                "Maple pairing request ticket is outside its trusted time window".to_string(),
            ));
        }
        Ok(VerifiedMaplePairRequestTicketV1(self.clone()))
    }

    fn verify_signatures(&self, issuers: &MaplePairingIssuerKeySet) -> Result<()> {
        let transcript = self.canonical_transcript()?;
        self.verify_controller_request()?;
        issuers.verify(&self.issuer_key_id, &transcript, &self.issuer_signature)
    }

    fn validate_fields(&self) -> Result<()> {
        if self.artifact_version != MAPLE_PAIRING_ARTIFACT_VERSION {
            return Err(Error::InvalidResponse(
                "Unsupported Maple pairing request-ticket artifact version".to_string(),
            ));
        }
        if self.subject_account_id.is_nil()
            || self.subject_project_id.is_nil()
            || self.pairing_request_id.is_nil()
            || self.pair_id.is_nil()
            || self.execution_target_id.is_nil()
            || self.controller_request_operation_id.is_nil()
            || self.execution_target_id != self.host.registration_id
        {
            return Err(Error::InvalidResponse(
                "Maple pairing request ticket contains invalid identity bindings".to_string(),
            ));
        }
        validate_device_claim(&self.controller, ErrorClass::InvalidResponse)?;
        validate_device_claim(&self.host, ErrorClass::InvalidResponse)?;
        validate_directed_device_claims(
            &self.controller,
            &self.host,
            self.execution_target_id,
            ErrorClass::InvalidResponse,
        )?;
        if self.pairing_incarnation == 0 || self.pairing_incarnation > i64::MAX as u64 {
            return Err(Error::InvalidResponse(
                "Maple pairing request ticket has an invalid incarnation".to_string(),
            ));
        }
        if self.protocol_min == 0 || self.protocol_min > self.protocol_max {
            return Err(Error::InvalidResponse(
                "Maple pairing request ticket has an invalid protocol range".to_string(),
            ));
        }
        validate_timestamp(
            self.created_at_unix_ms,
            "created timestamp",
            ErrorClass::InvalidResponse,
        )?;
        validate_timestamp(
            self.expires_at_unix_ms,
            "expiry timestamp",
            ErrorClass::InvalidResponse,
        )?;
        if self.expires_at_unix_ms <= self.created_at_unix_ms {
            return Err(Error::InvalidResponse(
                "Maple pairing request ticket expiry must follow creation".to_string(),
            ));
        }
        if self
            .expires_at_unix_ms
            .checked_sub(self.created_at_unix_ms)
            .is_none_or(|ttl| ttl > MAPLE_PAIRING_MAX_TICKET_TTL_MS)
        {
            return Err(Error::InvalidResponse(format!(
                "Maple pairing request ticket lifetime exceeds {MAPLE_PAIRING_MAX_TICKET_TTL_MS} milliseconds"
            )));
        }
        validate_response_token(
            &self.issuer_key_id,
            MAX_ISSUER_KEY_ID_BYTES,
            "issuer key ID",
        )?;
        Ok(())
    }

    fn controller_request(&self) -> CreateMaplePairingRequest {
        CreateMaplePairingRequest {
            protocol_version: MAPLE_PAIRING_PROTOCOL_VERSION,
            transcript_version: MAPLE_PAIRING_TRANSCRIPT_VERSION,
            operation_id: self.controller_request_operation_id,
            asserted_account_id: self.subject_account_id,
            asserted_project_id: self.subject_project_id,
            controller_registration_id: self.controller.registration_id,
            controller_device_id: self.controller.device_id,
            controller_installation_id: self.controller.installation_id,
            controller_endpoint_id: self.controller.endpoint_id.clone(),
            controller_endpoint_epoch: self.controller.endpoint_epoch,
            host_registration_id: self.host.registration_id,
            host_device_id: self.host.device_id,
            host_installation_id: self.host.installation_id,
            host_endpoint_id: self.host.endpoint_id.clone(),
            host_endpoint_epoch: self.host.endpoint_epoch,
            direction: self.direction,
            execution_target_id: self.execution_target_id,
            pairing_request_nonce: self.pairing_request_nonce.clone(),
            protocol_min: self.protocol_min,
            protocol_max: self.protocol_max,
            signature: self.controller_request_signature.clone(),
        }
    }

    fn verify_controller_request(&self) -> Result<()> {
        let request = self.controller_request();
        let transcript = request
            .canonical_transcript()
            .map_err(configuration_as_invalid_response)?;
        if sha256_base64(&transcript) != self.controller_request_digest {
            return Err(Error::InvalidResponse(
                "Maple pairing request ticket controller digest does not match its claims"
                    .to_string(),
            ));
        }
        let controller_key =
            validate_device_claim(&self.controller, ErrorClass::InvalidResponse)?.0;
        let signature = decode_exact_base64(
            &self.controller_request_signature,
            ED25519_SIGNATURE_BYTES,
            "Maple pairing controller request signature",
            ErrorClass::InvalidResponse,
        )?;
        let signature: [u8; ED25519_SIGNATURE_BYTES] = signature
            .try_into()
            .expect("exact signature length was checked");
        let controller_key = VerifyingKey::from_bytes(&controller_key).map_err(|_| {
            Error::InvalidResponse(
                "Maple pairing controller key is not a valid Ed25519 point".to_string(),
            )
        })?;
        controller_key
            .verify_strict(&transcript, &Signature::from_bytes(&signature))
            .map_err(|_| {
                Error::InvalidResponse(
                    "Maple pairing controller request signature does not verify".to_string(),
                )
            })
    }
}

fn configuration_as_invalid_response(error: Error) -> Error {
    match error {
        Error::Configuration(message) => Error::InvalidResponse(message),
        other => other,
    }
}

impl MaplePairAuthorizationV1 {
    pub fn canonical_transcript(&self) -> Result<Vec<u8>> {
        self.validate_fields()?;
        let request_nonce =
            decode_response_bytes(&self.pairing_request_nonce, NONCE_BYTES, "request nonce")?;
        let request_digest = decode_response_bytes(
            &self.controller_request_digest,
            DIGEST_BYTES,
            "controller request digest",
        )?;
        let request_signature = decode_response_bytes(
            &self.controller_request_signature,
            ED25519_SIGNATURE_BYTES,
            "controller request signature",
        )?;
        let ticket_digest = decode_response_bytes(
            &self.request_ticket_digest,
            DIGEST_BYTES,
            "request ticket digest",
        )?;
        let approval_nonce = decode_response_bytes(
            &self.host_approval_nonce,
            NONCE_BYTES,
            "host approval nonce",
        )?;
        let approval_digest = decode_response_bytes(
            &self.host_approval_digest,
            DIGEST_BYTES,
            "host approval digest",
        )?;
        let approval_signature = decode_response_bytes(
            &self.host_approval_signature,
            ED25519_SIGNATURE_BYTES,
            "host approval signature",
        )?;
        let mut canonical = CanonicalBytes::new("os.maple-pair-authorization.v1");
        canonical
            .append_u16(self.artifact_version)
            .append_uuid(self.subject_account_id)
            .append_uuid(self.subject_project_id)
            .append_uuid(self.pairing_request_id)
            .append_uuid(self.pair_id)
            .append_str(self.direction.as_str())
            .append_uuid(self.execution_target_id);
        append_device_claim(
            &mut canonical,
            &self.controller,
            ErrorClass::InvalidResponse,
        )?;
        append_device_claim(&mut canonical, &self.host, ErrorClass::InvalidResponse)?;
        canonical
            .append_bytes(&request_nonce)
            .append_uuid(self.controller_request_operation_id)
            .append_bytes(&request_digest)
            .append_bytes(&request_signature)
            .append_bytes(&ticket_digest)
            .append_uuid(self.host_approval_operation_id)
            .append_i64(self.host_approval_expected_pairing_revision)
            .append_bytes(&approval_nonce)
            .append_bytes(&approval_digest)
            .append_bytes(&approval_signature)
            .append_u64(self.pairing_incarnation)
            .append_uuid(self.revocation_stream_id)
            .append_u64(self.revocation_stream_generation)
            .append_u16(self.protocol_min)
            .append_u16(self.protocol_max)
            .append_i64(self.approved_at_unix_ms)
            .append_str(&self.issuer_key_id);
        Ok(canonical.into_bytes())
    }

    pub fn transcript_digest(&self) -> Result<String> {
        Ok(sha256_base64(&self.canonical_transcript()?))
    }

    /// Verifies the issuer, both embedded device proofs, and the exact signed
    /// request ticket that this authorization approves.
    ///
    /// An issuer signature alone is insufficient for the authority wrapper:
    /// callers must supply the ticket so `request_ticket_digest` and every pair
    /// claim are checked before this value can be used to construct a host
    /// commit.
    pub fn verify_against_ticket(
        &self,
        issuers: &MaplePairingIssuerKeySet,
        ticket: &MaplePairRequestTicketV1,
    ) -> Result<VerifiedMaplePairAuthorizationV1> {
        // Validate the ticket at the issuer-signed approval time. This rejects
        // authorizations issued outside the request window while preserving
        // restart recovery after the ticket later expires.
        ticket.verify_at(
            issuers,
            self.approved_at_unix_ms,
            MAPLE_PAIRING_MAX_CLOCK_SKEW_MS,
            MAPLE_PAIRING_MAX_TICKET_TTL_MS,
        )?;
        self.verify_signatures(issuers)?;
        self.validate_ticket_binding(ticket)?;
        Ok(VerifiedMaplePairAuthorizationV1(self.clone()))
    }

    fn verify_signatures(&self, issuers: &MaplePairingIssuerKeySet) -> Result<()> {
        let transcript = self.canonical_transcript()?;
        self.verify_controller_request()?;
        self.verify_host_approval()?;
        issuers.verify(&self.issuer_key_id, &transcript, &self.issuer_signature)
    }

    fn validate_ticket_binding(&self, ticket: &MaplePairRequestTicketV1) -> Result<()> {
        if self.subject_account_id != ticket.subject_account_id
            || self.subject_project_id != ticket.subject_project_id
            || self.pairing_request_id != ticket.pairing_request_id
            || self.pair_id != ticket.pair_id
            || self.direction != ticket.direction
            || self.execution_target_id != ticket.execution_target_id
            || self.controller != ticket.controller
            || self.host != ticket.host
            || self.pairing_request_nonce != ticket.pairing_request_nonce
            || self.controller_request_operation_id != ticket.controller_request_operation_id
            || self.controller_request_digest != ticket.controller_request_digest
            || self.controller_request_signature != ticket.controller_request_signature
            || self.pairing_incarnation != ticket.pairing_incarnation
            || self.protocol_min != ticket.protocol_min
            || self.protocol_max != ticket.protocol_max
            || self.request_ticket_digest != ticket.transcript_digest()?
            || self.approved_at_unix_ms < ticket.created_at_unix_ms
        {
            return Err(Error::InvalidResponse(
                "Maple pair authorization does not match its issuer-verified request ticket"
                    .to_string(),
            ));
        }
        Ok(())
    }

    fn validate_fields(&self) -> Result<()> {
        if self.artifact_version != MAPLE_PAIRING_ARTIFACT_VERSION
            || self.subject_account_id.is_nil()
            || self.subject_project_id.is_nil()
            || self.pairing_request_id.is_nil()
            || self.pair_id.is_nil()
            || self.execution_target_id.is_nil()
            || self.controller_request_operation_id.is_nil()
            || self.host_approval_operation_id.is_nil()
            || self.execution_target_id != self.host.registration_id
            || self.host_approval_expected_pairing_revision != 1
        {
            return Err(Error::InvalidResponse(
                "Maple pair authorization contains invalid identity or revision bindings"
                    .to_string(),
            ));
        }
        validate_device_claim(&self.controller, ErrorClass::InvalidResponse)?;
        validate_device_claim(&self.host, ErrorClass::InvalidResponse)?;
        validate_directed_device_claims(
            &self.controller,
            &self.host,
            self.execution_target_id,
            ErrorClass::InvalidResponse,
        )?;
        if self.pairing_incarnation == 0
            || self.pairing_incarnation > i64::MAX as u64
            || self.protocol_min == 0
            || self.protocol_min > self.protocol_max
        {
            return Err(Error::InvalidResponse(
                "Maple pair authorization has an invalid incarnation or protocol range".to_string(),
            ));
        }
        validate_revocation_stream(
            self.revocation_stream_id,
            self.revocation_stream_generation,
            ErrorClass::InvalidResponse,
        )?;
        validate_timestamp(
            self.approved_at_unix_ms,
            "approval timestamp",
            ErrorClass::InvalidResponse,
        )?;
        validate_response_token(
            &self.issuer_key_id,
            MAX_ISSUER_KEY_ID_BYTES,
            "issuer key ID",
        )?;
        Ok(())
    }

    fn controller_request(&self) -> CreateMaplePairingRequest {
        CreateMaplePairingRequest {
            protocol_version: MAPLE_PAIRING_PROTOCOL_VERSION,
            transcript_version: MAPLE_PAIRING_TRANSCRIPT_VERSION,
            operation_id: self.controller_request_operation_id,
            asserted_account_id: self.subject_account_id,
            asserted_project_id: self.subject_project_id,
            controller_registration_id: self.controller.registration_id,
            controller_device_id: self.controller.device_id,
            controller_installation_id: self.controller.installation_id,
            controller_endpoint_id: self.controller.endpoint_id.clone(),
            controller_endpoint_epoch: self.controller.endpoint_epoch,
            host_registration_id: self.host.registration_id,
            host_device_id: self.host.device_id,
            host_installation_id: self.host.installation_id,
            host_endpoint_id: self.host.endpoint_id.clone(),
            host_endpoint_epoch: self.host.endpoint_epoch,
            direction: self.direction,
            execution_target_id: self.execution_target_id,
            pairing_request_nonce: self.pairing_request_nonce.clone(),
            protocol_min: self.protocol_min,
            protocol_max: self.protocol_max,
            signature: self.controller_request_signature.clone(),
        }
    }

    fn verify_controller_request(&self) -> Result<()> {
        let request = self.controller_request();
        let transcript = request
            .canonical_transcript()
            .map_err(configuration_as_invalid_response)?;
        if sha256_base64(&transcript) != self.controller_request_digest {
            return Err(Error::InvalidResponse(
                "Maple pair authorization controller digest mismatch".to_string(),
            ));
        }
        verify_embedded_signature(
            &self.controller,
            &transcript,
            &self.controller_request_signature,
            "controller request",
        )
    }

    fn verify_host_approval(&self) -> Result<()> {
        let approval = ApproveMaplePairingRequest {
            protocol_version: MAPLE_PAIRING_PROTOCOL_VERSION,
            transcript_version: MAPLE_PAIRING_TRANSCRIPT_VERSION,
            operation_id: self.host_approval_operation_id,
            asserted_account_id: self.subject_account_id,
            asserted_project_id: self.subject_project_id,
            host_registration_id: self.host.registration_id,
            pairing_request_id: self.pairing_request_id,
            pair_id: self.pair_id,
            expected_pairing_revision: self.host_approval_expected_pairing_revision,
            pairing_incarnation: self.pairing_incarnation,
            revocation_stream_id: self.revocation_stream_id,
            revocation_stream_generation: self.revocation_stream_generation,
            request_ticket_digest: self.request_ticket_digest.clone(),
            host_approval_nonce: self.host_approval_nonce.clone(),
            approved_protocol_min: self.protocol_min,
            approved_protocol_max: self.protocol_max,
            signature: self.host_approval_signature.clone(),
        };
        let transcript = approval
            .canonical_transcript()
            .map_err(configuration_as_invalid_response)?;
        if sha256_base64(&transcript) != self.host_approval_digest {
            return Err(Error::InvalidResponse(
                "Maple pair authorization host approval digest mismatch".to_string(),
            ));
        }
        verify_embedded_signature(
            &self.host,
            &transcript,
            &self.host_approval_signature,
            "host approval",
        )
    }
}

fn decode_response_bytes(value: &str, length: usize, field: &str) -> Result<Vec<u8>> {
    decode_exact_base64(
        value,
        length,
        &format!("Maple pairing {field}"),
        ErrorClass::InvalidResponse,
    )
}

fn verify_embedded_signature(
    claim: &MaplePairingDeviceClaimV1,
    transcript: &[u8],
    signature: &str,
    label: &str,
) -> Result<()> {
    let public_key = validate_device_claim(claim, ErrorClass::InvalidResponse)?.0;
    let signature = decode_response_bytes(signature, ED25519_SIGNATURE_BYTES, label)?;
    let signature: [u8; ED25519_SIGNATURE_BYTES] = signature
        .try_into()
        .expect("exact signature length was checked");
    let public_key = VerifyingKey::from_bytes(&public_key).map_err(|_| {
        Error::InvalidResponse(format!(
            "Maple pairing {label} key is not a valid Ed25519 point"
        ))
    })?;
    public_key
        .verify_strict(transcript, &Signature::from_bytes(&signature))
        .map_err(|_| {
            Error::InvalidResponse(format!("Maple pairing {label} signature does not verify"))
        })
}

impl MaplePairRevocationV1 {
    pub fn canonical_transcript(&self) -> Result<Vec<u8>> {
        self.validate_fields()?;
        let authorization_digest = self.authorization_digest_bytes()?;
        let mut canonical = CanonicalBytes::new("os.maple-pair-revocation.v1");
        canonical
            .append_u16(self.artifact_version)
            .append_uuid(self.event_id)
            .append_uuid(self.subject_account_id)
            .append_uuid(self.subject_project_id)
            .append_uuid(self.recipient_host_registration_id)
            .append_u64(self.issuer_sequence)
            .append_uuid(self.revocation_stream_id)
            .append_u64(self.revocation_stream_generation)
            .append_uuid(self.pairing_request_id)
            .append_uuid(self.pair_id)
            .append_str(self.direction.as_str())
            .append_uuid(self.execution_target_id);
        append_device_claim(
            &mut canonical,
            &self.controller,
            ErrorClass::InvalidResponse,
        )?;
        append_device_claim(&mut canonical, &self.host, ErrorClass::InvalidResponse)?;
        canonical
            .append_u64(self.pairing_incarnation)
            .append_bytes(&authorization_digest)
            .append_uuid(self.revoked_by_registration_id)
            .append_str(self.revoked_by_role.as_str())
            .append_str(&self.reason_code)
            .append_i64(self.revoked_at_unix_ms)
            .append_str(&self.issuer_key_id);
        Ok(canonical.into_bytes())
    }

    pub fn transcript_digest(&self) -> Result<String> {
        Ok(sha256_base64(&self.canonical_transcript()?))
    }

    /// Verifies the issuer signature and structural event claims, but does not
    /// yet make the event safe to durably apply or acknowledge.
    ///
    /// Revocation pages cannot prove that `pair_authorization_digest` matches
    /// the host's durable authorization. Call
    /// [`IssuerVerifiedMaplePairRevocationV1::bind_to_authorization`] with the
    /// host's ticket-bound authorization before updating an allowlist.
    pub fn verify_issuer(
        &self,
        issuers: &MaplePairingIssuerKeySet,
    ) -> Result<IssuerVerifiedMaplePairRevocationV1> {
        let transcript = self.canonical_transcript()?;
        issuers.verify(&self.issuer_key_id, &transcript, &self.issuer_signature)?;
        Ok(IssuerVerifiedMaplePairRevocationV1(self.clone()))
    }

    pub fn verify_against_authorization(
        &self,
        issuers: &MaplePairingIssuerKeySet,
        authorization: &VerifiedMaplePairAuthorizationV1,
        revocation_stream: &DurablyReconciledMapleRevocationStreamV1,
    ) -> Result<VerifiedMaplePairRevocationV1> {
        self.verify_issuer(issuers)?
            .bind_to_authorization(authorization, revocation_stream)
    }

    fn validate_authorization_binding(
        &self,
        authorization: &MaplePairAuthorizationV1,
    ) -> Result<()> {
        if self.pair_authorization_digest != authorization.transcript_digest()?
            || self.subject_account_id != authorization.subject_account_id
            || self.subject_project_id != authorization.subject_project_id
            || self.pairing_request_id != authorization.pairing_request_id
            || self.pair_id != authorization.pair_id
            || self.direction != authorization.direction
            || self.execution_target_id != authorization.execution_target_id
            || self.controller != authorization.controller
            || self.host != authorization.host
            || self.pairing_incarnation != authorization.pairing_incarnation
            || self.revocation_stream_id != authorization.revocation_stream_id
            || self.revocation_stream_generation != authorization.revocation_stream_generation
            || self.revoked_at_unix_ms < authorization.approved_at_unix_ms
        {
            return Err(Error::InvalidResponse(
                "Maple pair revocation does not match the host's verified authorization"
                    .to_string(),
            ));
        }
        Ok(())
    }

    fn validate_fields(&self) -> Result<()> {
        if self.artifact_version != MAPLE_PAIRING_ARTIFACT_VERSION
            || self.event_id.is_nil()
            || self.subject_account_id.is_nil()
            || self.subject_project_id.is_nil()
            || self.recipient_host_registration_id.is_nil()
            || self.issuer_sequence == 0
            || self.issuer_sequence > i64::MAX as u64
            || self.pairing_request_id.is_nil()
            || self.pair_id.is_nil()
            || self.execution_target_id.is_nil()
            || self.revoked_by_registration_id.is_nil()
            || self.execution_target_id != self.host.registration_id
            || self.recipient_host_registration_id != self.host.registration_id
            || !matches!(
                (self.revoked_by_role, self.revoked_by_registration_id),
                (MaplePairingRole::Controller, registration) if registration == self.controller.registration_id
            ) && !matches!(
                (self.revoked_by_role, self.revoked_by_registration_id),
                (MaplePairingRole::Host, registration) if registration == self.host.registration_id
            )
        {
            return Err(Error::InvalidResponse(
                "Maple pair revocation contains invalid identity or sequence bindings".to_string(),
            ));
        }
        validate_device_claim(&self.controller, ErrorClass::InvalidResponse)?;
        validate_device_claim(&self.host, ErrorClass::InvalidResponse)?;
        validate_directed_device_claims(
            &self.controller,
            &self.host,
            self.execution_target_id,
            ErrorClass::InvalidResponse,
        )?;
        if self.pairing_incarnation == 0 || self.pairing_incarnation > i64::MAX as u64 {
            return Err(Error::InvalidResponse(
                "Maple pair revocation has an invalid pairing incarnation".to_string(),
            ));
        }
        validate_revocation_stream(
            self.revocation_stream_id,
            self.revocation_stream_generation,
            ErrorClass::InvalidResponse,
        )?;
        validate_response_token(
            &self.reason_code,
            MAX_REASON_CODE_BYTES,
            "revocation reason code",
        )?;
        validate_response_token(
            &self.issuer_key_id,
            MAX_ISSUER_KEY_ID_BYTES,
            "issuer key ID",
        )?;
        validate_timestamp(
            self.revoked_at_unix_ms,
            "revocation timestamp",
            ErrorClass::InvalidResponse,
        )
    }

    fn authorization_digest_bytes(&self) -> Result<Vec<u8>> {
        let digest = decode_response_bytes(
            &self.pair_authorization_digest,
            DIGEST_BYTES,
            "pair authorization digest",
        )?;
        if digest.iter().all(|byte| *byte == 0) {
            return Err(Error::InvalidResponse(
                "Maple pair revocation contains an all-zero authorization digest".to_string(),
            ));
        }
        Ok(digest)
    }
}

#[derive(Clone)]
pub struct VerifiedMaplePairRequestTicketV1(MaplePairRequestTicketV1);

redacted_debug!(VerifiedMaplePairRequestTicketV1);

impl VerifiedMaplePairRequestTicketV1 {
    pub fn as_inner(&self) -> &MaplePairRequestTicketV1 {
        &self.0
    }

    pub fn into_inner(self) -> MaplePairRequestTicketV1 {
        self.0
    }
}

#[derive(Clone)]
pub struct VerifiedMapleRevocationStreamCheckpointV1(MapleRevocationStreamCheckpointV1);

redacted_debug!(VerifiedMapleRevocationStreamCheckpointV1);

impl VerifiedMapleRevocationStreamCheckpointV1 {
    pub fn as_inner(&self) -> &MapleRevocationStreamCheckpointV1 {
        &self.0
    }

    pub fn into_inner(self) -> MapleRevocationStreamCheckpointV1 {
        self.0
    }

    pub fn namespace_matches(&self, stream_id: Uuid, generation: u64) -> bool {
        self.0.revocation_stream_id == stream_id
            && self.0.revocation_stream_generation == generation
    }

    /// Compares this signed checkpoint with the exact namespace loaded from
    /// the host's durable account-scoped admission store.
    ///
    /// Lower generations and same-generation/different-ID checkpoints fail
    /// closed. Missing or higher generations produce a reset-required plan;
    /// generation jumps are valid because destructive resets can consume
    /// namespace generations without ever issuing an admission.
    pub fn plan_durable_transition(
        self,
        installed: Option<MapleInstalledRevocationStreamNamespaceV1>,
    ) -> Result<MapleRevocationStreamTransitionV1> {
        let requires_admission_reset = match installed {
            None => true,
            Some(installed)
                if installed.subject_account_id != self.0.subject_account_id
                    || installed.subject_project_id != self.0.subject_project_id
                    || installed.host_registration_id != self.0.host.registration_id =>
            {
                return Err(Error::Configuration(
                    "Maple durable revocation namespace belongs to a different account or host"
                        .to_string(),
                ));
            }
            Some(installed) if self.0.security_epoch < installed.security_epoch => {
                return Err(Error::InvalidResponse(
                    "Maple revocation stream checkpoint security epoch is older than durable state"
                        .to_string(),
                ));
            }
            Some(installed) if self.0.security_epoch > installed.security_epoch => {
                return Err(Error::InvalidResponse(
                    "Maple revocation stream security epoch advanced without a verified reset-clear transition"
                        .to_string(),
                ));
            }
            Some(installed)
                if self.0.revocation_stream_generation < installed.revocation_stream_generation =>
            {
                return Err(Error::InvalidResponse(
                    "Maple revocation stream checkpoint generation is older than durable state"
                        .to_string(),
                ));
            }
            Some(installed)
                if self.0.revocation_stream_generation
                    == installed.revocation_stream_generation =>
            {
                if self.0.revocation_stream_id != installed.revocation_stream_id {
                    return Err(Error::InvalidResponse(
                        "Maple revocation stream ID changed without a generation increase"
                            .to_string(),
                    ));
                }
                false
            }
            Some(_) => true,
        };
        Ok(MapleRevocationStreamTransitionV1 {
            checkpoint: self.0,
            requires_admission_reset,
        })
    }
}

#[derive(Clone)]
pub struct VerifiedMapleResetClearRequiredV1(MapleResetClearRequiredV1);

redacted_debug!(VerifiedMapleResetClearRequiredV1);

impl VerifiedMapleResetClearRequiredV1 {
    pub fn as_inner(&self) -> &MapleResetClearRequiredV1 {
        &self.0
    }

    pub fn into_inner(self) -> MapleResetClearRequiredV1 {
        self.0
    }

    pub fn event_digest(&self) -> Result<String> {
        self.0.event_digest()
    }

    /// Verifies the full recursive link from an already issuer-verified
    /// predecessor. Genesis artifacts are verified directly by their issuer;
    /// each successor must advance exactly once while retaining the exact full
    /// host claim and authority scope. Even a valid endpoint-epoch refresh is
    /// forbidden until the predecessor clear has been durably acknowledged.
    pub fn verify_successor_of(
        &self,
        predecessor: &VerifiedMapleResetClearRequiredV1,
    ) -> Result<()> {
        let current = &self.0;
        let previous = &predecessor.0;
        if previous.reset_generation.checked_add(1) != Some(current.reset_generation)
            || previous.cumulative_reset_count.checked_add(1)
                != Some(current.cumulative_reset_count)
            || current.source_security_epoch != previous.security_epoch
            || current.subject_account_id != previous.subject_account_id
            || current.subject_project_id != previous.subject_project_id
            || current.recipient_host_registration_id != previous.recipient_host_registration_id
            || current.host != previous.host
            || current.source_revocation_stream_id != previous.revocation_stream_id
            || current.source_revocation_stream_generation != previous.revocation_stream_generation
            || current.previous_reset_clear_event_id != Some(previous.event_id)
            || current.previous_instruction_material_digest.as_deref()
                != Some(previous.instruction_material_digest.as_str())
            || current.previous_chain_digest.as_deref() != Some(previous.chain_digest.as_str())
            || current.reset_at_unix_ms < previous.reset_at_unix_ms
            || current.reset_id == previous.reset_id
            || current.event_id == previous.event_id
        {
            return Err(Error::InvalidResponse(
                "Maple reset-clear successor does not extend the verified predecessor exactly"
                    .to_string(),
            ));
        }
        Ok(())
    }

    fn bind_to_checkpoint(
        self,
        checkpoint: &VerifiedMapleRevocationStreamCheckpointV1,
    ) -> Result<VerifiedLatestMapleResetClearRequiredV1> {
        let reset = &self.0;
        let checkpoint = checkpoint.as_inner();
        if reset.subject_account_id != checkpoint.subject_account_id
            || reset.subject_project_id != checkpoint.subject_project_id
            || reset.recipient_host_registration_id != checkpoint.host.registration_id
            || !device_claim_is_same_identity_at_or_before(&reset.host, &checkpoint.host)
            || reset.security_epoch != checkpoint.security_epoch
            || reset.revocation_stream_id != checkpoint.revocation_stream_id
            || reset.revocation_stream_generation != checkpoint.revocation_stream_generation
            || reset.issuer_sequence != 1
            || checkpoint.last_issued_issuer_sequence != 1
            || checkpoint.last_acked_issuer_sequence != 0
        {
            return Err(Error::InvalidResponse(
                "Maple reset-clear instruction is not the pending head of its verified checkpoint"
                    .to_string(),
            ));
        }
        Ok(VerifiedLatestMapleResetClearRequiredV1(self.0))
    }

    /// Binds an issuer-verified retained reset-clear event to a signed
    /// checkpoint without treating it as a currently actionable clear
    /// instruction. Historical reset events always occupy sequence 1 of their
    /// target namespace, which may already be acknowledged or followed by
    /// ordinary revocations.
    pub fn verify_historical_against_checkpoint(
        &self,
        checkpoint: &VerifiedMapleRevocationStreamCheckpointV1,
    ) -> Result<()> {
        let reset = &self.0;
        let checkpoint = checkpoint.as_inner();
        if reset.subject_account_id != checkpoint.subject_account_id
            || reset.subject_project_id != checkpoint.subject_project_id
            || reset.recipient_host_registration_id != checkpoint.host.registration_id
            || !device_claim_is_same_identity_at_or_before(&reset.host, &checkpoint.host)
            || reset.security_epoch != checkpoint.security_epoch
            || reset.revocation_stream_id != checkpoint.revocation_stream_id
            || reset.revocation_stream_generation != checkpoint.revocation_stream_generation
            || reset.issuer_sequence != 1
            || reset.issuer_sequence > checkpoint.last_issued_issuer_sequence
            || reset.issuer_sequence > checkpoint.last_acked_issuer_sequence
        {
            return Err(Error::InvalidResponse(
                "Maple reset-clear history event does not belong to its verified checkpoint"
                    .to_string(),
            ));
        }
        Ok(())
    }
}

/// Issuer-verified reset-clear instruction bound to the exact signed pending
/// checkpoint head. Only this latest-head wrapper can be promoted after the
/// caller's durable full-scope local admission clear.
///
/// This capability is intentionally not `Clone`. It must be moved out of its
/// verified response and is consumed even when the durable-clear callback
/// fails; after a failure, fetch and verify current signed sync state again
/// before retrying. Process-wide replay safety remains enforced by the
/// service's single-head checkpoint compare-and-swap and operation-id
/// idempotency, because signed wire artifacts can always be deserialized and
/// verified again.
///
/// ```compile_fail
/// use opensecret::VerifiedLatestMapleResetClearRequiredV1;
///
/// fn duplicate_latest(latest: VerifiedLatestMapleResetClearRequiredV1) {
///     let _duplicate = latest.clone();
/// }
/// ```
pub struct VerifiedLatestMapleResetClearRequiredV1(MapleResetClearRequiredV1);

redacted_debug!(VerifiedLatestMapleResetClearRequiredV1);

impl VerifiedLatestMapleResetClearRequiredV1 {
    pub fn as_inner(&self) -> &MapleResetClearRequiredV1 {
        &self.0
    }

    /// Invokes the caller's durable admission-store transaction for the exact
    /// verified latest head and promotes it only when that transaction returns
    /// success. The callback must atomically remove every admission in the
    /// artifact's full account/project/host-installation scope and persist the
    /// target epoch/namespace before returning.
    ///
    /// This deliberately accepts neither a boolean nor a raw/deserialized
    /// receipt. The only SDK path to an ACK-ready wrapper passes the complete
    /// verified instruction through an explicit storage callback.
    pub fn after_durable_full_scope_admission_clear<F>(
        self,
        clear_and_commit: F,
    ) -> Result<DurablyClearedMapleResetClearRequiredV1>
    where
        F: FnOnce(&MapleResetClearRequiredV1) -> Result<()>,
    {
        clear_and_commit(&self.0)?;
        Ok(DurablyClearedMapleResetClearRequiredV1(self.0))
    }
}

pub struct DurablyClearedMapleResetClearRequiredV1(MapleResetClearRequiredV1);

redacted_debug!(DurablyClearedMapleResetClearRequiredV1);

impl DurablyClearedMapleResetClearRequiredV1 {
    pub fn as_inner(&self) -> &MapleResetClearRequiredV1 {
        &self.0
    }
}

#[derive(Clone)]
pub enum VerifiedMapleRevocationStreamEventV1 {
    PairRevocation(IssuerVerifiedMaplePairRevocationV1),
    ResetClearRequired(VerifiedMapleResetClearRequiredV1),
}

impl std::fmt::Debug for VerifiedMapleRevocationStreamEventV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let event_type = match self {
            Self::PairRevocation(_) => "pair_revocation",
            Self::ResetClearRequired(_) => "reset_clear_required",
        };
        formatter
            .debug_struct("VerifiedMapleRevocationStreamEventV1")
            .field("event_type", &event_type)
            .field("authority_material", &"[redacted]")
            .finish()
    }
}

impl VerifiedMapleRevocationStreamEventV1 {
    pub fn issuer_sequence(&self) -> u64 {
        match self {
            Self::PairRevocation(event) => event.as_inner().issuer_sequence,
            Self::ResetClearRequired(event) => event.as_inner().issuer_sequence,
        }
    }

    pub fn event_id(&self) -> Uuid {
        match self {
            Self::PairRevocation(event) => event.as_inner().event_id,
            Self::ResetClearRequired(event) => event.as_inner().event_id,
        }
    }

    pub fn event_digest(&self) -> Result<String> {
        match self {
            Self::PairRevocation(event) => event.as_inner().transcript_digest(),
            Self::ResetClearRequired(event) => event.event_digest(),
        }
    }
}

pub struct VerifiedMapleRevocationSyncV1 {
    sync: MapleRevocationSyncV1,
    checkpoint: VerifiedMapleRevocationStreamCheckpointV1,
    reset_clear_instruction: Option<VerifiedLatestMapleResetClearRequiredV1>,
}

redacted_debug!(VerifiedMapleRevocationSyncV1);

impl VerifiedMapleRevocationSyncV1 {
    pub fn as_inner(&self) -> &MapleRevocationSyncV1 {
        &self.sync
    }

    pub fn into_inner(self) -> MapleRevocationSyncV1 {
        self.sync
    }

    pub fn checkpoint(&self) -> &VerifiedMapleRevocationStreamCheckpointV1 {
        &self.checkpoint
    }

    pub fn has_reset_clear_instruction(&self) -> bool {
        self.reset_clear_instruction.is_some()
    }

    /// Consumes the verified sync state and extracts its actionable pending
    /// reset-clear capability, if present. A borrowed sync view never exposes
    /// that capability.
    pub fn into_reset_clear_instruction(self) -> Option<VerifiedLatestMapleResetClearRequiredV1> {
        self.reset_clear_instruction
    }
}

/// Registration receipt whose signed security synchronization state has been
/// verified against the caller's exact signed request and explicit issuer
/// trust anchors. Private fields prevent an unverified network DTO from being
/// mistaken for authority-bearing state.
pub struct VerifiedRegisterMapleDeviceResponseV1 {
    response: crate::types::MapleDeviceRegistrationResponse,
    revocation_sync: VerifiedMapleRevocationSyncV1,
}

redacted_debug!(VerifiedRegisterMapleDeviceResponseV1);

impl VerifiedRegisterMapleDeviceResponseV1 {
    pub(crate) fn new(
        response: crate::types::MapleDeviceRegistrationResponse,
        revocation_sync: VerifiedMapleRevocationSyncV1,
    ) -> Self {
        Self {
            response,
            revocation_sync,
        }
    }

    pub fn registration_id(&self) -> Uuid {
        self.response.registration_id
    }

    pub fn device_id(&self) -> Uuid {
        self.response.device_id
    }

    pub fn revision(&self) -> i64 {
        self.response.revision
    }

    pub fn accepted_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.response.accepted_at
    }

    pub fn security_epoch(&self) -> u64 {
        self.response.security_epoch
    }

    pub fn revocation_sync(&self) -> &VerifiedMapleRevocationSyncV1 {
        &self.revocation_sync
    }

    pub fn has_reset_clear_plan(&self) -> bool {
        self.revocation_sync.has_reset_clear_instruction()
    }

    /// Consumes the verified registration receipt and extracts its actionable
    /// pending reset-clear capability, if present.
    pub fn into_reset_clear_plan(self) -> Option<VerifiedLatestMapleResetClearRequiredV1> {
        self.revocation_sync.into_reset_clear_instruction()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MapleInstalledRevocationStreamNamespaceV1 {
    pub subject_account_id: Uuid,
    pub subject_project_id: Uuid,
    pub host_registration_id: Uuid,
    pub security_epoch: u64,
    pub revocation_stream_id: Uuid,
    pub revocation_stream_generation: u64,
}

redacted_debug!(MapleInstalledRevocationStreamNamespaceV1);

impl MapleInstalledRevocationStreamNamespaceV1 {
    pub fn new(
        subject_account_id: Uuid,
        subject_project_id: Uuid,
        host_registration_id: Uuid,
        security_epoch: u64,
        revocation_stream_id: Uuid,
        revocation_stream_generation: u64,
    ) -> Result<Self> {
        validate_non_nil(&[
            ("subject account ID", subject_account_id),
            ("subject project ID", subject_project_id),
            ("host registration ID", host_registration_id),
        ])?;
        validate_revocation_stream(
            revocation_stream_id,
            revocation_stream_generation,
            ErrorClass::Configuration,
        )?;
        validate_reset_counter(
            security_epoch,
            "installed security epoch",
            ErrorClass::Configuration,
        )?;
        Ok(Self {
            subject_account_id,
            subject_project_id,
            host_registration_id,
            security_epoch,
            revocation_stream_id,
            revocation_stream_generation,
        })
    }
}

#[derive(Clone)]
pub struct MapleRevocationStreamTransitionV1 {
    checkpoint: MapleRevocationStreamCheckpointV1,
    requires_admission_reset: bool,
}

redacted_debug!(MapleRevocationStreamTransitionV1);

impl MapleRevocationStreamTransitionV1 {
    pub fn checkpoint(&self) -> &MapleRevocationStreamCheckpointV1 {
        &self.checkpoint
    }

    pub fn requires_admission_reset(&self) -> bool {
        self.requires_admission_reset
    }

    /// Promotes the checkpoint after the caller durably commits this plan.
    ///
    /// When [`Self::requires_admission_reset`] is true, the transaction MUST
    /// clear every prior admission for this account and host, install the new
    /// namespace, and reset its cursor to zero before this call. An unchanged
    /// namespace must still preserve monotonic durable checkpoint/cursor state.
    /// The SDK deliberately performs no storage and cannot auto-ack this step.
    pub fn after_durable_commit(self) -> DurablyReconciledMapleRevocationStreamV1 {
        DurablyReconciledMapleRevocationStreamV1(self.checkpoint)
    }
}

/// A signed host-account revocation namespace that the caller explicitly
/// attests it has reconciled with durable local admission state.
#[derive(Clone)]
pub struct DurablyReconciledMapleRevocationStreamV1(MapleRevocationStreamCheckpointV1);

redacted_debug!(DurablyReconciledMapleRevocationStreamV1);

impl DurablyReconciledMapleRevocationStreamV1 {
    pub fn as_inner(&self) -> &MapleRevocationStreamCheckpointV1 {
        &self.0
    }

    fn validate_authorization(&self, authorization: &MaplePairAuthorizationV1) -> Result<()> {
        if self.0.subject_account_id != authorization.subject_account_id
            || self.0.subject_project_id != authorization.subject_project_id
            || !device_claim_is_same_identity_at_or_before(&authorization.host, &self.0.host)
            || self.0.revocation_stream_id != authorization.revocation_stream_id
            || self.0.revocation_stream_generation != authorization.revocation_stream_generation
        {
            return Err(Error::InvalidResponse(
                "Maple pair authorization does not match the durably reconciled revocation stream"
                    .to_string(),
            ));
        }
        Ok(())
    }

    fn validate_revocation(&self, revocation: &MaplePairRevocationV1) -> Result<()> {
        if self.0.subject_account_id != revocation.subject_account_id
            || self.0.subject_project_id != revocation.subject_project_id
            || !device_claim_is_same_identity_at_or_before(&revocation.host, &self.0.host)
            || self.0.revocation_stream_id != revocation.revocation_stream_id
            || self.0.revocation_stream_generation != revocation.revocation_stream_generation
        {
            return Err(Error::InvalidResponse(
                "Maple pair revocation does not match the durably reconciled revocation stream"
                    .to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct VerifiedMaplePairAuthorizationV1(MaplePairAuthorizationV1);

redacted_debug!(VerifiedMaplePairAuthorizationV1);

impl VerifiedMaplePairAuthorizationV1 {
    pub fn as_inner(&self) -> &MaplePairAuthorizationV1 {
        &self.0
    }

    pub fn into_inner(self) -> MaplePairAuthorizationV1 {
        self.0
    }

    pub fn transcript_digest(&self) -> Result<String> {
        self.0.transcript_digest()
    }
}

/// An awaiting-host-commit authorization whose status revision and host role
/// were verified. This is the only input accepted by the manual confirm
/// constructor, preventing active or revoked historical artifacts from being
/// reinstalled and reconfirmed.
#[derive(Clone)]
pub struct HostCommitReadyMaplePairAuthorizationV1 {
    authorization: VerifiedMaplePairAuthorizationV1,
    expected_pairing_revision: i64,
}

redacted_debug!(HostCommitReadyMaplePairAuthorizationV1);

impl HostCommitReadyMaplePairAuthorizationV1 {
    pub fn authorization(&self) -> &VerifiedMaplePairAuthorizationV1 {
        &self.authorization
    }

    pub fn expected_pairing_revision(&self) -> i64 {
        self.expected_pairing_revision
    }
}

#[derive(Clone)]
pub struct IssuerVerifiedMaplePairRevocationV1(MaplePairRevocationV1);

redacted_debug!(IssuerVerifiedMaplePairRevocationV1);

impl IssuerVerifiedMaplePairRevocationV1 {
    pub fn as_inner(&self) -> &MaplePairRevocationV1 {
        &self.0
    }

    pub fn into_inner(self) -> MaplePairRevocationV1 {
        self.0
    }

    /// Binds an issuer-verified event to the host's ticket-bound durable pair
    /// authorization. Only the resulting wrapper may construct an ACK.
    pub fn bind_to_authorization(
        &self,
        authorization: &VerifiedMaplePairAuthorizationV1,
        revocation_stream: &DurablyReconciledMapleRevocationStreamV1,
    ) -> Result<VerifiedMaplePairRevocationV1> {
        revocation_stream.validate_authorization(authorization.as_inner())?;
        revocation_stream.validate_revocation(&self.0)?;
        self.0
            .validate_authorization_binding(authorization.as_inner())?;
        Ok(VerifiedMaplePairRevocationV1(self.0.clone()))
    }
}

#[derive(Clone)]
pub struct VerifiedMaplePairRevocationV1(MaplePairRevocationV1);

redacted_debug!(VerifiedMaplePairRevocationV1);

impl VerifiedMaplePairRevocationV1 {
    pub fn as_inner(&self) -> &MaplePairRevocationV1 {
        &self.0
    }

    pub fn into_inner(self) -> MaplePairRevocationV1 {
        self.0
    }

    pub fn transcript_digest(&self) -> Result<String> {
        self.0.transcript_digest()
    }
}

#[derive(Clone)]
pub struct VerifiedMaplePairingStatusV1 {
    status: MaplePairingStatusV1,
}

redacted_debug!(VerifiedMaplePairingStatusV1);

impl VerifiedMaplePairingStatusV1 {
    pub fn as_inner(&self) -> &MaplePairingStatusV1 {
        &self.status
    }

    pub fn into_inner(self) -> MaplePairingStatusV1 {
        self.status
    }

    /// Returns the authorization only after the enclosing status verifier has
    /// bound it to the status's issuer-verified request ticket.
    pub fn pair_authorization(&self) -> Option<VerifiedMaplePairAuthorizationV1> {
        self.status.pair_authorization.clone().map(|authorization| {
            // Construction of `Self` already verified and bound this artifact
            // to the status ticket; the opaque wrapper cannot be forged by an
            // SDK caller.
            VerifiedMaplePairAuthorizationV1(authorization)
        })
    }

    /// Returns a host-commit-ready authorization only for a verified host view
    /// in `awaiting_host_commit` state whose namespace matches the host's
    /// durably reconciled revocation stream.
    pub fn host_commit_ready_authorization(
        &self,
        revocation_stream: &DurablyReconciledMapleRevocationStreamV1,
    ) -> Option<HostCommitReadyMaplePairAuthorizationV1> {
        if self.status.state != MaplePairingState::AwaitingHostCommit || self.status.revision != 2 {
            return None;
        }
        self.pair_authorization().and_then(|authorization| {
            revocation_stream
                .validate_authorization(authorization.as_inner())
                .ok()?;
            Some(HostCommitReadyMaplePairAuthorizationV1 {
                authorization,
                expected_pairing_revision: self.status.revision,
            })
        })
    }
}

impl MaplePairingStatusV1 {
    pub(crate) fn verify(
        &self,
        issuers: &MaplePairingIssuerKeySet,
        expected_role: MaplePairingRole,
        trusted_now_unix_ms: i64,
    ) -> Result<VerifiedMaplePairingStatusV1> {
        if trusted_now_unix_ms <= 0 {
            return Err(Error::Configuration(
                "Maple pairing status verification requires a positive trusted current time"
                    .to_string(),
            ));
        }
        if self.pairing_request_id.is_nil()
            || self.pair_id.is_nil()
            || self.revision <= 0
            || self.pairing_incarnation == 0
            || self.pairing_incarnation > i64::MAX as u64
            || self.execution_target_id.is_nil()
            || self.controller_registration_id.is_nil()
            || self.host_registration_id.is_nil()
            || self.execution_target_id != self.host_registration_id
        {
            return Err(Error::InvalidResponse(
                "Maple pairing status contains invalid identity, revision, or incarnation fields"
                    .to_string(),
            ));
        }
        self.validate_revocation_stream_shape()?;
        validate_timestamp(
            self.created_at_unix_ms,
            "created timestamp",
            ErrorClass::InvalidResponse,
        )?;
        validate_timestamp(
            self.expires_at_unix_ms,
            "expiry timestamp",
            ErrorClass::InvalidResponse,
        )?;
        if self.expires_at_unix_ms <= self.created_at_unix_ms {
            return Err(Error::InvalidResponse(
                "Maple pairing status expiry must follow creation".to_string(),
            ));
        }

        let ticket = self.request_ticket.as_ref().ok_or_else(|| {
            Error::InvalidResponse(
                "Maple pairing participant status omitted its request ticket".to_string(),
            )
        })?;
        match self.state {
            MaplePairingState::Pending => {
                ticket.verify_at(
                    issuers,
                    trusted_now_unix_ms,
                    MAPLE_PAIRING_MAX_CLOCK_SKEW_MS,
                    MAPLE_PAIRING_MAX_TICKET_TTL_MS,
                )?;
            }
            MaplePairingState::Expired => {
                ticket.verify_signatures(issuers)?;
                if trusted_now_unix_ms
                    .checked_add(MAPLE_PAIRING_MAX_CLOCK_SKEW_MS)
                    .is_none_or(|latest| latest < ticket.expires_at_unix_ms)
                {
                    return Err(Error::InvalidResponse(
                        "Maple pairing was labeled expired before its signed expiry window"
                            .to_string(),
                    ));
                }
            }
            MaplePairingState::AwaitingHostCommit
            | MaplePairingState::Active
            | MaplePairingState::Revoked => ticket.verify_signatures(issuers)?,
        }
        self.validate_ticket_binding(ticket)?;

        match self.state {
            MaplePairingState::Pending | MaplePairingState::Expired => {
                let expected_revision = if self.state == MaplePairingState::Pending {
                    1
                } else {
                    2
                };
                if self.revision != expected_revision {
                    return Err(Error::InvalidResponse(
                        "Maple pending or expired pairing status has an invalid state revision"
                            .to_string(),
                    ));
                }
                if self.pair_authorization.is_some()
                    || self.revocation.is_some()
                    || self.approved_at_unix_ms.is_some()
                    || self.activated_at_unix_ms.is_some()
                    || self.revoked_at_unix_ms.is_some()
                {
                    return Err(Error::InvalidResponse(
                        "Maple pending or expired pairing status contains later-state artifacts"
                            .to_string(),
                    ));
                }
            }
            MaplePairingState::AwaitingHostCommit => {
                if self.revision != 2
                    || self.approved_at_unix_ms.is_none()
                    || self.activated_at_unix_ms.is_some()
                    || self.revocation.is_some()
                    || self.revoked_at_unix_ms.is_some()
                {
                    return Err(Error::InvalidResponse(
                        "Maple awaiting-host-commit status has inconsistent lifecycle fields"
                            .to_string(),
                    ));
                }
                match expected_role {
                    MaplePairingRole::Host => self.verify_authorization(issuers)?,
                    MaplePairingRole::Controller if self.pair_authorization.is_none() => {}
                    MaplePairingRole::Controller => {
                        return Err(Error::InvalidResponse(
                            "Maple controller status exposed a pre-commit pair authorization"
                                .to_string(),
                        ));
                    }
                }
                if self
                    .approved_at_unix_ms
                    .is_some_and(|approved| approved < self.created_at_unix_ms)
                {
                    return Err(Error::InvalidResponse(
                        "Maple pairing approval precedes the signed request creation".to_string(),
                    ));
                }
            }
            MaplePairingState::Active => {
                self.verify_authorization(issuers)?;
                if self.revision != 3
                    || self.approved_at_unix_ms.is_none()
                    || self.activated_at_unix_ms.is_none()
                    || self.revocation.is_some()
                    || self.revoked_at_unix_ms.is_some()
                {
                    return Err(Error::InvalidResponse(
                        "Maple active pairing status has inconsistent lifecycle fields".to_string(),
                    ));
                }
                if self
                    .approved_at_unix_ms
                    .zip(self.activated_at_unix_ms)
                    .is_none_or(|(approved, activated)| {
                        approved < self.created_at_unix_ms || activated < approved
                    })
                {
                    return Err(Error::InvalidResponse(
                        "Maple pairing active lifecycle timestamps are out of order".to_string(),
                    ));
                }
            }
            MaplePairingState::Revoked => {
                let revocation = self.revocation.as_ref().ok_or_else(|| {
                    Error::InvalidResponse(
                        "Maple revoked status omitted its revocation artifact".to_string(),
                    )
                })?;
                revocation.verify_issuer(issuers)?;
                let valid_revision = match (self.activated_at_unix_ms, expected_role) {
                    (Some(_), _) => {
                        self.verify_authorization(issuers)?;
                        self.validate_revocation_binding(revocation)?;
                        self.revision == 4
                    }
                    (None, MaplePairingRole::Host) => {
                        self.verify_authorization(issuers)?;
                        self.validate_revocation_binding(revocation)?;
                        self.revision == 3
                    }
                    (None, MaplePairingRole::Controller) if self.pair_authorization.is_none() => {
                        self.validate_redacted_revocation_binding(ticket, revocation)?;
                        self.revision == 3
                    }
                    (None, MaplePairingRole::Controller) => {
                        return Err(Error::InvalidResponse(
                            "Maple controller status exposed a pre-commit pair authorization"
                                .to_string(),
                        ));
                    }
                };
                if !valid_revision
                    || self.approved_at_unix_ms.is_none()
                    || self.revoked_at_unix_ms.is_none()
                {
                    return Err(Error::InvalidResponse(
                        "Maple revoked pairing status omitted lifecycle timestamps".to_string(),
                    ));
                }
                let approved = self.approved_at_unix_ms.expect("checked above");
                let revoked = self.revoked_at_unix_ms.expect("checked above");
                if approved < self.created_at_unix_ms
                    || revoked < approved
                    || self
                        .activated_at_unix_ms
                        .is_some_and(|activated| activated < approved || revoked < activated)
                {
                    return Err(Error::InvalidResponse(
                        "Maple pairing revoked lifecycle timestamps are out of order".to_string(),
                    ));
                }
            }
        }
        Ok(VerifiedMaplePairingStatusV1 {
            status: self.clone(),
        })
    }

    fn validate_ticket_binding(&self, ticket: &MaplePairRequestTicketV1) -> Result<()> {
        if ticket.pairing_request_id != self.pairing_request_id
            || ticket.pair_id != self.pair_id
            || ticket.pairing_incarnation != self.pairing_incarnation
            || ticket.direction != self.direction
            || ticket.execution_target_id != self.execution_target_id
            || ticket.controller.registration_id != self.controller_registration_id
            || ticket.host.registration_id != self.host_registration_id
            || ticket.created_at_unix_ms != self.created_at_unix_ms
            || ticket.expires_at_unix_ms != self.expires_at_unix_ms
        {
            return Err(Error::InvalidResponse(
                "Maple pairing status does not match its signed request ticket".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_revocation_stream_shape(&self) -> Result<()> {
        let namespace = match (self.revocation_stream_id, self.revocation_stream_generation) {
            (Some(stream_id), Some(generation)) => {
                validate_revocation_stream(stream_id, generation, ErrorClass::InvalidResponse)?;
                Some((stream_id, generation))
            }
            (None, None) => None,
            _ => {
                return Err(Error::InvalidResponse(
                    "Maple pairing status contains a partial revocation namespace".to_string(),
                ));
            }
        };
        let required = matches!(
            self.state,
            MaplePairingState::AwaitingHostCommit
                | MaplePairingState::Active
                | MaplePairingState::Revoked
        );
        if namespace.is_some() != required {
            return Err(Error::InvalidResponse(
                "Maple pairing status revocation namespace does not match its lifecycle state"
                    .to_string(),
            ));
        }
        if let (Some((stream_id, generation)), Some(authorization)) =
            (namespace, self.pair_authorization.as_ref())
        {
            if authorization.revocation_stream_id != stream_id
                || authorization.revocation_stream_generation != generation
            {
                return Err(Error::InvalidResponse(
                    "Maple pairing status namespace does not match its authorization".to_string(),
                ));
            }
        }
        if let (Some((stream_id, generation)), Some(revocation)) =
            (namespace, self.revocation.as_ref())
        {
            if revocation.revocation_stream_id != stream_id
                || revocation.revocation_stream_generation != generation
            {
                return Err(Error::InvalidResponse(
                    "Maple pairing status namespace does not match its revocation".to_string(),
                ));
            }
        }
        Ok(())
    }

    /// Validates a controller-visible precommit revocation after the service
    /// has intentionally redacted the not-yet-active pair authorization.
    ///
    /// The issuer-signed authorization digest remains an opaque lineage
    /// commitment here; this path cannot produce pairing or ACK authority.
    fn validate_redacted_revocation_binding(
        &self,
        ticket: &MaplePairRequestTicketV1,
        revocation: &MaplePairRevocationV1,
    ) -> Result<()> {
        revocation.authorization_digest_bytes()?;
        if revocation.subject_account_id != ticket.subject_account_id
            || revocation.subject_project_id != ticket.subject_project_id
            || revocation.recipient_host_registration_id != self.host_registration_id
            || revocation.pairing_request_id != self.pairing_request_id
            || revocation.pairing_request_id != ticket.pairing_request_id
            || revocation.pair_id != self.pair_id
            || revocation.pair_id != ticket.pair_id
            || revocation.pairing_incarnation != self.pairing_incarnation
            || revocation.pairing_incarnation != ticket.pairing_incarnation
            || revocation.direction != self.direction
            || revocation.direction != ticket.direction
            || revocation.execution_target_id != self.execution_target_id
            || revocation.execution_target_id != ticket.execution_target_id
            || revocation.controller != ticket.controller
            || revocation.host != ticket.host
            || Some(revocation.revocation_stream_id) != self.revocation_stream_id
            || Some(revocation.revocation_stream_generation) != self.revocation_stream_generation
            || Some(revocation.revoked_at_unix_ms) != self.revoked_at_unix_ms
        {
            return Err(Error::InvalidResponse(
                "Maple controller pre-commit revocation does not match its signed request ticket"
                    .to_string(),
            ));
        }
        Ok(())
    }

    fn verify_authorization(&self, issuers: &MaplePairingIssuerKeySet) -> Result<()> {
        let authorization = self.pair_authorization.as_ref().ok_or_else(|| {
            Error::InvalidResponse(
                "Maple pairing status omitted its pair authorization".to_string(),
            )
        })?;
        let ticket = self.request_ticket.as_ref().ok_or_else(|| {
            Error::InvalidResponse("Missing Maple pairing request ticket".to_string())
        })?;
        authorization.verify_against_ticket(issuers, ticket)?;
        if authorization.pairing_request_id != self.pairing_request_id
            || authorization.pair_id != self.pair_id
            || authorization.pairing_incarnation != self.pairing_incarnation
            || authorization.direction != self.direction
            || authorization.execution_target_id != self.execution_target_id
            || Some(authorization.revocation_stream_id) != self.revocation_stream_id
            || Some(authorization.revocation_stream_generation) != self.revocation_stream_generation
            || Some(authorization.approved_at_unix_ms) != self.approved_at_unix_ms
        {
            return Err(Error::InvalidResponse(
                "Maple pairing status does not match its signed pair authorization".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_revocation_binding(&self, revocation: &MaplePairRevocationV1) -> Result<()> {
        let authorization = self
            .pair_authorization
            .as_ref()
            .ok_or_else(|| Error::InvalidResponse("Missing pair authorization".to_string()))?;
        let authorization_digest = authorization.transcript_digest()?;
        if revocation.subject_account_id != authorization.subject_account_id
            || revocation.subject_project_id != authorization.subject_project_id
            || revocation.recipient_host_registration_id != self.host_registration_id
            || revocation.pairing_request_id != self.pairing_request_id
            || revocation.pair_id != self.pair_id
            || revocation.pairing_incarnation != self.pairing_incarnation
            || revocation.direction != self.direction
            || revocation.execution_target_id != self.execution_target_id
            || revocation.controller != authorization.controller
            || revocation.host != authorization.host
            || revocation.revocation_stream_id != authorization.revocation_stream_id
            || revocation.revocation_stream_generation != authorization.revocation_stream_generation
            || revocation.pair_authorization_digest != authorization_digest
            || Some(revocation.revoked_at_unix_ms) != self.revoked_at_unix_ms
        {
            return Err(Error::InvalidResponse(
                "Maple pairing status does not match its signed revocation".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct VerifiedMaplePairingMutationResponse {
    pub protocol_version: u16,
    pub operation_id: Uuid,
    pub pairing: VerifiedMaplePairingStatusV1,
}

redacted_debug!(VerifiedMaplePairingMutationResponse);

#[derive(Clone)]
pub struct VerifiedMaplePairingListResponse {
    pub protocol_version: u16,
    pub query_id: Uuid,
    pub role: MaplePairingRole,
    pub pairings: Vec<VerifiedMaplePairingStatusV1>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

redacted_debug!(VerifiedMaplePairingListResponse);

#[derive(Clone)]
pub struct VerifiedMaplePairingStatusResponse {
    pub protocol_version: u16,
    pub query_id: Uuid,
    pub pairing: VerifiedMaplePairingStatusV1,
}

redacted_debug!(VerifiedMaplePairingStatusResponse);

pub struct VerifiedMaplePairingRevocationListResponse {
    pub protocol_version: u16,
    pub query_id: Uuid,
    /// Signed current security epoch, status, namespace checkpoint, and any
    /// pending latest reset-clear head.
    pub revocation_sync: VerifiedMapleRevocationSyncV1,
    /// Events whose issuer signatures and page sequence are verified. Each
    /// pair revocation must still be bound to the host's local ticket-bound
    /// authorization before durable apply; reset-clear ACK construction
    /// additionally requires the durable full-scope-clear wrapper.
    pub events: Vec<VerifiedMapleRevocationStreamEventV1>,
    pub next_after_issuer_sequence: u64,
    pub has_more: bool,
}

impl VerifiedMaplePairingRevocationListResponse {
    pub fn has_reset_clear_instruction(&self) -> bool {
        self.revocation_sync.has_reset_clear_instruction()
    }

    /// Consumes the verified page and extracts its actionable pending
    /// reset-clear capability, if present. Historical reset events remain in
    /// `events` only as read-only verified history and cannot use this path.
    pub fn into_reset_clear_instruction(self) -> Option<VerifiedLatestMapleResetClearRequiredV1> {
        self.revocation_sync.into_reset_clear_instruction()
    }
}

impl std::fmt::Debug for VerifiedMaplePairingRevocationListResponse {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("VerifiedMaplePairingRevocationListResponse")
            .field("protocol_version", &self.protocol_version)
            .field("event_count", &self.events.len())
            .field("has_more", &self.has_more)
            .field("authority_material", &"[redacted]")
            .finish()
    }
}

/// Verified operation receipt for one exact revocation acknowledgement.
///
/// This proves that the submitted operation was accepted with the returned
/// event/cursor values. It deliberately does not expose its signed checkpoint
/// as [`VerifiedMapleRevocationStreamCheckpointV1`]: an idempotent replay may
/// return a historical receipt after current state has advanced. Fetch and
/// verify a fresh registration or revocation-list sync before deciding
/// readiness or installing a current durable namespace.
///
/// ```compile_fail
/// use opensecret::VerifiedMaplePairingRevocationAckResponse;
///
/// fn install_receipt_as_current(receipt: VerifiedMaplePairingRevocationAckResponse) {
///     let _transition = receipt.stream_checkpoint.plan_durable_transition(None);
/// }
/// ```
#[derive(Clone)]
pub struct VerifiedMaplePairingRevocationAckResponse {
    protocol_version: u16,
    operation_id: Uuid,
    host_registration_id: Uuid,
    event_id: Uuid,
    issuer_sequence: u64,
    last_acked_issuer_sequence: u64,
    accepted_at_unix_ms: i64,
}

impl VerifiedMaplePairingRevocationAckResponse {
    pub fn protocol_version(&self) -> u16 {
        self.protocol_version
    }

    pub fn operation_id(&self) -> Uuid {
        self.operation_id
    }

    pub fn host_registration_id(&self) -> Uuid {
        self.host_registration_id
    }

    pub fn event_id(&self) -> Uuid {
        self.event_id
    }

    pub fn issuer_sequence(&self) -> u64 {
        self.issuer_sequence
    }

    pub fn last_acked_issuer_sequence(&self) -> u64 {
        self.last_acked_issuer_sequence
    }

    pub fn accepted_at_unix_ms(&self) -> i64 {
        self.accepted_at_unix_ms
    }
}

impl std::fmt::Debug for VerifiedMaplePairingRevocationAckResponse {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("VerifiedMaplePairingRevocationAckResponse")
            .field("protocol_version", &self.protocol_version)
            .field("issuer_sequence", &self.issuer_sequence)
            .field(
                "last_acked_issuer_sequence",
                &self.last_acked_issuer_sequence,
            )
            .field("authority_material", &"[redacted]")
            .finish()
    }
}

impl MaplePairingMutationResponse {
    pub(crate) fn verify_create(
        self,
        request: &CreateMaplePairingRequest,
        issuers: &MaplePairingIssuerKeySet,
        trusted_now_unix_ms: i64,
    ) -> Result<VerifiedMaplePairingMutationResponse> {
        self.verify_common(
            request.operation_id,
            MaplePairingState::Pending,
            MaplePairingRole::Controller,
            issuers,
            trusted_now_unix_ms,
        )?;
        let pairing = &self.pairing;
        let ticket = pairing.request_ticket.as_ref().ok_or_else(|| {
            Error::InvalidResponse(
                "Maple pairing create receipt omitted request ticket".to_string(),
            )
        })?;
        if ticket.subject_account_id != request.asserted_account_id
            || ticket.subject_project_id != request.asserted_project_id
            || ticket.controller_request_operation_id != request.operation_id
            || ticket.controller.registration_id != request.controller_registration_id
            || ticket.controller.device_id != request.controller_device_id
            || ticket.controller.installation_id != request.controller_installation_id
            || ticket.controller.endpoint_id != request.controller_endpoint_id
            || ticket.controller.endpoint_epoch != request.controller_endpoint_epoch
            || ticket.host.registration_id != request.host_registration_id
            || ticket.host.device_id != request.host_device_id
            || ticket.host.installation_id != request.host_installation_id
            || ticket.host.endpoint_id != request.host_endpoint_id
            || ticket.host.endpoint_epoch != request.host_endpoint_epoch
            || ticket.pairing_request_nonce != request.pairing_request_nonce
            || ticket.controller_request_digest != request.transcript_digest()?
            || ticket.controller_request_signature != request.signature
            || ticket.protocol_min != request.protocol_min
            || ticket.protocol_max != request.protocol_max
            || pairing.controller_registration_id != request.controller_registration_id
            || pairing.host_registration_id != request.host_registration_id
            || pairing.execution_target_id != request.execution_target_id
            || pairing.direction != request.direction
        {
            return Err(Error::InvalidResponse(
                "Maple pairing create receipt does not match its submitted device claims"
                    .to_string(),
            ));
        }
        self.into_verified(MaplePairingRole::Controller, issuers, trusted_now_unix_ms)
    }

    pub(crate) fn verify_approve(
        self,
        request: &ApproveMaplePairingRequest,
        issuers: &MaplePairingIssuerKeySet,
        trusted_now_unix_ms: i64,
    ) -> Result<VerifiedMaplePairingMutationResponse> {
        self.verify_mutation_binding(
            MutationExpectation {
                operation_id: request.operation_id,
                pairing_request_id: request.pairing_request_id,
                pair_id: request.pair_id,
                expected_revision: request.expected_pairing_revision,
                pairing_incarnation: request.pairing_incarnation,
                state: MaplePairingState::AwaitingHostCommit,
                role: MaplePairingRole::Host,
            },
            issuers,
            trusted_now_unix_ms,
        )?;
        let authorization = self.pairing.pair_authorization.as_ref().ok_or_else(|| {
            Error::InvalidResponse("Maple approval receipt omitted pair authorization".to_string())
        })?;
        if authorization.request_ticket_digest != request.request_ticket_digest
            || authorization.subject_account_id != request.asserted_account_id
            || authorization.subject_project_id != request.asserted_project_id
            || authorization.host.registration_id != request.host_registration_id
            || authorization.pairing_request_id != request.pairing_request_id
            || authorization.pair_id != request.pair_id
            || authorization.pairing_incarnation != request.pairing_incarnation
            || authorization.revocation_stream_id != request.revocation_stream_id
            || authorization.revocation_stream_generation != request.revocation_stream_generation
            || authorization.host_approval_operation_id != request.operation_id
            || authorization.host_approval_expected_pairing_revision
                != request.expected_pairing_revision
            || authorization.host_approval_nonce != request.host_approval_nonce
            || authorization.protocol_min != request.approved_protocol_min
            || authorization.protocol_max != request.approved_protocol_max
            || authorization.host_approval_digest != request.transcript_digest()?
            || authorization.host_approval_signature != request.signature
        {
            return Err(Error::InvalidResponse(
                "Maple approval receipt does not bind the submitted host approval".to_string(),
            ));
        }
        self.into_verified(MaplePairingRole::Host, issuers, trusted_now_unix_ms)
    }

    pub(crate) fn verify_confirm(
        self,
        request: &ConfirmMaplePairingRequest,
        issuers: &MaplePairingIssuerKeySet,
        trusted_now_unix_ms: i64,
    ) -> Result<VerifiedMaplePairingMutationResponse> {
        self.verify_mutation_binding(
            MutationExpectation {
                operation_id: request.operation_id,
                pairing_request_id: request.pairing_request_id,
                pair_id: request.pair_id,
                expected_revision: request.expected_pairing_revision,
                pairing_incarnation: request.pairing_incarnation,
                state: MaplePairingState::Active,
                role: MaplePairingRole::Host,
            },
            issuers,
            trusted_now_unix_ms,
        )?;
        let authorization = self.pairing.pair_authorization.as_ref().ok_or_else(|| {
            Error::InvalidResponse("Maple confirmation receipt omitted authorization".to_string())
        })?;
        if authorization.subject_account_id != request.asserted_account_id
            || authorization.subject_project_id != request.asserted_project_id
            || authorization.host.registration_id != request.host_registration_id
            || authorization.transcript_digest()?.as_str()
                != request.pair_authorization_digest.as_str()
        {
            return Err(Error::InvalidResponse(
                "Maple confirmation receipt does not bind the submitted pair authorization"
                    .to_string(),
            ));
        }
        self.into_verified(MaplePairingRole::Host, issuers, trusted_now_unix_ms)
    }

    pub(crate) fn verify_revoke(
        self,
        request: &RevokeMaplePairingRequest,
        issuers: &MaplePairingIssuerKeySet,
        trusted_now_unix_ms: i64,
    ) -> Result<VerifiedMaplePairingMutationResponse> {
        self.verify_mutation_binding(
            MutationExpectation {
                operation_id: request.operation_id,
                pairing_request_id: request.pairing_request_id,
                pair_id: request.pair_id,
                expected_revision: request.expected_pairing_revision,
                pairing_incarnation: request.pairing_incarnation,
                state: MaplePairingState::Revoked,
                role: request.actor_role,
            },
            issuers,
            trusted_now_unix_ms,
        )?;
        let revocation = self.pairing.revocation.as_ref().ok_or_else(|| {
            Error::InvalidResponse(
                "Maple revocation receipt omitted revocation artifact".to_string(),
            )
        })?;
        if revocation.revoked_by_registration_id != request.actor_registration_id
            || revocation.revoked_by_role != request.actor_role
            || revocation.reason_code != request.reason_code
            || revocation.subject_account_id != request.asserted_account_id
            || revocation.subject_project_id != request.asserted_project_id
            || revocation.revocation_stream_id != request.revocation_stream_id
            || revocation.revocation_stream_generation != request.revocation_stream_generation
        {
            return Err(Error::InvalidResponse(
                "Maple revocation receipt does not bind the submitted revocation".to_string(),
            ));
        }
        self.into_verified(request.actor_role, issuers, trusted_now_unix_ms)
    }

    fn verify_mutation_binding(
        &self,
        expected: MutationExpectation,
        issuers: &MaplePairingIssuerKeySet,
        trusted_now_unix_ms: i64,
    ) -> Result<()> {
        self.verify_common(
            expected.operation_id,
            expected.state,
            expected.role,
            issuers,
            trusted_now_unix_ms,
        )?;
        if self.pairing.pairing_request_id != expected.pairing_request_id
            || self.pairing.pair_id != expected.pair_id
            || self.pairing.revision != expected.expected_revision + 1
            || self.pairing.pairing_incarnation != expected.pairing_incarnation
        {
            return Err(Error::InvalidResponse(
                "Maple pairing mutation receipt does not match its request".to_string(),
            ));
        }
        Ok(())
    }

    fn verify_common(
        &self,
        operation_id: Uuid,
        state: MaplePairingState,
        role: MaplePairingRole,
        issuers: &MaplePairingIssuerKeySet,
        trusted_now_unix_ms: i64,
    ) -> Result<()> {
        if self.protocol_version != MAPLE_PAIRING_PROTOCOL_VERSION
            || self.operation_id != operation_id
            || self.pairing.state != state
        {
            return Err(Error::InvalidResponse(
                "Maple pairing mutation receipt has an unsupported version or mismatched operation/state"
                    .to_string(),
            ));
        }
        self.pairing.verify(issuers, role, trusted_now_unix_ms)?;
        Ok(())
    }

    fn into_verified(
        self,
        role: MaplePairingRole,
        issuers: &MaplePairingIssuerKeySet,
        trusted_now_unix_ms: i64,
    ) -> Result<VerifiedMaplePairingMutationResponse> {
        Ok(VerifiedMaplePairingMutationResponse {
            protocol_version: self.protocol_version,
            operation_id: self.operation_id,
            pairing: self.pairing.verify(issuers, role, trusted_now_unix_ms)?,
        })
    }
}

#[derive(Clone, Copy)]
struct MutationExpectation {
    operation_id: Uuid,
    pairing_request_id: Uuid,
    pair_id: Uuid,
    expected_revision: i64,
    pairing_incarnation: u64,
    state: MaplePairingState,
    role: MaplePairingRole,
}

impl MaplePairingListResponse {
    pub(crate) fn verify_for(
        self,
        request: &ListMaplePairingsRequest,
        issuers: &MaplePairingIssuerKeySet,
        trusted_now_unix_ms: i64,
    ) -> Result<VerifiedMaplePairingListResponse> {
        let effective_limit = validate_limit(request.limit)?;
        let cursor_consistent = self.has_more == self.next_cursor.is_some();
        let cursor_valid = self.next_cursor.as_ref().is_none_or(|cursor| {
            !cursor.is_empty()
                && cursor.len() <= MAPLE_PAIRING_CURSOR_MAX_BYTES
                && cursor.is_ascii()
                && Some(cursor.as_str()) != request.cursor.as_deref()
        });
        if self.protocol_version != MAPLE_PAIRING_PROTOCOL_VERSION
            || self.query_id != request.query_id
            || self.role != request.role
            || self.pairings.len() > usize::from(effective_limit)
            || !cursor_consistent
            || !cursor_valid
            || (self.has_more && self.pairings.is_empty())
        {
            return Err(Error::InvalidResponse(
                "Maple pairing list response is unsupported or internally inconsistent".to_string(),
            ));
        }
        let requested_states: HashSet<_> = request.states.iter().copied().collect();
        let mut pair_ids = HashSet::with_capacity(self.pairings.len());
        let mut request_ids = HashSet::with_capacity(self.pairings.len());
        let mut verified = Vec::with_capacity(self.pairings.len());
        let mut previous_pair_id: Option<Uuid> = None;
        for pairing in self.pairings {
            let ticket = pairing.request_ticket.as_ref().ok_or_else(|| {
                Error::InvalidResponse(
                    "Maple pairing list item omitted its signed request ticket".to_string(),
                )
            })?;
            let belongs_to_actor = match request.role {
                MaplePairingRole::Controller => {
                    pairing.controller_registration_id == request.actor_registration_id
                }
                MaplePairingRole::Host => {
                    pairing.host_registration_id == request.actor_registration_id
                }
            };
            let ordered = previous_pair_id.is_none_or(|previous| previous > pairing.pair_id);
            if !belongs_to_actor
                || ticket.subject_account_id != request.asserted_account_id
                || ticket.subject_project_id != request.asserted_project_id
                || !ordered
                || !requested_states.contains(&pairing.state)
                || !pair_ids.insert(pairing.pair_id)
                || !request_ids.insert(pairing.pairing_request_id)
            {
                return Err(Error::InvalidResponse(
                    "Maple pairing list response contains an unrequested state or duplicate identity"
                        .to_string(),
                ));
            }
            previous_pair_id = Some(pairing.pair_id);
            verified.push(pairing.verify(issuers, request.role, trusted_now_unix_ms)?);
        }
        Ok(VerifiedMaplePairingListResponse {
            protocol_version: self.protocol_version,
            query_id: self.query_id,
            role: self.role,
            pairings: verified,
            next_cursor: self.next_cursor,
            has_more: self.has_more,
        })
    }
}

impl MaplePairingStatusResponse {
    pub(crate) fn verify_for(
        self,
        request: &MaplePairingStatusRequest,
        actor_role: MaplePairingRole,
        issuers: &MaplePairingIssuerKeySet,
        trusted_now_unix_ms: i64,
    ) -> Result<VerifiedMaplePairingStatusResponse> {
        if self.protocol_version != MAPLE_PAIRING_PROTOCOL_VERSION
            || self.query_id != request.query_id
            || self.pairing.pair_id != request.pair_id
            || self.pairing.request_ticket.as_ref().is_none_or(|ticket| {
                ticket.subject_account_id != request.asserted_account_id
                    || ticket.subject_project_id != request.asserted_project_id
            })
            || match actor_role {
                MaplePairingRole::Controller => {
                    self.pairing.controller_registration_id != request.actor_registration_id
                }
                MaplePairingRole::Host => {
                    self.pairing.host_registration_id != request.actor_registration_id
                }
            }
        {
            return Err(Error::InvalidResponse(
                "Maple pairing status response does not match its query".to_string(),
            ));
        }
        Ok(VerifiedMaplePairingStatusResponse {
            protocol_version: self.protocol_version,
            query_id: self.query_id,
            pairing: self
                .pairing
                .verify(issuers, actor_role, trusted_now_unix_ms)?,
        })
    }
}

impl MaplePairingRevocationListResponse {
    pub(crate) fn verify_for(
        self,
        request: &ListMaplePairingRevocationsRequest,
        issuers: &MaplePairingIssuerKeySet,
    ) -> Result<VerifiedMaplePairingRevocationListResponse> {
        request.canonical_transcript()?;
        let effective_limit = validate_limit(request.limit)?;
        let revocation_sync = self.revocation_sync.verify(issuers)?;
        let checkpoint_inner = revocation_sync.checkpoint().as_inner();
        let discovery = request.revocation_stream_id.is_nil()
            && request.revocation_stream_generation == 0
            && request.after_issuer_sequence == 0;
        let reset_page_shape_is_valid = match self.revocation_sync.status {
            MapleRevocationSyncStatusV1::ResetClearRequired => {
                self.events.len() == 1
                    && matches!(
                        (&self.events[0], self.revocation_sync.reset_clear_instruction.as_ref()),
                        (MapleRevocationStreamEventV1::ResetClearRequired(event), Some(sync_event))
                            if event == sync_event
                    )
            }
            MapleRevocationSyncStatusV1::Ready
            | MapleRevocationSyncStatusV1::RevocationsPending => true,
        };
        if self.protocol_version != MAPLE_PAIRING_PROTOCOL_VERSION
            || self.query_id != request.query_id
            || self.events.len() > usize::from(effective_limit)
            || (self.has_more && self.events.is_empty())
            || !reset_page_shape_is_valid
            || checkpoint_inner.subject_account_id != request.asserted_account_id
            || checkpoint_inner.subject_project_id != request.asserted_project_id
            || checkpoint_inner.host.registration_id != request.host_registration_id
            || (!discovery
                && (checkpoint_inner.revocation_stream_id != request.revocation_stream_id
                    || checkpoint_inner.revocation_stream_generation
                        != request.revocation_stream_generation))
            || request.after_issuer_sequence > checkpoint_inner.last_acked_issuer_sequence
            || self.next_after_issuer_sequence > checkpoint_inner.last_issued_issuer_sequence
            || self.has_more
                != (self.next_after_issuer_sequence < checkpoint_inner.last_issued_issuer_sequence)
        {
            return Err(Error::InvalidResponse(
                "Maple pairing revocation page is unsupported or internally inconsistent"
                    .to_string(),
            ));
        }
        let mut expected = request
            .after_issuer_sequence
            .checked_add(1)
            .ok_or_else(|| {
                Error::Configuration("Maple pairing revocation cursor cannot advance".to_string())
            })?;
        let mut event_ids = HashSet::with_capacity(self.events.len());
        let mut verified = Vec::with_capacity(self.events.len());
        let mut expected_next = request.after_issuer_sequence;
        for event in self.events {
            let event_matches_scope = match &event {
                MapleRevocationStreamEventV1::PairRevocation(event) => {
                    event.recipient_host_registration_id == request.host_registration_id
                        && event.subject_account_id == request.asserted_account_id
                        && event.subject_project_id == request.asserted_project_id
                        && event.revocation_stream_id == checkpoint_inner.revocation_stream_id
                        && event.revocation_stream_generation
                            == checkpoint_inner.revocation_stream_generation
                        && device_claim_is_same_identity_at_or_before(
                            &event.host,
                            &checkpoint_inner.host,
                        )
                }
                MapleRevocationStreamEventV1::ResetClearRequired(event) => {
                    if revocation_sync.has_reset_clear_instruction() {
                        event.verify_against_checkpoint(issuers, revocation_sync.checkpoint())?;
                    } else {
                        event
                            .verify(issuers)?
                            .verify_historical_against_checkpoint(revocation_sync.checkpoint())?;
                    }
                    event.recipient_host_registration_id == request.host_registration_id
                        && event.subject_account_id == request.asserted_account_id
                        && event.subject_project_id == request.asserted_project_id
                        && event.security_epoch == checkpoint_inner.security_epoch
                        && event.revocation_stream_id == checkpoint_inner.revocation_stream_id
                        && event.revocation_stream_generation
                            == checkpoint_inner.revocation_stream_generation
                        && device_claim_is_same_identity_at_or_before(
                            &event.host,
                            &checkpoint_inner.host,
                        )
                }
            };
            if event.issuer_sequence() != expected
                || !event_matches_scope
                || !event_ids.insert(event.event_id())
            {
                return Err(Error::InvalidResponse(
                    "Maple pairing revocation page has a gap or mismatched host/event".to_string(),
                ));
            }
            expected = expected.checked_add(1).ok_or_else(|| {
                Error::InvalidResponse("Maple pairing revocation sequence overflowed".to_string())
            })?;
            expected_next = event.issuer_sequence();
            verified.push(event.verify(issuers)?);
        }
        if self.next_after_issuer_sequence != expected_next {
            return Err(Error::InvalidResponse(
                "Maple pairing revocation page returned a non-progressing sequence cursor"
                    .to_string(),
            ));
        }
        Ok(VerifiedMaplePairingRevocationListResponse {
            protocol_version: self.protocol_version,
            query_id: self.query_id,
            revocation_sync,
            events: verified,
            next_after_issuer_sequence: self.next_after_issuer_sequence,
            has_more: self.has_more,
        })
    }
}

impl MaplePairingRevocationAckResponse {
    pub(crate) fn verify_for(
        self,
        request: &AckMaplePairingRevocationRequest,
        issuers: &MaplePairingIssuerKeySet,
    ) -> Result<VerifiedMaplePairingRevocationAckResponse> {
        request.canonical_transcript()?;
        let checkpoint = self.stream_checkpoint.verify(issuers)?;
        let checkpoint_inner = checkpoint.as_inner();
        if self.protocol_version != MAPLE_PAIRING_PROTOCOL_VERSION
            || self.operation_id != request.operation_id
            || self.host_registration_id != request.host_registration_id
            || self.event_id != request.event_id
            || self.issuer_sequence != request.issuer_sequence
            || self.last_acked_issuer_sequence != request.issuer_sequence
            || checkpoint_inner.subject_account_id != request.asserted_account_id
            || checkpoint_inner.subject_project_id != request.asserted_project_id
            || checkpoint_inner.host.registration_id != request.host_registration_id
            || checkpoint_inner.revocation_stream_id != request.revocation_stream_id
            || checkpoint_inner.revocation_stream_generation != request.revocation_stream_generation
            || checkpoint_inner.last_acked_issuer_sequence != request.issuer_sequence
            || self.accepted_at_unix_ms < 0
        {
            return Err(Error::InvalidResponse(
                "Maple pairing revocation acknowledgement does not match its request".to_string(),
            ));
        }
        Ok(VerifiedMaplePairingRevocationAckResponse {
            protocol_version: self.protocol_version,
            operation_id: self.operation_id,
            host_registration_id: self.host_registration_id,
            event_id: self.event_id,
            issuer_sequence: self.issuer_sequence,
            last_acked_issuer_sequence: self.last_acked_issuer_sequence,
            accepted_at_unix_ms: self.accepted_at_unix_ms,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use serde::de::DeserializeOwned;
    use serde_json::Value;

    const FIXTURE_JSON: &str = include_str!("../tests/fixtures/maple_pairing_v1_vectors.json");

    fn fixture() -> Value {
        serde_json::from_str(FIXTURE_JSON).expect("literal server pairing fixture")
    }

    fn fixture_value<T: DeserializeOwned>(name: &str) -> T {
        serde_json::from_value(fixture()[name].clone()).expect("fixture value matches SDK DTO")
    }

    fn fixture_string(name: &str) -> String {
        fixture()[name]
            .as_str()
            .expect("fixture value is a string")
            .to_string()
    }

    fn trusted_issuers() -> MaplePairingIssuerKeySet {
        MaplePairingIssuerKeySet::from_v1(fixture_value("issuer_keyset"))
            .expect("literal issuer key set is trusted explicitly")
    }

    #[test]
    fn local_pairing_fixture_bytes_are_exactly_pinned() {
        assert_eq!(
            hex::encode(Sha256::digest(FIXTURE_JSON.as_bytes())),
            "02be99e6c57c9e79de54a501fecc5ec5eeba19a2ef1d4ac54f131daff0f80a56"
        );
        assert!(FIXTURE_JSON.ends_with('\n'));
        assert!(!FIXTURE_JSON.ends_with("\n\n"));
        assert!(!FIXTURE_JSON.contains('\r'));
    }

    #[test]
    fn backend_owned_extended_vectors_cover_materialization_retirement_and_authority_edges() {
        let fixture = fixture();
        assert_eq!(fixture["fixture_schema_version"].as_u64(), Some(1));

        let create_bundle = &fixture["typed_materialization_vectors"]["create"];
        let create_request: CreateMaplePairingRequest =
            serde_json::from_value(create_bundle["request"].clone()).unwrap();
        let create_ticket: MaplePairRequestTicketV1 =
            serde_json::from_value(create_bundle["request_ticket"].clone()).unwrap();
        let create_response: MaplePairingMutationResponse =
            serde_json::from_value(create_bundle["response"].clone()).unwrap();
        assert_eq!(create_request, fixture_value("create_request"));
        assert_eq!(create_ticket, fixture_value("request_ticket"));
        create_response
            .verify_create(
                &create_request,
                &trusted_issuers(),
                create_ticket.created_at_unix_ms,
            )
            .unwrap();

        let revoke_bundle = &fixture["typed_materialization_vectors"]["revoke"];
        let revoke_request: RevokeMaplePairingRequest =
            serde_json::from_value(revoke_bundle["request"].clone()).unwrap();
        let revoke_response: MaplePairingMutationResponse =
            serde_json::from_value(revoke_bundle["response"].clone()).unwrap();
        assert_eq!(revoke_request, fixture_value("revoke_request"));
        assert_eq!(revoke_bundle["request_ticket"], fixture["request_ticket"]);
        assert_eq!(
            revoke_bundle["pair_authorization"],
            fixture["pair_authorization"]
        );
        assert_eq!(revoke_bundle["revocation"], fixture["pair_revocation"]);
        assert_eq!(revoke_bundle["response"], fixture["revoke_receipt"]);
        revoke_response
            .verify_revoke(
                &revoke_request,
                &trusted_issuers(),
                create_ticket.created_at_unix_ms,
            )
            .unwrap();

        let post_ack = &fixture["post_ack_registration_outcome_vectors"];
        let exact_old: crate::RegisterMapleDeviceRequest =
            serde_json::from_value(post_ack["exact_pre_reset_replay"]["request"].clone()).unwrap();
        let changed_old: crate::RegisterMapleDeviceRequest =
            serde_json::from_value(post_ack["changed_pre_reset_same_operation"]["request"].clone())
                .unwrap();
        let exact_pending: crate::RegisterMapleDeviceRequest =
            serde_json::from_value(post_ack["exact_pending_reset_replay"]["request"].clone())
                .unwrap();
        let changed_pending: crate::RegisterMapleDeviceRequest = serde_json::from_value(
            post_ack["changed_pending_reset_same_operation"]["request"].clone(),
        )
        .unwrap();
        let retired_fresh: crate::RegisterMapleDeviceRequest = serde_json::from_value(
            post_ack["fresh_operation_retired_installation"]["request"].clone(),
        )
        .unwrap();
        let fresh_installation: crate::RegisterMapleDeviceRequest =
            serde_json::from_value(post_ack["fresh_installation_current_epoch"]["request"].clone())
                .unwrap();
        for request in [
            &exact_old,
            &changed_old,
            &exact_pending,
            &changed_pending,
            &retired_fresh,
            &fresh_installation,
        ] {
            request.validate().unwrap();
        }
        assert_eq!(exact_old.operation_id, changed_old.operation_id);
        assert_ne!(exact_old, changed_old);
        assert_eq!(exact_pending.operation_id, changed_pending.operation_id);
        assert_ne!(exact_pending, changed_pending);
        assert_eq!(retired_fresh.installation_id, exact_pending.installation_id);
        assert_ne!(retired_fresh.operation_id, exact_pending.operation_id);
        assert_ne!(
            fresh_installation.installation_id,
            exact_pending.installation_id
        );
        assert_ne!(
            fresh_installation.identity_public_key,
            exact_pending.identity_public_key
        );
        assert_eq!(
            post_ack["exact_pre_reset_replay"]["response"],
            fixture["register_device_response_ready"]
        );
        assert_eq!(
            post_ack["exact_pre_reset_replay"]["expected"],
            "frozen_ready_replay"
        );
        assert_eq!(
            post_ack["changed_pre_reset_same_operation"]["expected"],
            "conflict"
        );
        assert_eq!(
            post_ack["exact_pending_reset_replay"]["response"],
            fixture["register_device_response_reset_clear_required"]
        );
        assert_eq!(
            post_ack["exact_pending_reset_replay"]["expected"],
            "reset_clear_required_replay"
        );
        assert_eq!(
            post_ack["changed_pending_reset_same_operation"]["expected"],
            "conflict"
        );
        assert_eq!(
            post_ack["fresh_operation_retired_installation"]["expected"],
            "MapleInstallationRetired"
        );
        assert_eq!(
            post_ack["fresh_installation_current_epoch"]["expected"],
            "ready"
        );
        assert_eq!(
            post_ack["operation_replay_precedes_retirement_and_epoch_gates"],
            true
        );
        let reset_response: crate::MapleDeviceRegistrationResponse = serde_json::from_value(
            fixture["register_device_response_reset_clear_required"].clone(),
        )
        .unwrap();
        let reset_instruction: MapleResetClearRequiredV1 = serde_json::from_value(
            fixture["reset_clear_three_reset_chain"][2]["instruction"].clone(),
        )
        .unwrap();
        let first_reset: MapleResetClearRequiredV1 = serde_json::from_value(
            fixture["reset_clear_three_reset_chain"][0]["instruction"].clone(),
        )
        .unwrap();
        let frozen_ready: crate::MapleDeviceRegistrationResponse =
            serde_json::from_value(fixture["register_device_response_ready"].clone()).unwrap();
        let frozen_ready_checkpoint = &frozen_ready.revocation_sync.stream_checkpoint;
        assert_eq!(frozen_ready_checkpoint.host, first_reset.host);
        assert_eq!(
            frozen_ready_checkpoint.security_epoch,
            first_reset.source_security_epoch
        );
        assert_eq!(
            frozen_ready_checkpoint.revocation_stream_id,
            first_reset.source_revocation_stream_id
        );
        assert_eq!(
            frozen_ready_checkpoint.revocation_stream_generation,
            first_reset.source_revocation_stream_generation
        );
        let ack_response: MaplePairingRevocationAckResponse =
            serde_json::from_value(fixture["reset_clear_ack_response"].clone()).unwrap();
        assert!(
            reset_response.accepted_at.timestamp_millis() >= reset_instruction.reset_at_unix_ms
        );
        assert!(reset_response.accepted_at.timestamp_millis() < ack_response.accepted_at_unix_ms);
        let fresh_response: crate::MapleDeviceRegistrationResponse = serde_json::from_value(
            post_ack["fresh_installation_current_epoch"]["response"].clone(),
        )
        .unwrap();
        fresh_response
            .revocation_sync
            .verify_against_registration(
                &fresh_installation,
                fresh_response.registration_id,
                fresh_response.security_epoch,
                &trusted_issuers(),
            )
            .unwrap();
        assert!(fresh_response.accepted_at.timestamp_millis() > ack_response.accepted_at_unix_ms);

        let two_host = &fixture["two_host_ack_namespace_vectors"];
        let host_a_request: AckMaplePairingRevocationRequest =
            serde_json::from_value(two_host["host_a"]["request"].clone()).unwrap();
        let host_b_request: AckMaplePairingRevocationRequest =
            serde_json::from_value(two_host["host_b"]["request"].clone()).unwrap();
        let host_a_response: MaplePairingRevocationAckResponse =
            serde_json::from_value(two_host["host_a"]["response"].clone()).unwrap();
        let host_b_response: MaplePairingRevocationAckResponse =
            serde_json::from_value(two_host["host_b"]["response"].clone()).unwrap();
        assert_eq!(host_a_request.operation_id, host_b_request.operation_id);
        assert_ne!(
            host_a_request.host_registration_id,
            host_b_request.host_registration_id
        );
        assert_ne!(
            host_a_request.canonical_transcript().unwrap(),
            host_b_request.canonical_transcript().unwrap()
        );
        let host_a_key = claim_signing_key(&host_a_response.stream_checkpoint.host);
        let host_b_key = claim_signing_key(&host_b_response.stream_checkpoint.host);
        host_a_request
            .validate_with_signing_key(&host_a_key)
            .unwrap();
        host_b_request
            .validate_with_signing_key(&host_b_key)
            .unwrap();
        host_a_response
            .clone()
            .verify_for(&host_a_request, &trusted_issuers())
            .unwrap();
        host_b_response
            .clone()
            .verify_for(&host_b_request, &trusted_issuers())
            .unwrap();
        assert!(host_a_response
            .verify_for(&host_b_request, &trusted_issuers())
            .is_err());
        assert!(host_b_response
            .verify_for(&host_a_request, &trusted_issuers())
            .is_err());

        let successor = &fixture["reset_clear_successor_vectors"];
        assert_eq!(
            successor["exact_full_host_claim_fields"],
            serde_json::json!([
                "registration_id",
                "device_id",
                "installation_id",
                "identity_algorithm",
                "identity_public_key",
                "endpoint_id",
                "endpoint_epoch"
            ])
        );
        let predecessor: MapleResetClearRequiredV1 =
            serde_json::from_value(successor["predecessor"].clone()).unwrap();
        let accepted: MapleResetClearRequiredV1 =
            serde_json::from_value(successor["accepted"]["instruction"].clone()).unwrap();
        let changed_host: MapleResetClearRequiredV1 =
            serde_json::from_value(successor["changed_host"]["instruction"].clone()).unwrap();
        assert_eq!(predecessor.host, accepted.host);
        assert_ne!(predecessor.host, changed_host.host);
        let predecessor = predecessor.verify(&trusted_issuers()).unwrap();
        accepted
            .verify(&trusted_issuers())
            .unwrap()
            .verify_successor_of(&predecessor)
            .unwrap();
        assert!(changed_host
            .verify(&trusted_issuers())
            .unwrap()
            .verify_successor_of(&predecessor)
            .is_err());

        let rotation = &fixture["issuer_rotation_vectors"];
        let artifact: MaplePairRequestTicketV1 =
            serde_json::from_value(rotation["artifact_signed_by_initial"].clone()).unwrap();
        let initial = MaplePairingIssuerKeySet::from_v1(
            serde_json::from_value(rotation["initial"]["keyset"].clone()).unwrap(),
        )
        .unwrap();
        let retained = MaplePairingIssuerKeySet::from_v1(
            serde_json::from_value(rotation["rotated_retaining_previous"]["keyset"].clone())
                .unwrap(),
        )
        .unwrap();
        let omitted = MaplePairingIssuerKeySet::from_v1(
            serde_json::from_value(rotation["rotated_without_previous"]["keyset"].clone()).unwrap(),
        )
        .unwrap();
        let remapped = MaplePairingIssuerKeySet::from_v1(
            serde_json::from_value(rotation["remapped_previous_key_id"]["keyset"].clone()).unwrap(),
        )
        .unwrap();
        for issuers in [&initial, &retained] {
            artifact
                .verify_at(
                    issuers,
                    artifact.created_at_unix_ms,
                    0,
                    MAPLE_PAIRING_MAX_TICKET_TTL_MS,
                )
                .unwrap();
        }
        assert!(artifact
            .verify_at(
                &omitted,
                artifact.created_at_unix_ms,
                0,
                MAPLE_PAIRING_MAX_TICKET_TTL_MS,
            )
            .is_err());
        assert!(artifact
            .verify_at(
                &remapped,
                artifact.created_at_unix_ms,
                0,
                MAPLE_PAIRING_MAX_TICKET_TTL_MS,
            )
            .is_err());
        assert_eq!(
            rotation["rotated_retaining_previous"]["registry_reconciliation_expected"],
            "append_accepted"
        );
        assert_eq!(
            rotation["rotated_without_previous"]["registry_reconciliation_expected"],
            "MaplePairingIssuerConfigurationConflict"
        );
        assert_eq!(
            rotation["remapped_previous_key_id"]["registry_reconciliation_expected"],
            "MaplePairingIssuerConfigurationConflict"
        );

        let tombstones = &fixture["registration_operation_tombstone_vectors"];
        assert_eq!(tombstones["exact_old_request"], "frozen_ready_replay");
        assert_eq!(
            tombstones["exact_pending_reset_request_after_ack"],
            "reset_clear_required_replay"
        );
        assert_eq!(
            tombstones["changed_pending_reset_request_after_ack"],
            "conflict"
        );
        assert_eq!(
            tombstones["fresh_operation_retired_installation"],
            "MapleInstallationRetired"
        );
        assert_eq!(tombstones["fresh_installation_current_epoch"], "ready");
    }

    fn reconciled_stream(name: &str) -> DurablyReconciledMapleRevocationStreamV1 {
        fixture_value::<MapleRevocationStreamCheckpointV1>(name)
            .verify(&trusted_issuers())
            .unwrap()
            .plan_durable_transition(None)
            .unwrap()
            .after_durable_commit()
    }

    fn claim_signing_key(claim: &MaplePairingDeviceClaimV1) -> [u8; 32] {
        decode_exact_base64(
            &claim.identity_public_key,
            ED25519_PUBLIC_KEY_BYTES,
            "fixture signing public key",
            ErrorClass::Configuration,
        )
        .expect("fixture key")
        .try_into()
        .expect("exact key length")
    }

    fn resign_revocation(mut revocation: MaplePairRevocationV1) -> MaplePairRevocationV1 {
        let seed = hex::decode(
            fixture()["test_private_seeds_hex"]["issuer"]
                .as_str()
                .unwrap(),
        )
        .unwrap();
        let seed: [u8; 32] = seed.try_into().unwrap();
        let signature =
            SigningKey::from_bytes(&seed).sign(&revocation.canonical_transcript().unwrap());
        revocation.issuer_signature = BASE64.encode(signature.to_bytes());
        revocation
    }

    #[test]
    fn literal_server_vectors_match_every_frozen_transcript_and_signature() {
        let create: CreateMaplePairingRequest = fixture_value("create_request");
        assert_eq!(
            hex::encode(create.canonical_transcript().unwrap()),
            fixture_string("create_request_transcript_hex")
        );
        assert_eq!(
            create.transcript_digest().unwrap(),
            fixture_string("create_request_digest")
        );
        create.validate().unwrap();

        let ticket: MaplePairRequestTicketV1 = fixture_value("request_ticket");
        assert_eq!(
            hex::encode(ticket.canonical_transcript().unwrap()),
            fixture_string("request_ticket_transcript_hex")
        );
        assert_eq!(
            ticket.transcript_digest().unwrap(),
            fixture_string("request_ticket_digest")
        );
        ticket
            .verify_at(
                &trusted_issuers(),
                ticket.created_at_unix_ms,
                MAPLE_PAIRING_MAX_CLOCK_SKEW_MS,
                MAPLE_PAIRING_MAX_TICKET_TTL_MS,
            )
            .unwrap();

        let list: ListMaplePairingsRequest = fixture_value("list_pairings_request");
        assert_eq!(
            hex::encode(list.canonical_transcript().unwrap()),
            fixture_string("list_pairings_request_transcript_hex")
        );
        assert_eq!(
            sha256_base64(&list.canonical_transcript().unwrap()),
            fixture_string("list_pairings_request_digest")
        );
        list.validate_with_signing_key(&claim_signing_key(&ticket.controller))
            .unwrap();
        let list_response: MaplePairingListResponse = fixture_value("list_pairings_response");
        list_response
            .verify_for(&list, &trusted_issuers(), ticket.created_at_unix_ms)
            .unwrap();

        let status: MaplePairingStatusRequest = fixture_value("pairing_status_request");
        assert_eq!(
            hex::encode(status.canonical_transcript().unwrap()),
            fixture_string("pairing_status_request_transcript_hex")
        );
        assert_eq!(
            sha256_base64(&status.canonical_transcript().unwrap()),
            fixture_string("pairing_status_request_digest")
        );
        status
            .validate_with_signing_key(&claim_signing_key(&ticket.controller))
            .unwrap();
        let status_response: MaplePairingStatusResponse = fixture_value("pairing_status_response");
        let active_at = status_response.pairing.activated_at_unix_ms.unwrap();
        status_response
            .verify_for(
                &status,
                MaplePairingRole::Controller,
                &trusted_issuers(),
                active_at,
            )
            .unwrap();

        let approval: ApproveMaplePairingRequest = fixture_value("approval_request");
        assert_eq!(
            hex::encode(approval.canonical_transcript().unwrap()),
            fixture_string("approval_request_transcript_hex")
        );
        assert_eq!(
            approval.transcript_digest().unwrap(),
            fixture_string("approval_request_digest")
        );
        approval
            .validate_with_signing_key(&claim_signing_key(&ticket.host))
            .unwrap();

        let authorization: MaplePairAuthorizationV1 = fixture_value("pair_authorization");
        assert_eq!(
            hex::encode(authorization.canonical_transcript().unwrap()),
            fixture_string("pair_authorization_transcript_hex")
        );
        assert_eq!(
            authorization.transcript_digest().unwrap(),
            fixture_string("pair_authorization_digest")
        );
        authorization
            .verify_against_ticket(&trusted_issuers(), &ticket)
            .unwrap();

        let confirm: ConfirmMaplePairingRequest = fixture_value("confirm_request");
        assert_eq!(
            hex::encode(confirm.canonical_transcript().unwrap()),
            fixture_string("confirm_request_transcript_hex")
        );
        assert_eq!(
            sha256_base64(&confirm.canonical_transcript().unwrap()),
            fixture_string("confirm_request_digest")
        );
        confirm
            .validate_with_signing_key(&claim_signing_key(&authorization.host))
            .unwrap();

        let active_receipt: MaplePairingMutationResponse = fixture_value("active_receipt");
        let verified_active = active_receipt
            .verify_confirm(
                &confirm,
                &trusted_issuers(),
                authorization.approved_at_unix_ms,
            )
            .unwrap();
        assert_eq!(
            verified_active.pairing.as_inner().state,
            MaplePairingState::Active
        );
        assert_eq!(verified_active.pairing.as_inner().revision, 3);

        let revocation: MaplePairRevocationV1 = fixture_value("pair_revocation");
        assert_eq!(
            hex::encode(revocation.canonical_transcript().unwrap()),
            fixture_string("pair_revocation_transcript_hex")
        );
        assert_eq!(
            revocation.transcript_digest().unwrap(),
            fixture_string("pair_revocation_digest")
        );
        revocation.verify_issuer(&trusted_issuers()).unwrap();

        let revoke: RevokeMaplePairingRequest = fixture_value("revoke_request");
        assert_eq!(
            hex::encode(revoke.canonical_transcript().unwrap()),
            fixture_string("revoke_request_transcript_hex")
        );
        assert_eq!(
            sha256_base64(&revoke.canonical_transcript().unwrap()),
            fixture_string("revoke_request_digest")
        );
        revoke
            .validate_with_signing_key(&claim_signing_key(&ticket.controller))
            .unwrap();
        let revoke_receipt: MaplePairingMutationResponse = fixture_value("revoke_receipt");
        let verified_revoke = revoke_receipt
            .verify_revoke(&revoke, &trusted_issuers(), revocation.revoked_at_unix_ms)
            .unwrap();
        assert_eq!(
            verified_revoke.pairing.as_inner().state,
            MaplePairingState::Revoked
        );

        let revocations: ListMaplePairingRevocationsRequest =
            fixture_value("list_revocations_request");
        assert_eq!(
            hex::encode(revocations.canonical_transcript().unwrap()),
            fixture_string("list_revocations_request_transcript_hex")
        );
        assert_eq!(
            sha256_base64(&revocations.canonical_transcript().unwrap()),
            fixture_string("list_revocations_request_digest")
        );
        revocations
            .validate_with_signing_key(&claim_signing_key(&ticket.host))
            .unwrap();
        let revocations_response: MaplePairingRevocationListResponse =
            fixture_value("list_revocations_response");
        let verified_page = revocations_response
            .verify_for(&revocations, &trusted_issuers())
            .unwrap();
        assert_eq!(
            verified_page
                .revocation_sync
                .checkpoint()
                .as_inner()
                .host
                .endpoint_epoch,
            5
        );

        let checkpoint: MapleRevocationStreamCheckpointV1 =
            fixture_value("revocation_stream_checkpoint");
        assert_eq!(
            hex::encode(checkpoint.canonical_transcript().unwrap()),
            fixture_string("revocation_stream_checkpoint_transcript_hex")
        );
        assert_eq!(
            checkpoint.transcript_digest().unwrap(),
            fixture_string("revocation_stream_checkpoint_digest")
        );
        checkpoint.verify(&trusted_issuers()).unwrap();

        let discovery: ListMaplePairingRevocationsRequest =
            fixture_value("discovery_list_revocations_request");
        assert_eq!(
            hex::encode(discovery.canonical_transcript().unwrap()),
            fixture_string("discovery_list_revocations_request_transcript_hex")
        );
        assert_eq!(
            sha256_base64(&discovery.canonical_transcript().unwrap()),
            fixture_string("discovery_list_revocations_request_digest")
        );
        discovery
            .validate_with_signing_key(&claim_signing_key(&ticket.host))
            .unwrap();
        let discovery_response: MaplePairingRevocationListResponse =
            fixture_value("discovery_list_revocations_response");
        discovery_response
            .verify_for(&discovery, &trusted_issuers())
            .unwrap();
        let discovery_checkpoint: MapleRevocationStreamCheckpointV1 =
            fixture_value("discovery_revocation_stream_checkpoint");
        assert_eq!(
            hex::encode(discovery_checkpoint.canonical_transcript().unwrap()),
            fixture_string("discovery_revocation_stream_checkpoint_transcript_hex")
        );
        assert_eq!(
            discovery_checkpoint.transcript_digest().unwrap(),
            fixture_string("discovery_revocation_stream_checkpoint_digest")
        );
        discovery_checkpoint.verify(&trusted_issuers()).unwrap();

        let ack: AckMaplePairingRevocationRequest = fixture_value("ack_revocation_request");
        assert_eq!(
            hex::encode(ack.canonical_transcript().unwrap()),
            fixture_string("ack_revocation_request_transcript_hex")
        );
        assert_eq!(
            sha256_base64(&ack.canonical_transcript().unwrap()),
            fixture_string("ack_revocation_request_digest")
        );
        ack.validate_with_signing_key(&claim_signing_key(&ticket.host))
            .unwrap();
        let ack_response: MaplePairingRevocationAckResponse =
            fixture_value("ack_revocation_response");
        ack_response.verify_for(&ack, &trusted_issuers()).unwrap();
        let ack_checkpoint: MapleRevocationStreamCheckpointV1 =
            fixture_value("ack_revocation_stream_checkpoint");
        assert_eq!(
            hex::encode(ack_checkpoint.canonical_transcript().unwrap()),
            fixture_string("ack_revocation_stream_checkpoint_transcript_hex")
        );
        assert_eq!(
            ack_checkpoint.transcript_digest().unwrap(),
            fixture_string("ack_revocation_stream_checkpoint_digest")
        );
        ack_checkpoint.verify(&trusted_issuers()).unwrap();
    }

    #[test]
    fn pairing_v1_has_only_directed_incarnation_and_no_shared_account_epoch() {
        let fixture = fixture().to_string();
        assert!(fixture.contains("pairing_incarnation"));
        assert!(!fixture.contains("account_authorization_epoch"));
        assert!(!fixture.contains("account_epoch"));

        let ticket: MaplePairRequestTicketV1 = fixture_value("request_ticket");
        assert_ne!(
            ticket.controller.registration_id,
            ticket.host.registration_id
        );
        assert_ne!(ticket.controller.endpoint_id, ticket.host.endpoint_id);
        assert_eq!(ticket.direction, MaplePairingDirection::ControllerToHost);
        assert_ne!(ticket.pairing_incarnation, 0);
    }

    #[test]
    fn ticket_verify_at_enforces_trusted_time_skew_and_maximum_ttl() {
        let ticket: MaplePairRequestTicketV1 = fixture_value("request_ticket");
        let issuers = trusted_issuers();
        ticket
            .verify_at(
                &issuers,
                ticket.created_at_unix_ms - MAPLE_PAIRING_MAX_CLOCK_SKEW_MS,
                MAPLE_PAIRING_MAX_CLOCK_SKEW_MS,
                MAPLE_PAIRING_MAX_TICKET_TTL_MS,
            )
            .unwrap();
        ticket
            .verify_at(
                &issuers,
                ticket.expires_at_unix_ms + MAPLE_PAIRING_MAX_CLOCK_SKEW_MS - 1,
                MAPLE_PAIRING_MAX_CLOCK_SKEW_MS,
                MAPLE_PAIRING_MAX_TICKET_TTL_MS,
            )
            .unwrap();
        assert!(matches!(
            ticket.verify_at(
                &issuers,
                ticket.expires_at_unix_ms + MAPLE_PAIRING_MAX_CLOCK_SKEW_MS,
                MAPLE_PAIRING_MAX_CLOCK_SKEW_MS,
                MAPLE_PAIRING_MAX_TICKET_TTL_MS,
            ),
            Err(Error::InvalidResponse(_))
        ));
        assert!(matches!(
            ticket.verify_at(
                &issuers,
                ticket.created_at_unix_ms,
                MAPLE_PAIRING_MAX_CLOCK_SKEW_MS + 1,
                MAPLE_PAIRING_MAX_TICKET_TTL_MS,
            ),
            Err(Error::Configuration(_))
        ));

        let mut excessive_ttl = ticket.clone();
        excessive_ttl.expires_at_unix_ms =
            excessive_ttl.created_at_unix_ms + MAPLE_PAIRING_MAX_TICKET_TTL_MS + 1;
        assert!(matches!(
            excessive_ttl.verify_at(
                &issuers,
                excessive_ttl.created_at_unix_ms,
                0,
                MAPLE_PAIRING_MAX_TICKET_TTL_MS,
            ),
            Err(Error::InvalidResponse(_))
        ));
    }

    #[test]
    fn artifact_verification_rejects_untrusted_tampered_and_mismatched_keys() {
        let authorization: MaplePairAuthorizationV1 = fixture_value("pair_authorization");
        let ticket: MaplePairRequestTicketV1 = fixture_value("request_ticket");
        let untrusted =
            MaplePairingIssuerKeySet::new([("different-key", claim_signing_key(&ticket.host))])
                .unwrap();
        assert!(matches!(
            authorization.verify_against_ticket(&untrusted, &ticket),
            Err(Error::InvalidResponse(_))
        ));

        let mut tampered = authorization.clone();
        tampered.pairing_incarnation += 1;
        assert!(matches!(
            tampered.verify_against_ticket(&trusted_issuers(), &ticket),
            Err(Error::InvalidResponse(_))
        ));

        let mut wrong_approval_revision = authorization.clone();
        wrong_approval_revision.host_approval_expected_pairing_revision = 2;
        assert!(matches!(
            wrong_approval_revision.canonical_transcript(),
            Err(Error::InvalidResponse(_))
        ));

        let mut mismatched = authorization;
        mismatched.controller.endpoint_id = mismatched.host.endpoint_id.clone();
        assert!(matches!(
            mismatched.canonical_transcript(),
            Err(Error::InvalidResponse(message)) if message.contains("must match")
        ));

        let mut collapsed: MaplePairRequestTicketV1 = fixture_value("request_ticket");
        collapsed.host.registration_id = collapsed.controller.registration_id;
        assert!(matches!(
            collapsed.canonical_transcript(),
            Err(Error::InvalidResponse(_))
        ));

        let ticket: MaplePairRequestTicketV1 = fixture_value("request_ticket");
        let mut early_authorization: MaplePairAuthorizationV1 = fixture_value("pair_authorization");
        early_authorization.approved_at_unix_ms = ticket.created_at_unix_ms - 1;
        assert!(matches!(
            early_authorization.validate_ticket_binding(&ticket),
            Err(Error::InvalidResponse(_))
        ));

        let authorization: MaplePairAuthorizationV1 = fixture_value("pair_authorization");
        let mut early_revocation: MaplePairRevocationV1 = fixture_value("pair_revocation");
        early_revocation.revoked_at_unix_ms = authorization.approved_at_unix_ms - 1;
        assert!(matches!(
            early_revocation.validate_authorization_binding(&authorization),
            Err(Error::InvalidResponse(_))
        ));
    }

    #[test]
    fn host_commit_is_manual_and_derived_from_verified_authorization() {
        let active_receipt: MaplePairingMutationResponse = fixture_value("active_receipt");
        let mut awaiting = active_receipt.pairing;
        awaiting.state = MaplePairingState::AwaitingHostCommit;
        awaiting.revision = 2;
        awaiting.activated_at_unix_ms = None;
        let verified_status = awaiting
            .verify(
                &trusted_issuers(),
                MaplePairingRole::Host,
                awaiting.approved_at_unix_ms.unwrap(),
            )
            .unwrap();
        let stream = reconciled_stream("revocation_stream_checkpoint");
        let ready = verified_status
            .host_commit_ready_authorization(&stream)
            .unwrap();
        let unsigned = ConfirmMaplePairingRequest::unsigned_v1_after_durable_commit(
            Uuid::parse_str("eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee").unwrap(),
            &ready,
        )
        .unwrap();
        assert!(unsigned.as_inner().signature.is_empty());
        assert_eq!(
            unsigned.as_inner().pair_authorization_digest,
            fixture_string("pair_authorization_digest")
        );
        assert_eq!(unsigned.as_inner().pairing_incarnation, 3);
        let literal_confirm: ConfirmMaplePairingRequest = fixture_value("confirm_request");
        assert_eq!(
            unsigned.as_inner(),
            &ConfirmMaplePairingRequest {
                signature: String::new(),
                ..literal_confirm
            }
        );

        let active_response: MaplePairingStatusResponse = fixture_value("pairing_status_response");
        let active = active_response
            .pairing
            .verify(
                &trusted_issuers(),
                MaplePairingRole::Controller,
                active_response.pairing.activated_at_unix_ms.unwrap(),
            )
            .unwrap();
        assert!(active.host_commit_ready_authorization(&stream).is_none());
    }

    #[test]
    fn revocation_namespace_transition_rejects_replay_and_requires_durable_reset() {
        let issuers = trusted_issuers();
        let checkpoint =
            fixture_value::<MapleRevocationStreamCheckpointV1>("revocation_stream_checkpoint")
                .verify(&issuers)
                .unwrap();
        let stream_id = checkpoint.as_inner().revocation_stream_id;
        let generation = checkpoint.as_inner().revocation_stream_generation;
        let account_id = checkpoint.as_inner().subject_account_id;
        let project_id = checkpoint.as_inner().subject_project_id;
        let host_registration_id = checkpoint.as_inner().host.registration_id;
        let security_epoch = checkpoint.as_inner().security_epoch;

        let same = MapleInstalledRevocationStreamNamespaceV1::new(
            account_id,
            project_id,
            host_registration_id,
            security_epoch,
            stream_id,
            generation,
        )
        .unwrap();
        let unchanged = checkpoint
            .clone()
            .plan_durable_transition(Some(same))
            .unwrap();
        assert!(!unchanged.requires_admission_reset());
        let reconciled_same = unchanged.after_durable_commit();
        let literal_list: ListMaplePairingRevocationsRequest =
            fixture_value("list_revocations_request");
        let unsigned_list = ListMaplePairingRevocationsRequest::unsigned_v1_from_reconciled_stream(
            literal_list.query_id,
            literal_list.after_issuer_sequence,
            literal_list.limit,
            &reconciled_same,
        )
        .unwrap();
        assert_eq!(
            unsigned_list,
            ListMaplePairingRevocationsRequest {
                signature: String::new(),
                ..literal_list
            }
        );

        let wrong_scope = MapleInstalledRevocationStreamNamespaceV1::new(
            Uuid::new_v4(),
            project_id,
            host_registration_id,
            security_epoch,
            stream_id,
            generation,
        )
        .unwrap();
        assert!(matches!(
            checkpoint
                .clone()
                .plan_durable_transition(Some(wrong_scope)),
            Err(Error::Configuration(_))
        ));

        let wrong_id = MapleInstalledRevocationStreamNamespaceV1::new(
            account_id,
            project_id,
            host_registration_id,
            security_epoch,
            Uuid::parse_str("17171717-1717-4717-8717-171717171717").unwrap(),
            generation,
        )
        .unwrap();
        assert!(matches!(
            checkpoint.clone().plan_durable_transition(Some(wrong_id)),
            Err(Error::InvalidResponse(_))
        ));

        let newer_installed = MapleInstalledRevocationStreamNamespaceV1::new(
            account_id,
            project_id,
            host_registration_id,
            security_epoch,
            Uuid::parse_str("18181818-1818-4818-8818-181818181818").unwrap(),
            generation + 1,
        )
        .unwrap();
        assert!(matches!(
            checkpoint.plan_durable_transition(Some(newer_installed)),
            Err(Error::InvalidResponse(_))
        ));

        let rotated = fixture_value::<MapleRevocationStreamCheckpointV1>(
            "discovery_revocation_stream_checkpoint",
        )
        .verify(&issuers)
        .unwrap();
        let old = MapleInstalledRevocationStreamNamespaceV1::new(
            account_id,
            project_id,
            host_registration_id,
            rotated.as_inner().security_epoch,
            stream_id,
            1,
        )
        .unwrap();
        let reset_before_crash = rotated.clone().plan_durable_transition(Some(old)).unwrap();
        assert!(reset_before_crash.requires_admission_reset());
        drop(reset_before_crash);

        // A crash before the clear/install transaction leaves the old durable
        // namespace intact, so replaying the verified discovery on restart
        // must require the reset again.
        let reset = rotated.plan_durable_transition(Some(old)).unwrap();
        assert!(reset.requires_admission_reset());
        let reconciled_rotated = reset.after_durable_commit();

        let literal_discovery: ListMaplePairingRevocationsRequest =
            fixture_value("discovery_list_revocations_request");
        let unsigned_discovery = ListMaplePairingRevocationsRequest::unsigned_v1_discovery(
            literal_discovery.query_id,
            literal_discovery.asserted_account_id,
            literal_discovery.asserted_project_id,
            literal_discovery.host_registration_id,
            literal_discovery.limit,
        )
        .unwrap();
        assert_eq!(
            unsigned_discovery,
            ListMaplePairingRevocationsRequest {
                signature: String::new(),
                ..literal_discovery.clone()
            }
        );
        let mut partial_discovery = literal_discovery;
        partial_discovery.after_issuer_sequence = 1;
        assert!(partial_discovery.canonical_transcript().is_err());

        let ticket: MaplePairRequestTicketV1 = fixture_value("request_ticket");
        let authorization: MaplePairAuthorizationV1 = fixture_value("pair_authorization");
        let verified_authorization = authorization
            .verify_against_ticket(&issuers, &ticket)
            .unwrap();
        let revocation: MaplePairRevocationV1 = fixture_value("pair_revocation");
        assert!(revocation
            .verify_against_authorization(&issuers, &verified_authorization, &reconciled_rotated,)
            .is_err());
    }

    #[test]
    fn controller_precommit_revocation_is_redacted_and_ticket_bound() {
        let issuers = trusted_issuers();
        let revoke_receipt: MaplePairingMutationResponse = fixture_value("revoke_receipt");
        let mut controller_view = revoke_receipt.pairing;
        let authorization = controller_view.pair_authorization.take().unwrap();
        controller_view.revision = 3;
        controller_view.activated_at_unix_ms = None;

        let verified = controller_view
            .verify(
                &issuers,
                MaplePairingRole::Controller,
                controller_view.revoked_at_unix_ms.unwrap(),
            )
            .unwrap();
        assert!(verified.pair_authorization().is_none());
        assert!(verified
            .host_commit_ready_authorization(&reconciled_stream("revocation_stream_checkpoint"))
            .is_none());
        assert!(controller_view
            .verify(
                &issuers,
                MaplePairingRole::Host,
                controller_view.revoked_at_unix_ms.unwrap(),
            )
            .is_err());

        let mut leaked = controller_view.clone();
        leaked.pair_authorization = Some(authorization);
        assert!(leaked
            .verify(
                &issuers,
                MaplePairingRole::Controller,
                leaked.revoked_at_unix_ms.unwrap(),
            )
            .is_err());

        let mut mismatched = controller_view.clone();
        let mut revocation = mismatched.revocation.take().unwrap();
        revocation.controller.endpoint_epoch += 1;
        mismatched.revocation = Some(resign_revocation(revocation));
        assert!(mismatched
            .verify(
                &issuers,
                MaplePairingRole::Controller,
                mismatched.revoked_at_unix_ms.unwrap(),
            )
            .is_err());

        let mut zero_digest: MaplePairRevocationV1 = fixture_value("pair_revocation");
        zero_digest.pair_authorization_digest = BASE64.encode([0u8; DIGEST_BYTES]);
        assert!(matches!(
            zero_digest.canonical_transcript(),
            Err(Error::InvalidResponse(_))
        ));
    }

    #[test]
    fn approval_and_revocation_ack_builders_preserve_verified_artifacts() {
        let ticket: MaplePairRequestTicketV1 = fixture_value("request_ticket");
        let verified_ticket = ticket
            .verify_at(
                &trusted_issuers(),
                ticket.created_at_unix_ms,
                MAPLE_PAIRING_MAX_CLOCK_SKEW_MS,
                MAPLE_PAIRING_MAX_TICKET_TTL_MS,
            )
            .unwrap();
        let literal_approval: ApproveMaplePairingRequest = fixture_value("approval_request");
        let stream = reconciled_stream("revocation_stream_checkpoint");
        let approval = ApproveMaplePairingRequest::unsigned_v1_from_verified_ticket(
            literal_approval.operation_id,
            literal_approval.host_approval_nonce.clone(),
            &verified_ticket,
            &stream,
        )
        .unwrap();
        assert_eq!(
            approval.as_inner().approved_protocol_min,
            ticket.protocol_min
        );
        assert_eq!(
            approval.as_inner().approved_protocol_max,
            ticket.protocol_max
        );
        assert_eq!(
            approval.as_inner(),
            &ApproveMaplePairingRequest {
                signature: String::new(),
                ..literal_approval
            }
        );

        let revocation: MaplePairRevocationV1 = fixture_value("pair_revocation");
        let authorization: MaplePairAuthorizationV1 = fixture_value("pair_authorization");
        let verified_authorization = authorization
            .verify_against_ticket(&trusted_issuers(), &ticket)
            .unwrap();
        let verified_revocation = revocation
            .verify_against_authorization(&trusted_issuers(), &verified_authorization, &stream)
            .unwrap();
        let literal_ack: AckMaplePairingRevocationRequest = fixture_value("ack_revocation_request");
        let ack = AckMaplePairingRevocationRequest::unsigned_v1_after_durable_commit(
            literal_ack.operation_id,
            literal_ack.expected_previous_issuer_sequence,
            &verified_revocation,
        )
        .unwrap();
        assert_eq!(
            ack.as_inner(),
            &AckMaplePairingRevocationRequest {
                signature: String::new(),
                ..literal_ack
            }
        );
    }

    #[test]
    fn issuer_keyset_and_literal_json_fail_closed_on_shape_drift() {
        let mut keyset: MaplePairingIssuerKeySetV1 = fixture_value("issuer_keyset");
        keyset.keys.push(keyset.keys[0].clone());
        assert!(MaplePairingIssuerKeySet::from_v1(keyset).is_err());

        let mut request = fixture()["create_request"].clone();
        request
            .as_object_mut()
            .unwrap()
            .insert("account_epoch".to_string(), Value::from(7));
        assert!(serde_json::from_value::<CreateMaplePairingRequest>(request).is_err());
    }

    #[test]
    fn list_and_revocation_pages_reject_nonprogress_and_request_mismatches() {
        let list_request = ListMaplePairingsRequest {
            protocol_version: 1,
            transcript_version: 1,
            query_id: Uuid::new_v4(),
            asserted_account_id: Uuid::new_v4(),
            asserted_project_id: Uuid::new_v4(),
            actor_registration_id: Uuid::new_v4(),
            role: MaplePairingRole::Host,
            states: vec![MaplePairingState::Active],
            cursor: Some("opaque-cursor".to_string()),
            limit: Some(1),
            signature: BASE64.encode([0u8; 64]),
        };
        let nonprogressing = MaplePairingListResponse {
            protocol_version: 1,
            query_id: list_request.query_id,
            role: list_request.role,
            pairings: Vec::new(),
            next_cursor: Some("opaque-cursor".to_string()),
            has_more: true,
        };
        assert!(matches!(
            nonprogressing.verify_for(&list_request, &trusted_issuers(), 1),
            Err(Error::InvalidResponse(_))
        ));

        let revocation_request: ListMaplePairingRevocationsRequest =
            fixture_value("list_revocations_request");
        let mut empty_jump: MaplePairingRevocationListResponse =
            fixture_value("list_revocations_response");
        empty_jump.events.clear();
        assert!(matches!(
            empty_jump.verify_for(&revocation_request, &trusted_issuers()),
            Err(Error::InvalidResponse(_))
        ));

        let mut gap: MaplePairingRevocationListResponse =
            fixture_value("list_revocations_response");
        match &mut gap.events[0] {
            MapleRevocationStreamEventV1::PairRevocation(event) => event.issuer_sequence += 1,
            MapleRevocationStreamEventV1::ResetClearRequired(event) => event.issuer_sequence += 1,
        }
        assert!(matches!(
            gap.verify_for(&revocation_request, &trusted_issuers()),
            Err(Error::InvalidResponse(_))
        ));
    }

    fn fixture_reset_admission_leaves(name: &str) -> (u16, Vec<MapleResetClearAdmissionLeafV1>) {
        let vector = &fixture()["reset_clear_admission_set_vectors"][name];
        let leaves = vector["leaves"]
            .as_array()
            .unwrap()
            .iter()
            .map(|leaf| MapleResetClearAdmissionLeafV1 {
                pair_id: serde_json::from_value(leaf["pair_id"].clone()).unwrap(),
                pairing_incarnation: leaf["pairing_incarnation"].as_u64().unwrap(),
                pair_authorization_digest: BASE64
                    .decode(leaf["pair_authorization_digest"].as_str().unwrap())
                    .unwrap()
                    .try_into()
                    .unwrap(),
            })
            .collect();
        (vector["artifact_version"].as_u64().unwrap() as u16, leaves)
    }

    #[test]
    fn reset_clear_admission_vectors_are_literal_bounded_and_order_independent() {
        for name in ["empty", "max_128"] {
            let vector = &fixture()["reset_clear_admission_set_vectors"][name];
            let (artifact_version, mut leaves) = fixture_reset_admission_leaves(name);
            assert_eq!(
                hex::encode(
                    reset_clear_admission_set_transcript(artifact_version, &leaves).unwrap()
                ),
                vector["transcript_hex"].as_str().unwrap()
            );
            assert_eq!(
                BASE64.encode(reset_clear_admission_set_digest(artifact_version, &leaves).unwrap()),
                vector["digest"].as_str().unwrap()
            );
            leaves.reverse();
            assert_eq!(
                BASE64.encode(reset_clear_admission_set_digest(artifact_version, &leaves).unwrap()),
                vector["digest"].as_str().unwrap()
            );
        }

        let (artifact_version, mut leaves) = fixture_reset_admission_leaves("max_128");
        let mut extra = leaves.last().unwrap().clone();
        extra.pair_id = Uuid::new_v4();
        leaves.push(extra);
        assert!(matches!(
            reset_clear_admission_set_transcript(artifact_version, &leaves),
            Err(Error::Configuration(_))
        ));

        let (_, mut duplicate) = fixture_reset_admission_leaves("max_128");
        duplicate[1] = duplicate[0].clone();
        assert!(matches!(
            reset_clear_admission_set_transcript(artifact_version, &duplicate),
            Err(Error::Configuration(_))
        ));
    }

    struct FixturePairingIssuer {
        key_id: String,
        signing_key: SigningKey,
    }

    impl MaplePairingIssuer for FixturePairingIssuer {
        fn key_id(&self) -> &str {
            &self.key_id
        }

        fn public_key_bytes(&self) -> [u8; ED25519_PUBLIC_KEY_BYTES] {
            *self.signing_key.verifying_key().as_bytes()
        }

        fn sign(&self, transcript: &[u8]) -> Result<[u8; ED25519_SIGNATURE_BYTES]> {
            Ok(self.signing_key.sign(transcript).to_bytes())
        }
    }

    fn fixture_pairing_issuer() -> FixturePairingIssuer {
        let seed: [u8; 32] = hex::decode(
            fixture()["test_private_seeds_hex"]["issuer"]
                .as_str()
                .unwrap(),
        )
        .unwrap()
        .try_into()
        .unwrap();
        FixturePairingIssuer {
            key_id: fixture()["issuer_keyset"]["keys"][0]["key_id"]
                .as_str()
                .unwrap()
                .to_string(),
            signing_key: SigningKey::from_bytes(&seed),
        }
    }

    #[test]
    fn three_reset_chain_matches_every_literal_digest_signature_and_successor() {
        let issuers = trusted_issuers();
        let vectors = fixture()["reset_clear_three_reset_chain"]
            .as_array()
            .unwrap()
            .clone();
        let mut previous: Option<VerifiedMapleResetClearRequiredV1> = None;
        for vector in vectors {
            let instruction: MapleResetClearRequiredV1 =
                serde_json::from_value(vector["instruction"].clone()).unwrap();
            assert_eq!(
                hex::encode(reset_clear_instruction_material_transcript(&instruction).unwrap()),
                vector["instruction_material_transcript_hex"]
                    .as_str()
                    .unwrap()
            );
            assert_eq!(
                instruction.instruction_material_digest,
                vector["instruction_material_digest"].as_str().unwrap()
            );
            assert_eq!(
                hex::encode(reset_clear_chain_transcript(&instruction).unwrap()),
                vector["chain_transcript_hex"].as_str().unwrap()
            );
            assert_eq!(
                instruction.chain_digest,
                vector["chain_digest"].as_str().unwrap()
            );
            assert_eq!(
                hex::encode(instruction.canonical_transcript().unwrap()),
                vector["signed_transcript_hex"].as_str().unwrap()
            );
            assert_eq!(
                instruction.event_digest().unwrap(),
                vector["event_digest"].as_str().unwrap()
            );
            let verified = instruction.verify(&issuers).unwrap();
            if let Some(previous) = &previous {
                verified.verify_successor_of(previous).unwrap();
            }
            let mut unsigned = instruction.clone();
            unsigned.instruction_material_digest.clear();
            unsigned.chain_digest.clear();
            unsigned.issuer_key_id.clear();
            unsigned.issuer_signature.clear();
            assert_eq!(
                sign_reset_clear_required(unsigned, &fixture_pairing_issuer()).unwrap(),
                instruction
            );
            previous = Some(verified);
        }

        let mut tampered: MapleResetClearRequiredV1 = serde_json::from_value(
            fixture()["reset_clear_three_reset_chain"][2]["instruction"].clone(),
        )
        .unwrap();
        tampered.cumulative_reset_count -= 1;
        assert!(matches!(
            tampered.verify(&issuers),
            Err(Error::InvalidResponse(_))
        ));

        let predecessor: MapleResetClearRequiredV1 = serde_json::from_value(
            fixture()["reset_clear_three_reset_chain"][1]["instruction"].clone(),
        )
        .unwrap();
        let predecessor = predecessor.verify(&issuers).unwrap();
        let mut changed_endpoint: MapleResetClearRequiredV1 = serde_json::from_value(
            fixture()["reset_clear_three_reset_chain"][2]["instruction"].clone(),
        )
        .unwrap();
        changed_endpoint.host.endpoint_epoch += 1;
        let changed_endpoint =
            sign_reset_clear_required(changed_endpoint, &fixture_pairing_issuer()).unwrap();
        let changed_endpoint = changed_endpoint.verify(&issuers).unwrap();
        assert!(matches!(
            changed_endpoint.verify_successor_of(&predecessor),
            Err(Error::InvalidResponse(_))
        ));
    }

    #[test]
    fn registration_sync_vectors_are_signed_request_bound_authority() {
        let issuers = trusted_issuers();
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
            let request: crate::RegisterMapleDeviceRequest = fixture_value(request_name);
            let transcript = request.canonical_transcript().unwrap();
            assert_eq!(hex::encode(&transcript), fixture_string(transcript_name));
            assert_eq!(
                BASE64.encode(Sha256::digest(&transcript)),
                fixture_string(digest_name)
            );
            request.validate().unwrap();
        }

        for (request_name, response_name, expected_status, has_reset_plan) in [
            (
                "register_device_request_epoch_1",
                "register_device_response_ready",
                MapleRevocationSyncStatusV1::Ready,
                false,
            ),
            (
                "register_device_request_epoch_4",
                "register_device_response_reset_clear_required",
                MapleRevocationSyncStatusV1::ResetClearRequired,
                true,
            ),
            (
                "register_device_request_epoch_1",
                "register_device_response_revocations_pending",
                MapleRevocationSyncStatusV1::RevocationsPending,
                false,
            ),
        ] {
            let request: crate::RegisterMapleDeviceRequest = fixture_value(request_name);
            let response: crate::MapleDeviceRegistrationResponse = fixture_value(response_name);
            request.validate().unwrap();
            assert_eq!(response.protocol_version, MAPLE_PAIRING_PROTOCOL_VERSION);
            assert_eq!(response.operation_id, request.operation_id);
            assert_eq!(response.device_id, request.device_id);
            assert_eq!(response.security_epoch, request.known_security_epoch);
            let sync = response
                .revocation_sync
                .verify_against_registration(
                    &request,
                    response.registration_id,
                    response.security_epoch,
                    &issuers,
                )
                .unwrap();
            assert_eq!(sync.as_inner().status, expected_status);
            let verified = VerifiedRegisterMapleDeviceResponseV1::new(response, sync);
            assert_eq!(verified.has_reset_clear_plan(), has_reset_plan);
        }

        let request: crate::RegisterMapleDeviceRequest =
            fixture_value("register_device_request_epoch_4");
        let mut response: crate::MapleDeviceRegistrationResponse =
            fixture_value("register_device_response_reset_clear_required");
        response.security_epoch -= 1;
        assert!(matches!(
            response.revocation_sync.verify_against_registration(
                &request,
                response.registration_id,
                response.security_epoch,
                &issuers,
            ),
            Err(Error::InvalidResponse(_))
        ));
    }

    #[test]
    fn every_frozen_v6_checkpoint_request_and_epoch_vector_is_consumed() {
        let issuers = trusted_issuers();
        let pending_sync: MapleRevocationSyncV1 = fixture_value("reset_clear_pending_sync");
        let host_key =
            claim_signing_key(&pending_sync.reset_clear_instruction.as_ref().unwrap().host);

        for (request_name, transcript_name, digest_name) in [
            (
                "reset_clear_list_revocations_request",
                "reset_clear_list_revocations_request_transcript_hex",
                "reset_clear_list_revocations_request_digest",
            ),
            (
                "reset_clear_historical_later_request",
                "reset_clear_historical_later_request_transcript_hex",
                "reset_clear_historical_later_request_digest",
            ),
        ] {
            let request: ListMaplePairingRevocationsRequest = fixture_value(request_name);
            let transcript = request.canonical_transcript().unwrap();
            assert_eq!(hex::encode(&transcript), fixture_string(transcript_name));
            assert_eq!(sha256_base64(&transcript), fixture_string(digest_name));
            request.validate_with_signing_key(&host_key).unwrap();
        }

        for (checkpoint_name, transcript_name, digest_name) in [
            (
                "reset_clear_pending_checkpoint",
                "reset_clear_pending_checkpoint_transcript_hex",
                "reset_clear_pending_checkpoint_digest",
            ),
            (
                "reset_clear_acked_checkpoint",
                "reset_clear_acked_checkpoint_transcript_hex",
                "reset_clear_acked_checkpoint_digest",
            ),
            (
                "reset_clear_later_checkpoint",
                "reset_clear_later_checkpoint_transcript_hex",
                "reset_clear_later_checkpoint_digest",
            ),
        ] {
            let checkpoint: MapleRevocationStreamCheckpointV1 = fixture_value(checkpoint_name);
            let transcript = checkpoint.canonical_transcript().unwrap();
            assert_eq!(hex::encode(&transcript), fixture_string(transcript_name));
            assert_eq!(
                checkpoint.transcript_digest().unwrap(),
                fixture_string(digest_name)
            );
            checkpoint.verify(&issuers).unwrap();
        }

        let later_revocation: MaplePairRevocationV1 =
            fixture_value("reset_clear_later_pair_revocation");
        assert_eq!(
            hex::encode(later_revocation.canonical_transcript().unwrap()),
            fixture_string("reset_clear_later_pair_revocation_transcript_hex")
        );
        assert_eq!(
            later_revocation.transcript_digest().unwrap(),
            fixture_string("reset_clear_later_pair_revocation_digest")
        );
        later_revocation.verify_issuer(&issuers).unwrap();

        let ack: AckMaplePairingRevocationRequest = fixture_value("reset_clear_ack_request");
        let ack_transcript = ack.canonical_transcript().unwrap();
        assert_eq!(
            hex::encode(&ack_transcript),
            fixture_string("reset_clear_ack_request_transcript_hex")
        );
        assert_eq!(
            sha256_base64(&ack_transcript),
            fixture_string("reset_clear_ack_request_digest")
        );
        ack.validate_with_signing_key(&host_key).unwrap();

        let device_list: crate::MapleDeviceListResponse =
            fixture_value("list_devices_response_security_epoch");
        assert_eq!(device_list.protocol_version, MAPLE_PAIRING_PROTOCOL_VERSION);
        assert_eq!(device_list.security_epoch, 4);
        assert!(device_list.devices.is_empty());
        assert!(device_list.next_cursor.is_none());
        assert!(!device_list.has_more);
        let device_list_json = serde_json::to_value(&device_list).unwrap();
        assert!(device_list_json.get("signature").is_none());
        assert!(device_list_json.get("revocation_sync").is_none());

        for vector in fixture()["security_epoch_outcome_vectors"]
            .as_array()
            .unwrap()
        {
            let known = vector["known_security_epoch"].as_u64().unwrap();
            let current = vector["current_security_epoch"].as_u64().unwrap();
            let reset_pending = vector["pending_reset_clear"].as_bool().unwrap();
            let expected_outcome = if known < current {
                "MapleSecurityEpochStale"
            } else if known > current {
                "conflict"
            } else if reset_pending {
                "reset_clear_required"
            } else {
                "ready"
            };
            assert_eq!(
                vector["registration_operation_accepted"].as_bool().unwrap(),
                known == current
            );
            assert_eq!(vector["outcome"].as_str().unwrap(), expected_outcome);
        }

        let tombstones = &fixture()["registration_operation_tombstone_vectors"];
        assert_eq!(
            tombstones["storage"]["operation_lookup_digest"],
            "hmac_only"
        );
        assert_eq!(tombstones["storage"]["raw_operation_id_stored"], false);
        assert_eq!(tombstones["exact_old_request"], "frozen_ready_replay");
        assert_eq!(
            tombstones["changed_request_same_operation_lookup"],
            "conflict"
        );

        let structural = &fixture()["wire_structural_assertions"];
        assert_eq!(
            structural["list_devices_security_epoch_source"],
            "authenticated_encrypted_account_authority_head"
        );
        assert_eq!(structural["list_devices_response_signed"], false);
        assert_eq!(
            structural["pending_reset_checkpoint"],
            serde_json::json!({
                "last_issued_issuer_sequence": 1,
                "last_acked_issuer_sequence": 0,
            })
        );
        assert_eq!(structural["historical_reset_event_allowed_after_ack"], true);
        assert_eq!(
            structural["historical_reset_event_allowed_when_later_events_exist"],
            true
        );
        assert_eq!(
            structural["reset_admission_public_shape"],
            serde_json::json!(["admission_count", "admission_set_digest"])
        );
        assert_eq!(structural["retained_admission_leaves_cross_wire"], false);

        let instruction: MapleResetClearRequiredV1 = serde_json::from_value(
            fixture()["reset_clear_three_reset_chain"][2]["instruction"].clone(),
        )
        .unwrap();
        let event = MapleRevocationStreamEventV1::ResetClearRequired(instruction.clone());
        let event_json = serde_json::to_string(&event).unwrap();
        assert!(event_json.starts_with("{\"event_type\":\"reset_clear_required\",\"event\":"));
        assert_eq!(
            structural["reset_event_json_key_order"],
            serde_json::json!(["event_type", "event"])
        );
        let instruction_json = serde_json::to_value(instruction).unwrap();
        assert!(instruction_json.get("admission_count").is_some());
        assert!(instruction_json.get("admission_set_digest").is_some());
        assert!(instruction_json.get("admission_leaves").is_none());
        assert!(instruction_json.get("admissions").is_none());
    }

    #[test]
    fn v6_authority_debug_is_fully_redacted_before_and_after_verification() {
        let issuers = trusted_issuers();
        let instruction: MapleResetClearRequiredV1 = serde_json::from_value(
            fixture()["reset_clear_three_reset_chain"][2]["instruction"].clone(),
        )
        .unwrap();
        let checkpoint: MapleRevocationStreamCheckpointV1 =
            fixture_value("reset_clear_pending_checkpoint");
        let sync: MapleRevocationSyncV1 = fixture_value("reset_clear_pending_sync");
        let event = MapleRevocationStreamEventV1::ResetClearRequired(instruction.clone());
        let list_request: ListMaplePairingRevocationsRequest =
            fixture_value("reset_clear_list_revocations_request");
        let list: MaplePairingRevocationListResponse =
            fixture_value("reset_clear_list_revocations_response");
        let ack_request: AckMaplePairingRevocationRequest =
            fixture_value("reset_clear_ack_request");
        let ack_response: MaplePairingRevocationAckResponse =
            fixture_value("reset_clear_ack_response");
        let register_request: crate::RegisterMapleDeviceRequest =
            fixture_value("register_device_request_epoch_4");
        let register_response: crate::MapleDeviceRegistrationResponse =
            fixture_value("register_device_response_reset_clear_required");

        let verified_list = list.clone().verify_for(&list_request, &issuers).unwrap();
        let verified_event = verified_list.events[0].clone();
        let verified_ack = ack_response
            .clone()
            .verify_for(&ack_request, &issuers)
            .unwrap();
        let verified_sync = register_response
            .revocation_sync
            .verify_against_registration(
                &register_request,
                register_response.registration_id,
                register_response.security_epoch,
                &issuers,
            )
            .unwrap();
        let verified_register =
            VerifiedRegisterMapleDeviceResponseV1::new(register_response.clone(), verified_sync);

        let forbidden_values = [
            instruction.event_id.to_string(),
            instruction.reset_id.to_string(),
            instruction.subject_account_id.to_string(),
            instruction.subject_project_id.to_string(),
            instruction.recipient_host_registration_id.to_string(),
            instruction.host.device_id.to_string(),
            instruction.host.installation_id.to_string(),
            instruction.host.identity_public_key.clone(),
            instruction.host.endpoint_id.clone(),
            instruction.source_revocation_stream_id.to_string(),
            instruction.revocation_stream_id.to_string(),
            instruction.admission_set_digest.clone(),
            instruction.instruction_material_digest.clone(),
            instruction.chain_digest.clone(),
            instruction.issuer_key_id.clone(),
            instruction.issuer_signature.clone(),
            ack_request.operation_id.to_string(),
            ack_request.event_digest.clone(),
            ack_request.signature.clone(),
            list.query_id.to_string(),
            register_response.operation_id.to_string(),
            register_response.registration_id.to_string(),
            register_response.device_id.to_string(),
        ];
        let forbidden_fields = [
            "operation_id",
            "query_id",
            "registration_id",
            "device_id",
            "installation_id",
            "identity_public_key",
            "endpoint_id",
            "revocation_stream_id",
            "event_id",
            "reset_id",
            "event_digest",
            "admission_set_digest",
            "instruction_material_digest",
            "chain_digest",
            "issuer_key_id",
            "issuer_signature",
            "next_after_issuer_sequence",
        ];
        for debug in [
            format!("{instruction:?}"),
            format!("{checkpoint:?}"),
            format!("{sync:?}"),
            format!("{event:?}"),
            format!("{list:?}"),
            format!("{ack_request:?}"),
            format!("{ack_response:?}"),
            format!("{register_response:?}"),
            format!("{verified_event:?}"),
            format!("{verified_list:?}"),
            format!("{verified_ack:?}"),
            format!("{verified_register:?}"),
        ] {
            assert!(debug.contains("[redacted]"));
            for secret in &forbidden_values {
                assert!(!debug.contains(secret), "Debug leaked authority value");
            }
            for field in forbidden_fields {
                assert!(!debug.contains(field), "Debug leaked authority field");
            }
        }
    }

    #[test]
    fn opaque_authority_wrapper_debug_never_delegates_to_inner_wire_dtos() {
        let issuers = trusted_issuers();
        let ticket: MaplePairRequestTicketV1 = fixture_value("request_ticket");
        let verified_ticket = ticket
            .verify_at(
                &issuers,
                ticket.created_at_unix_ms,
                MAPLE_PAIRING_MAX_CLOCK_SKEW_MS,
                MAPLE_PAIRING_MAX_TICKET_TTL_MS,
            )
            .unwrap();
        let authorization: MaplePairAuthorizationV1 = fixture_value("pair_authorization");
        let verified_authorization = authorization
            .verify_against_ticket(&issuers, &ticket)
            .unwrap();
        let revocation: MaplePairRevocationV1 = fixture_value("pair_revocation");
        let verified_revocation = revocation.verify_issuer(&issuers).unwrap();
        let status_response: MaplePairingStatusResponse = fixture_value("pairing_status_response");
        let status_request: MaplePairingStatusRequest = fixture_value("pairing_status_request");
        let verified_status = status_response
            .verify_for(
                &status_request,
                MaplePairingRole::Controller,
                &issuers,
                ticket.created_at_unix_ms,
            )
            .unwrap();
        let checkpoint =
            fixture_value::<MapleRevocationStreamCheckpointV1>("revocation_stream_checkpoint")
                .verify(&issuers)
                .unwrap();
        let reconciled = checkpoint
            .clone()
            .plan_durable_transition(None)
            .unwrap()
            .after_durable_commit();
        let reset: MapleResetClearRequiredV1 = serde_json::from_value(
            fixture()["reset_clear_three_reset_chain"][2]["instruction"].clone(),
        )
        .unwrap();
        let verified_reset = reset.verify(&issuers).unwrap();
        let pending_checkpoint =
            fixture_value::<MapleRevocationStreamCheckpointV1>("reset_clear_pending_checkpoint")
                .verify(&issuers)
                .unwrap();
        let latest_reset = reset
            .verify_against_checkpoint(&issuers, &pending_checkpoint)
            .unwrap();

        let forbidden = [
            ticket.pairing_request_id.to_string(),
            ticket.pair_id.to_string(),
            ticket.pairing_request_nonce.clone(),
            ticket.controller.identity_public_key.clone(),
            ticket.host.endpoint_id.clone(),
            ticket.controller_request_digest.clone(),
            ticket.issuer_signature.clone(),
            authorization.host_approval_nonce.clone(),
            authorization.host_approval_digest.clone(),
            authorization.issuer_signature.clone(),
            revocation.event_id.to_string(),
            revocation.pair_authorization_digest.clone(),
            revocation.issuer_signature.clone(),
            reset.event_id.to_string(),
            reset.instruction_material_digest.clone(),
            reset.chain_digest.clone(),
        ];
        for debug in [
            format!("{verified_ticket:?}"),
            format!("{verified_authorization:?}"),
            format!("{verified_revocation:?}"),
            format!("{verified_status:?}"),
            format!("{checkpoint:?}"),
            format!("{reconciled:?}"),
            format!("{verified_reset:?}"),
            format!("{latest_reset:?}"),
        ] {
            assert!(debug.contains("[redacted]"));
            for secret in &forbidden {
                assert!(
                    !debug.contains(secret),
                    "opaque wrapper Debug leaked inner DTO"
                );
            }
        }
    }

    #[test]
    fn reset_clear_pending_and_historical_pages_have_distinct_authority() {
        let issuers = trusted_issuers();
        let pending_request: ListMaplePairingRevocationsRequest =
            fixture_value("reset_clear_list_revocations_request");
        let pending_response: MaplePairingRevocationListResponse =
            fixture_value("reset_clear_list_revocations_response");
        let pending = pending_response
            .verify_for(&pending_request, &issuers)
            .unwrap();
        assert!(pending.has_reset_clear_instruction());
        assert!(matches!(
            pending.events.as_slice(),
            [VerifiedMapleRevocationStreamEventV1::ResetClearRequired(_)]
        ));

        let acked_history: MaplePairingRevocationListResponse =
            fixture_value("reset_clear_historical_acked_response");
        let acked = acked_history
            .verify_for(&pending_request, &issuers)
            .unwrap();
        assert!(!acked.has_reset_clear_instruction());
        assert!(matches!(
            acked.events.as_slice(),
            [VerifiedMapleRevocationStreamEventV1::ResetClearRequired(_)]
        ));

        let later_request: ListMaplePairingRevocationsRequest =
            fixture_value("reset_clear_historical_later_request");
        let later_response: MaplePairingRevocationListResponse =
            fixture_value("reset_clear_historical_later_response");
        let later = later_response
            .clone()
            .verify_for(&later_request, &issuers)
            .unwrap();
        assert!(!later.has_reset_clear_instruction());
        assert!(matches!(
            later.events.as_slice(),
            [
                VerifiedMapleRevocationStreamEventV1::ResetClearRequired(_),
                VerifiedMapleRevocationStreamEventV1::PairRevocation(_)
            ]
        ));

        let mut too_small = later_request.clone();
        too_small.limit = Some(1);
        assert!(matches!(
            later_response.verify_for(&too_small, &issuers),
            Err(Error::InvalidResponse(_))
        ));

        // Even when every nested artifact remains issuer-signed, a sync that
        // relabels the unacknowledged reset head as an ordinary pending
        // revocation must fail closed rather than downgrading it to history.
        let mut downgraded_pending: MaplePairingRevocationListResponse =
            fixture_value("reset_clear_list_revocations_response");
        downgraded_pending.revocation_sync.status = MapleRevocationSyncStatusV1::RevocationsPending;
        downgraded_pending.revocation_sync.reset_clear_instruction = None;
        assert!(matches!(
            downgraded_pending.verify_for(&pending_request, &issuers),
            Err(Error::InvalidResponse(_))
        ));
    }

    #[test]
    fn reset_clear_ack_requires_successful_durable_full_scope_callback() {
        let issuers = trusted_issuers();
        let request: ListMaplePairingRevocationsRequest =
            fixture_value("reset_clear_list_revocations_request");
        let response: MaplePairingRevocationListResponse =
            fixture_value("reset_clear_list_revocations_response");
        let failure_latest = response
            .clone()
            .verify_for(&request, &issuers)
            .unwrap()
            .into_reset_clear_instruction()
            .unwrap();
        let failure = failure_latest.after_durable_full_scope_admission_clear(|_| {
            Err(Error::Io(std::io::Error::other("durable commit failed")))
        });
        assert!(matches!(failure, Err(Error::Io(_))));

        let expected_sync: MapleRevocationSyncV1 = fixture_value("reset_clear_pending_sync");
        let expected_instruction = expected_sync.reset_clear_instruction.unwrap();
        // A failed callback consumes the linear capability. Re-fetching and
        // verifying current signed state is required before another attempt.
        let latest = response
            .verify_for(&request, &issuers)
            .unwrap()
            .into_reset_clear_instruction()
            .unwrap();
        let cleared = latest
            .after_durable_full_scope_admission_clear(|instruction| {
                assert_eq!(instruction, &expected_instruction);
                Ok(())
            })
            .unwrap();
        let literal: AckMaplePairingRevocationRequest = fixture_value("reset_clear_ack_request");
        let prepared = AckMaplePairingRevocationRequest::unsigned_v1_after_durable_reset_clear(
            literal.operation_id,
            cleared,
        )
        .unwrap();
        let exact_retry = prepared.clone();
        assert_eq!(exact_retry.as_inner(), prepared.as_inner());
        let ack_response: MaplePairingRevocationAckResponse =
            fixture_value("reset_clear_ack_response");
        ack_response
            .clone()
            .verify_for(&literal, &trusted_issuers())
            .unwrap();
        let mut zero_time = ack_response.clone();
        zero_time.accepted_at_unix_ms = 0;
        zero_time.verify_for(&literal, &trusted_issuers()).unwrap();
        let mut negative_time = ack_response;
        negative_time.accepted_at_unix_ms = -1;
        assert!(matches!(
            negative_time.verify_for(&literal, &trusted_issuers()),
            Err(Error::InvalidResponse(_))
        ));
        assert_eq!(
            prepared.as_inner(),
            &AckMaplePairingRevocationRequest {
                signature: String::new(),
                ..literal
            }
        );
    }

    #[test]
    fn ack_receipt_does_not_establish_current_readiness() {
        let issuers = trusted_issuers();
        let request: AckMaplePairingRevocationRequest = fixture_value("reset_clear_ack_request");
        let response: MaplePairingRevocationAckResponse = fixture_value("reset_clear_ack_response");
        let receipt = response.verify_for(&request, &issuers).unwrap();
        assert_eq!(receipt.operation_id(), request.operation_id);
        assert_eq!(receipt.event_id(), request.event_id);
        assert_eq!(
            receipt.last_acked_issuer_sequence(),
            request.issuer_sequence
        );

        // An exact retry may still return the receipt above after the signed
        // current head has advanced. Only this separately fetched and verified
        // sync can establish current readiness.
        let later_request: ListMaplePairingRevocationsRequest =
            fixture_value("reset_clear_historical_later_request");
        let later_response: MaplePairingRevocationListResponse =
            fixture_value("reset_clear_historical_later_response");
        let current = later_response.verify_for(&later_request, &issuers).unwrap();
        assert_eq!(
            current.revocation_sync.as_inner().status,
            MapleRevocationSyncStatusV1::RevocationsPending
        );
        assert_eq!(
            current
                .revocation_sync
                .checkpoint()
                .as_inner()
                .last_issued_issuer_sequence,
            2
        );
        assert_eq!(
            current
                .revocation_sync
                .checkpoint()
                .as_inner()
                .last_acked_issuer_sequence,
            receipt.last_acked_issuer_sequence()
        );
    }
}
