import React, { createContext, useState, useEffect } from "react";
import * as platformApi from "./platformApi";
import { setPlatformApiUrl } from "./platformApi";
import { getAttestation } from "./getAttestation";
import { authenticate } from "./attestation";
import {
  parseAttestationForView,
  AWS_ROOT_CERT_DER,
  EXPECTED_ROOT_CERT_HASH,
  ParsedAttestationView
} from "./attestationForView";
import type { AttestationDocument } from "./attestation";
import { PcrConfig } from "./pcr";
import type {
  Organization,
  Project,
  ProjectSecret,
  ProjectSettings,
  EmailSettings,
  OAuthSettings,
  OrganizationMember,
  PlatformOrg,
  PlatformUser,
  OrganizationInvite
} from "./platformApi";

export type DeveloperRole = "owner" | "admin" | "developer" | "viewer";

export type OrganizationDetails = Organization;

export type ProjectDetails = Project;

export { type ProjectSettings };

export type DeveloperResponse = PlatformUser & { organizations: PlatformOrg[] };

export type OpenSecretDeveloperAuthState = {
  loading: boolean;
  developer?: DeveloperResponse;
};

export type OpenSecretDeveloperContextType = {
  auth: OpenSecretDeveloperAuthState;

  /**
   * Signs in a developer with email and password
   * @param email - Developer's email address
   * @param password - Developer's password
   * @returns A promise that resolves to the login response with access and refresh tokens
   *
   * @description
   * - Calls the login API endpoint
   * - Stores access_token and refresh_token in localStorage
   * - Updates the developer state with user information
   * - Throws an error if authentication fails
   */
  signIn: (email: string, password: string) => Promise<platformApi.PlatformLoginResponse>;

  /**
   * Registers a new developer account
   * @param email - Developer's email address
   * @param password - Developer's password
   * @param name - Optional developer name
   * @returns A promise that resolves to the login response with access and refresh tokens
   *
   * @description
   * - Calls the registration API endpoint
   * - Stores access_token and refresh_token in localStorage
   * - Updates the developer state with new user information
   * - Throws an error if account creation fails
   */
  signUp: (
    email: string,
    password: string,
    name?: string
  ) => Promise<platformApi.PlatformLoginResponse>;

  /**
   * Signs out the current developer by removing authentication tokens
   *
   * @description
   * - Calls the logout API endpoint with the current refresh_token
   * - Removes access_token, refresh_token from localStorage
   * - Resets the developer state to show no user is authenticated
   */
  signOut: () => Promise<void>;

  /**
   * Refreshes the developer's authentication state
   * @returns A promise that resolves when the refresh is complete
   * @throws {Error} If the refresh fails
   *
   * @description
   * - Retrieves the latest developer information from the server
   * - Updates the developer state with fresh data
   * - Useful after making changes that affect developer profile or organization membership
   */
  refetchDeveloper: () => Promise<void>;

  /**
   * Additional PCR0 hashes to validate against
   */
  pcrConfig: PcrConfig;

  /**
   * Gets attestation from the enclave
   */
  getAttestation: typeof getAttestation;

  /**
   * Authenticates an attestation document
   */
  authenticate: typeof authenticate;

  /**
   * Parses an attestation document for viewing
   */
  parseAttestationForView: (
    document: AttestationDocument,
    cabundle: Uint8Array[],
    pcrConfig?: PcrConfig
  ) => Promise<ParsedAttestationView>;

  /**
   * AWS root certificate in DER format
   */
  awsRootCertDer: typeof AWS_ROOT_CERT_DER;

  /**
   * Expected hash of the AWS root certificate
   */
  expectedRootCertHash: typeof EXPECTED_ROOT_CERT_HASH;

  /**
   * Gets and verifies an attestation document from the enclave
   * @returns A promise resolving to the parsed attestation document
   * @throws {Error} If attestation fails or is invalid
   *
   * @description
   * This is a convenience function that:
   * 1. Fetches the attestation document with a random nonce
   * 2. Authenticates the document
   * 3. Parses it for viewing
   */
  getAttestationDocument: () => Promise<ParsedAttestationView>;

  /**
   * Creates a new organization
   * @param name - Organization name
   * @returns A promise that resolves to the created organization
   */
  createOrganization: (name: string) => Promise<Organization>;

  /**
   * Lists all organizations the developer has access to
   * @returns A promise resolving to array of organization details
   */
  listOrganizations: () => Promise<Organization[]>;

  /**
   * Deletes an organization (requires owner role)
   * @param orgId - Organization ID
   */
  deleteOrganization: (orgId: string) => Promise<void>;

  /**
   * Creates a new project within an organization
   * @param orgId - Organization ID
   * @param name - Project name
   * @param description - Optional project description
   * @returns A promise that resolves to the project details including client ID
   */
  createProject: (orgId: string, name: string, description?: string) => Promise<Project>;

  /**
   * Lists all projects within an organization
   * @param orgId - Organization ID
   * @returns A promise resolving to array of project details
   */
  listProjects: (orgId: string) => Promise<Project[]>;

  /**
   * Gets a single project by ID
   * @param orgId - Organization ID
   * @param projectId - Project ID
   * @returns A promise resolving to the project details
   */
  getProject: (orgId: string, projectId: string) => Promise<Project>;

  /**
   * Updates project details
   * @param orgId - Organization ID
   * @param projectId - Project ID
   * @param updates - Object containing fields to update
   */
  updateProject: (
    orgId: string,
    projectId: string,
    updates: { name?: string; description?: string; status?: string }
  ) => Promise<Project>;

  /**
   * Deletes a project
   * @param orgId - Organization ID
   * @param projectId - Project ID
   */
  deleteProject: (orgId: string, projectId: string) => Promise<void>;

  /**
   * Creates a new secret for a project
   * @param orgId - Organization ID
   * @param projectId - Project ID
   * @param keyName - Secret key name (must be alphanumeric)
   * @param secret - Secret value (must be base64 encoded by the caller)
   *
   * Example:
   * ```typescript
   * // To encode a string secret
   * import { encode } from "@stablelib/base64";
   * const encodedSecret = encode(new TextEncoder().encode("my-secret-value"));
   *
   * // Now pass the encoded secret to the function
   * createProjectSecret(orgId, projectId, "mySecretKey", encodedSecret);
   * ```
   */
  createProjectSecret: (
    orgId: string,
    projectId: string,
    keyName: string,
    secret: string
  ) => Promise<ProjectSecret>;

  /**
   * Lists all secrets for a project
   * @param orgId - Organization ID
   * @param projectId - Project ID
   */
  listProjectSecrets: (orgId: string, projectId: string) => Promise<ProjectSecret[]>;

  /**
   * Deletes a project secret
   * @param orgId - Organization ID
   * @param projectId - Project ID
   * @param keyName - Secret key name
   */
  deleteProjectSecret: (orgId: string, projectId: string, keyName: string) => Promise<void>;

  /**
   * Gets email configuration for a project
   * @param orgId - Organization ID
   * @param projectId - Project ID
   */
  getEmailSettings: (orgId: string, projectId: string) => Promise<EmailSettings>;

  /**
   * Updates email configuration
   * @param orgId - Organization ID
   * @param projectId - Project ID
   * @param settings - Email settings
   */
  updateEmailSettings: (
    orgId: string,
    projectId: string,
    settings: EmailSettings
  ) => Promise<EmailSettings>;

  /**
   * Gets OAuth settings for a project
   * @param orgId - Organization ID
   * @param projectId - Project ID
   */
  getOAuthSettings: (orgId: string, projectId: string) => Promise<OAuthSettings>;

  /**
   * Updates OAuth configuration
   * @param orgId - Organization ID
   * @param projectId - Project ID
   * @param settings - OAuth settings
   */
  updateOAuthSettings: (
    orgId: string,
    projectId: string,
    settings: OAuthSettings
  ) => Promise<OAuthSettings>;

  /**
   * Creates an invitation to join an organization
   * @param orgId - Organization ID
   * @param email - Developer's email address
   * @param role - Role to assign (defaults to "admin")
   */
  inviteDeveloper: (orgId: string, email: string, role?: string) => Promise<OrganizationInvite>;

  /**
   * Lists all members of an organization
   * @param orgId - Organization ID
   */
  listOrganizationMembers: (orgId: string) => Promise<OrganizationMember[]>;

  /**
   * Lists all pending invitations for an organization
   * @param orgId - Organization ID
   */
  listOrganizationInvites: (orgId: string) => Promise<OrganizationInvite[]>;

  /**
   * Gets a specific invitation by code
   * @param orgId - Organization ID
   * @param inviteCode - Invitation UUID code
   */
  getOrganizationInvite: (orgId: string, inviteCode: string) => Promise<OrganizationInvite>;

  /**
   * Deletes an invitation
   * @param orgId - Organization ID
   * @param inviteCode - Invitation UUID code
   */
  deleteOrganizationInvite: (orgId: string, inviteCode: string) => Promise<{ message: string }>;

  /**
   * Updates a member's role
   * @param orgId - Organization ID
   * @param userId - User ID to update
   * @param role - New role to assign
   */
  updateMemberRole: (orgId: string, userId: string, role: string) => Promise<OrganizationMember>;

  /**
   * Removes a member from the organization
   * @param orgId - Organization ID
   * @param userId - User ID to remove
   */
  removeMember: (orgId: string, userId: string) => Promise<void>;

  /**
   * Accepts an organization invitation
   * @param code - Invitation UUID code
   */
  acceptInvite: (code: string) => Promise<{ message: string }>;

  /**
   * Returns the current OpenSecret developer API URL being used
   */
  apiUrl: string;
};

export const OpenSecretDeveloperContext = createContext<OpenSecretDeveloperContextType>({
  auth: {
    loading: true,
    developer: undefined
  },
  signIn: async () => {
    throw new Error("signIn called outside of OpenSecretDeveloper provider");
  },
  signUp: async () => {
    throw new Error("signUp called outside of OpenSecretDeveloper provider");
  },
  signOut: async () => {
    throw new Error("signOut called outside of OpenSecretDeveloper provider");
  },
  refetchDeveloper: async () => {
    throw new Error("refetchDeveloper called outside of OpenSecretDeveloper provider");
  },
  pcrConfig: {},
  getAttestation,
  authenticate,
  parseAttestationForView,
  awsRootCertDer: AWS_ROOT_CERT_DER,
  expectedRootCertHash: EXPECTED_ROOT_CERT_HASH,
  getAttestationDocument: async () => {
    throw new Error("getAttestationDocument called outside of OpenSecretDeveloper provider");
  },
  createOrganization: platformApi.createOrganization,
  listOrganizations: platformApi.listOrganizations,
  deleteOrganization: platformApi.deleteOrganization,
  createProject: platformApi.createProject,
  listProjects: platformApi.listProjects,
  getProject: platformApi.getProject,
  updateProject: platformApi.updateProject,
  deleteProject: platformApi.deleteProject,
  createProjectSecret: platformApi.createProjectSecret,
  listProjectSecrets: platformApi.listProjectSecrets,
  deleteProjectSecret: platformApi.deleteProjectSecret,
  getEmailSettings: platformApi.getEmailSettings,
  updateEmailSettings: platformApi.updateEmailSettings,
  getOAuthSettings: platformApi.getOAuthSettings,
  updateOAuthSettings: platformApi.updateOAuthSettings,
  inviteDeveloper: platformApi.inviteDeveloper,
  listOrganizationMembers: platformApi.listOrganizationMembers,
  listOrganizationInvites: platformApi.listOrganizationInvites,
  getOrganizationInvite: platformApi.getOrganizationInvite,
  deleteOrganizationInvite: platformApi.deleteOrganizationInvite,
  updateMemberRole: platformApi.updateMemberRole,
  removeMember: platformApi.removeMember,
  acceptInvite: platformApi.acceptInvite,
  apiUrl: ""
});

/**
 * Provider component for OpenSecret developer operations.
 * This provider is used for managing organizations, projects, and developer access.
 *
 * @param props - Configuration properties for the OpenSecret developer provider
 * @param props.children - React child components to be wrapped by the provider
 * @param props.apiUrl - URL of OpenSecret developer API
 *
 * @example
 * ```tsx
 * <OpenSecretDeveloper
 *   apiUrl='https://developer.opensecret.cloud'
 * >
 *   <App />
 * </OpenSecretDeveloper>
 * ```
 */
export function OpenSecretDeveloper({
  children,
  apiUrl,
  pcrConfig = {}
}: {
  children: React.ReactNode;
  apiUrl: string;
  pcrConfig?: PcrConfig;
}) {
  const [auth, setAuth] = useState<OpenSecretDeveloperAuthState>({
    loading: true,
    developer: undefined
  });

  useEffect(() => {
    if (!apiUrl || apiUrl.trim() === "") {
      throw new Error(
        "OpenSecretDeveloper requires a non-empty apiUrl. Please provide a valid API endpoint URL."
      );
    }
    setPlatformApiUrl(apiUrl);

    // Configure the apiConfig service with the platform URL
    // Using dynamic import to avoid circular dependencies
    import("./apiConfig").then(({ apiConfig }) => {
      const appUrl = apiConfig.appApiUrl || '';
      apiConfig.configure(appUrl, apiUrl);
    });
  }, [apiUrl]);

  async function fetchDeveloper() {
    const access_token = window.localStorage.getItem("access_token");
    const refresh_token = window.localStorage.getItem("refresh_token");
    if (!access_token || !refresh_token) {
      setAuth({
        loading: false,
        developer: undefined
      });
      return;
    }

    try {
      const response = await platformApi.platformMe();
      setAuth({
        loading: false,
        developer: {
          ...response.user,
          organizations: response.organizations
        }
      });
    } catch (error) {
      console.error("Failed to fetch developer:", error);
      setAuth({
        loading: false,
        developer: undefined
      });
    }
  }

  const getAttestationDocument = async () => {
    const nonce = window.crypto.randomUUID();
    const response = await fetch(`${apiUrl}/attestation/${nonce}`);
    if (!response.ok) {
      throw new Error("Failed to fetch attestation document");
    }

    const data = await response.json();
    const verifiedDocument = await authenticate(
      data.attestation_document,
      AWS_ROOT_CERT_DER,
      nonce
    );
    return parseAttestationForView(verifiedDocument, verifiedDocument.cabundle, pcrConfig);
  };

  useEffect(() => {
    fetchDeveloper();
  }, []);

  async function signIn(email: string, password: string) {
    try {
      const { access_token, refresh_token } = await platformApi.platformLogin(email, password);
      window.localStorage.setItem("access_token", access_token);
      window.localStorage.setItem("refresh_token", refresh_token);
      await fetchDeveloper();
      return { access_token, refresh_token, id: "", email };
    } catch (error) {
      console.error("Login error:", error);
      throw error;
    }
  }

  async function signUp(email: string, password: string, name?: string) {
    try {
      const { access_token, refresh_token } = await platformApi.platformRegister(
        email,
        password,
        name
      );
      window.localStorage.setItem("access_token", access_token);
      window.localStorage.setItem("refresh_token", refresh_token);
      await fetchDeveloper();
      return { access_token, refresh_token, id: "", email, name };
    } catch (error) {
      console.error("Registration error:", error);
      throw error;
    }
  }

  const value: OpenSecretDeveloperContextType = {
    auth,
    signIn,
    signUp,
    refetchDeveloper: fetchDeveloper,
    signOut: async () => {
      const refresh_token = window.localStorage.getItem("refresh_token");
      if (refresh_token) {
        try {
          await platformApi.platformLogout(refresh_token);
        } catch (error) {
          console.error("Error during logout:", error);
        }
      }
      localStorage.removeItem("access_token");
      localStorage.removeItem("refresh_token");
      setAuth({
        loading: false,
        developer: undefined
      });
    },
    pcrConfig,
    getAttestation,
    authenticate,
    parseAttestationForView,
    awsRootCertDer: AWS_ROOT_CERT_DER,
    expectedRootCertHash: EXPECTED_ROOT_CERT_HASH,
    getAttestationDocument,
    createOrganization: platformApi.createOrganization,
    listOrganizations: platformApi.listOrganizations,
    deleteOrganization: platformApi.deleteOrganization,
    createProject: platformApi.createProject,
    listProjects: platformApi.listProjects,
    getProject: platformApi.getProject,
    updateProject: platformApi.updateProject,
    deleteProject: platformApi.deleteProject,
    createProjectSecret: platformApi.createProjectSecret,
    listProjectSecrets: platformApi.listProjectSecrets,
    deleteProjectSecret: platformApi.deleteProjectSecret,
    getEmailSettings: platformApi.getEmailSettings,
    updateEmailSettings: platformApi.updateEmailSettings,
    getOAuthSettings: platformApi.getOAuthSettings,
    updateOAuthSettings: platformApi.updateOAuthSettings,
    inviteDeveloper: platformApi.inviteDeveloper,
    listOrganizationMembers: platformApi.listOrganizationMembers,
    listOrganizationInvites: platformApi.listOrganizationInvites,
    getOrganizationInvite: platformApi.getOrganizationInvite,
    deleteOrganizationInvite: platformApi.deleteOrganizationInvite,
    updateMemberRole: platformApi.updateMemberRole,
    removeMember: platformApi.removeMember,
    acceptInvite: platformApi.acceptInvite,
    apiUrl
  };

  return (
    <OpenSecretDeveloperContext.Provider value={value}>
      {children}
    </OpenSecretDeveloperContext.Provider>
  );
}
