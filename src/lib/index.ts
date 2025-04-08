// Export types from api
export type {
  KVListItem,
  LoginResponse,
  UserResponse,
  GithubAuthResponse,
  GoogleAuthResponse
} from "./api";

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
export { validatePcrSet } from "./attestationForView";
export type { PcrConfig, Pcr0ValidationResult } from "./pcr";
export { validatePcr0Hash } from "./pcr";
export type { PcrEntry } from "./pcrHistory";
export { validatePcrAgainstHistory, getPcrHistoryList } from "./pcrHistory";

// Import necessary functions for the validator
import { validatePcr0Hash as validPcr0 } from "./pcr";
import { validatePcrSet as validPcrSet } from "./attestationForView";
import { validatePcrAgainstHistory as validPcrAgainstHistory } from "./pcrHistory";

// High-level PCR validation interface
export const PcrValidator = {
  /**
   * Validates PCR0 against known good values (local and optionally remote)
   */
  validatePcr0: validPcr0,

  /**
   * Validates a PCR set (PCR0/PCR1/PCR2) - for remote attestation verification
   */
  validatePcrSet: validPcrSet,

  /**
   * Validates PCRs against signed history records
   */
  validateAgainstHistory: validPcrAgainstHistory
};

// Export crypto utilities
// TODO: these can actually just be used internally by the password reset function
export { generateSecureSecret, hashSecret } from "./crypto";
