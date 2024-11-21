// Export types from api
export type {
  KVListItem,
  LoginResponse,
  UserResponse,
  GithubAuthResponse,
  GoogleAuthResponse
} from "./api";

// Export types and components from main
export type { OpenSecretAuthState, OpenSecretContextType } from "./main";

// Export the provider and context
export { OpenSecretProvider, OpenSecretContext } from "./main";

// Export the hook
export { useOpenSecret } from "./context";

// Export key functions that might be useful
export { setApiUrl } from "./api";

// Export getAttestation
export { getAttestation } from "./getAttestation";

export { authenticate } from "./attestation";
export type { AttestationDocument } from "./attestation";

// Export attestationForView
export { parseAttestationForView } from "./attestationForView";
export type { ParsedAttestationView } from "./attestationForView";
export { EXPECTED_ROOT_CERT_HASH, VALID_PCR0_VALUES, VALID_PCR0_VALUES_DEV } from "./attestationForView";

// Export crypto stuff
export { generateSecureSecret, hashSecret } from "./crypto";
