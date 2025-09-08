// Export types from api
export type {
  KVListItem,
  LoginResponse,
  UserResponse,
  GithubAuthResponse,
  GoogleAuthResponse,
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
  ConversationDeleteResponse
} from "./api";

// Export API key management functions
export { createApiKey, listApiKeys, deleteApiKey } from "./api";

// Export AI customization options
export { createCustomFetch, type CustomFetchOptions } from "./ai";

// Re-export Model type from OpenAI for convenience
export type { Model } from "openai/resources/models.js";

// Export API configuration
export { apiConfig, type ApiContext, type ApiEndpoint } from "./apiConfig";

// Export the provider and context
export { OpenSecretProvider, OpenSecretContext } from "./main";
export { OpenSecretDeveloper, OpenSecretDeveloperContext } from "./developer";

// Export the hooks
export { useOpenSecret } from "./context";
export { useOpenSecretDeveloper } from "./developerContext";

// Export types needed by consumers
export type { OpenSecretAuthState, OpenSecretContextType } from "./main";
export type {
  OpenSecretDeveloperAuthState,
  OpenSecretDeveloperContextType,
  DeveloperRole,
  OrganizationDetails,
  ProjectDetails,
  ProjectSettings
} from "./developer";
export type { AttestationDocument } from "./attestation";
export type { ParsedAttestationView } from "./attestationForView";
export type { PcrConfig, Pcr0ValidationResult } from "./pcr";

// Export crypto utilities
// TODO: these can actually just be used internally by the password reset function
export { generateSecureSecret, hashSecret } from "./crypto";
