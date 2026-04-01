// Export types from api
export type {
  KVListItem,
  LoginResponse,
  UserResponse,
  GithubAuthResponse,
  GoogleAuthResponse,
  AppleAuthResponse,
  DocumentResponse,
  DocumentUploadInitResponse,
  DocumentStatusRequest,
  DocumentStatusResponse,
  ApiKey,
  ApiKeyCreateResponse,
  ApiKeyListResponse,
  ResponsesRetrieveResponse,
  ResponsesListResponse,
  ResponsesListParams,
  ResponsesCancelResponse,
  ResponsesDeleteResponse,
  ResponsesCreateRequest,
  Conversation,
  ConversationItem,
  ConversationCreateRequest,
  ConversationUpdateRequest,
  ConversationItemsResponse,
  ConversationsListResponse,
  ConversationDeleteResponse,
  ConversationsDeleteResponse,
  BatchDeleteConversationsRequest,
  BatchDeleteItemResult,
  BatchDeleteConversationsResponse,
  ThirdPartyTokenResponse,
  RefreshResponse,
  PrivateKeyResponse,
  PrivateKeyBytesResponse,
  PublicKeyResponse,
  SignMessageResponse,
  EncryptDataResponse,
  AppleUser,
} from './api';

// Export API key management functions
export { createApiKey, listApiKeys, deleteApiKey } from './api';

// Export AI customization options
export { createCustomFetch, type CustomFetchOptions } from './ai';

// Re-export Model type from OpenAI for convenience
export type { Model } from 'openai/resources/models.js';

// Export configuration functions
export { configure, getConfig, isConfigured, resetConfig } from './config';
export type { OpenSecretConfig } from './config';

// Export storage abstraction
export { getStorage } from './storage';
export type { StorageProvider } from './storage';

// Export API configuration
export { apiConfig, type ApiContext, type ApiEndpoint } from './apiConfig';

// Export all API functions directly
export {
  // Authentication
  fetchLogin as signIn,
  fetchSignUp as signUp,
  fetchGuestLogin as signInGuest,
  fetchGuestSignUp as signUpGuest,
  fetchLogout as signOut,
  convertGuestToEmailAccount as convertGuestToUserAccount,
  // User management
  fetchUser,
  refreshToken as refreshAccessToken,
  verifyEmail,
  requestNewVerificationCode,
  changePassword,
  requestPasswordReset,
  confirmPasswordReset,
  requestAccountDeletion,
  confirmAccountDeletion,
  // OAuth
  initiateGitHubAuth,
  handleGitHubCallback,
  initiateGoogleAuth,
  handleGoogleCallback,
  initiateAppleAuth,
  handleAppleCallback,
  handleAppleNativeSignIn,
  // Key-value storage
  fetchGet as get,
  fetchPut as put,
  fetchList as list,
  fetchDelete as del,
  // Cryptographic operations
  fetchPrivateKey as getPrivateKey,
  fetchPrivateKeyBytes as getPrivateKeyBytes,
  fetchPublicKey as getPublicKey,
  signMessage,
  encryptData,
  decryptData,
  // Third-party tokens
  generateThirdPartyToken,
  // AI
  fetchModels,
  // Document processing
  uploadDocument,
  checkDocumentStatus,
  uploadDocumentWithPolling,
  // Utility
  getApiUrl,
} from './api';

// Export AI custom fetch
export { createCustomFetch as createAiCustomFetch } from './ai';

// Export attestation functions
export { getAttestation } from './getAttestation';
export { authenticate } from './attestation';
export {
  parseAttestationForView,
  AWS_ROOT_CERT_DER as awsRootCertDer,
  EXPECTED_ROOT_CERT_HASH as expectedRootCertHash,
} from './attestationForView';

// Export the provider and context
export { OpenSecretProvider, OpenSecretContext } from './main';
export { OpenSecretDeveloper, OpenSecretDeveloperContext } from './developer';

// Export the hooks
export { useOpenSecret } from './context';
export { useOpenSecretDeveloper } from './developerContext';

// Export types needed by consumers
export type { OpenSecretAuthState, OpenSecretContextType } from './main';
export type {
  OpenSecretDeveloperAuthState,
  OpenSecretDeveloperContextType,
  DeveloperRole,
  OrganizationDetails,
  ProjectDetails,
  ProjectSettings,
} from './developer';
export type { AttestationDocument } from './attestation';
export type { ParsedAttestationView } from './attestationForView';
export type { PcrConfig, Pcr0ValidationResult } from './pcr';

// Export crypto utilities
// TODO: these can actually just be used internally by the password reset function
export { generateSecureSecret, hashSecret } from './crypto';
