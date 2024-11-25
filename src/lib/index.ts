// Export types from api
export type {
  KVListItem,
  LoginResponse,
  UserResponse,
  GithubAuthResponse,
  GoogleAuthResponse
} from "./api";

// Export the provider and context
export { OpenSecretProvider, OpenSecretContext } from "./main";

// Export the hook
export { useOpenSecret } from "./context";

// Export types needed by consumers
export type { OpenSecretAuthState, OpenSecretContextType } from "./main";
export type { AttestationDocument } from "./attestation";
export type { ParsedAttestationView } from "./attestationForView";
export type { PcrConfig, Pcr0ValidationResult } from "./pcr";

// Export crypto utilities
// TODO: these can actually just be used internally by the password reset function
export { generateSecureSecret, hashSecret } from "./crypto";
