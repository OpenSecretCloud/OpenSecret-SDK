import React, { createContext, useState, useEffect } from "react";
import * as api from "./api";
import { createCustomFetch } from "./ai";
import { getAttestation } from "./getAttestation";
import { authenticate } from "./attestation";
import {
  parseAttestationForView,
  AWS_ROOT_CERT_DER,
  EXPECTED_ROOT_CERT_HASH,
  ParsedAttestationView
} from "./attestationForView";
import type { AttestationDocument } from "./attestation";
import type { LoginResponse, ThirdPartyTokenResponse } from "./api";
import { PcrConfig } from "./pcr";

export type OpenSecretAuthState = {
  loading: boolean;
  user?: api.UserResponse;
};

export type OpenSecretContextType = {
  auth: OpenSecretAuthState;

  /**
   * The client ID for this project/tenant
   * @description A UUID that identifies which project/tenant this instance belongs to
   */
  clientId: string;

  /**
   * Authenticates a user with email and password
   * @param email - User's email address
   * @param password - User's password
   * @returns A promise that resolves when authentication is complete
   * @throws {Error} If login fails
   *
   * @description
   * - Calls the login API endpoint with the configured clientId
   * - Stores access_token and refresh_token in localStorage
   * - Updates the auth state with user information
   * - Throws an error if authentication fails
   */
  signIn: (email: string, password: string) => Promise<void>;

  /**
   * Creates a new user account
   * @param email - User's email address
   * @param password - User's chosen password
   * @param inviteCode - Invitation code for registration
   * @param name - Optional user's full name
   * @returns A promise that resolves when account creation is complete
   * @throws {Error} If signup fails
   *
   * @description
   * - Calls the registration API endpoint
   * - Stores access_token and refresh_token in localStorage
   * - Updates the auth state with new user information
   * - Throws an error if account creation fails
   */
  signUp: (email: string, password: string, inviteCode: string, name?: string) => Promise<void>;

  /**
   * Authenticates a guest user with user id and password
   * @param id - User's unique id
   * @param password - User's password
   * @returns A promise that resolves when authentication is complete
   * @throws {Error} If login fails
   *
   * @description
   * - Calls the login API endpoint
   * - Stores access_token and refresh_token in localStorage
   * - Updates the auth state with user information
   * - Throws an error if authentication fails
   */
  signInGuest: (id: string, password: string) => Promise<void>;

  /**
   * Creates a new guest account, which can be upgraded to a normal account later with email.
   * @param password - User's chosen password, cannot be changed or recovered without adding email address.
   * @param inviteCode - Invitation code for registration
   * @returns A promise that resolves to the login response containing the guest ID
   * @throws {Error} If signup fails
   *
   * @description
   * - Calls the registration API endpoint
   * - Stores access_token and refresh_token in localStorage
   * - Updates the auth state with new user information
   * - Throws an error if account creation fails
   */
  signUpGuest: (password: string, inviteCode: string) => Promise<LoginResponse>;

  /**
   * Upgrades a guest account to a user account with email and password authentication.
   * @param email - User's email address
   * @param password - User's chosen password
   * @param name - Optional user's full name
   * @returns A promise that resolves when account creation is complete
   * @throws {Error} If:
   * - The current user is not a guest account
   * - The email address is already in use
   * - The user is not authenticated
   *
   * @description
   * - Upgrades the currently signed-in guest account (identified by their UUID) to a full email account
   * - Requires the user to be currently authenticated as a guest
   * - Updates the auth state with new user information
   * - Preserves all existing data associated with the guest account
   */
  convertGuestToUserAccount: (
    email: string,
    password: string,
    name?: string | null
  ) => Promise<void>;

  /**
   * Logs out the current user
   * @returns A promise that resolves when logout is complete
   * @throws {Error} If logout fails
   *
   * @description
   * - Calls the logout API endpoint with the current refresh_token
   * - Removes access_token, refresh_token from localStorage
   * - Removes session-related items from sessionStorage
   * - Resets the auth state to show no user is authenticated
   */
  signOut: () => Promise<void>;

  /**
   * Retrieves a value from key-value storage
   * @param key - The unique identifier for the stored value
   * @returns A promise resolving to the stored value
   * @throws {Error} If the key cannot be retrieved
   *
   * @description
   * - Calls the authenticated API endpoint to fetch a value
   * - Returns undefined if the key does not exist
   * - Requires an active authentication session
   * - Logs any retrieval errors
   */
  get: typeof api.fetchGet;

  /**
   * Stores a key-value pair in the user's storage
   * @param key - The unique identifier for the value
   * @param value - The string value to be stored
   * @returns A promise resolving to the server's response
   * @throws {Error} If the value cannot be stored
   *
   * @description
   * - Calls the authenticated API endpoint to store a value
   * - Requires an active authentication session
   * - Overwrites any existing value for the given key
   * - Logs any storage errors
   */
  put: typeof api.fetchPut;

  /**
   * Retrieves all key-value pairs stored by the user
   * @returns A promise resolving to an array of stored items
   * @throws {Error} If the list cannot be retrieved
   *
   * @description
   * - Calls the authenticated API endpoint to fetch all stored items
   * - Returns an array of key-value pairs with metadata
   * - Requires an active authentication session
   * - Each item includes key, value, creation, and update timestamps
   * - Logs any listing errors
   */
  list: typeof api.fetchList;

  /**
   * Deletes a key-value pair from the user's storage
   * @param key - The unique identifier for the value to be deleted
   * @returns A promise resolving when the deletion is complete
   * @throws {Error} If the key cannot be deleted
   *
   * @description
   * - Calls the authenticated API endpoint to remove a specific key
   * - Requires an active authentication session
   * - Throws an error if the deletion fails (including for non-existent keys)
   * - Propagates any server-side errors directly
   */
  del: typeof api.fetchDelete;

  verifyEmail: typeof api.verifyEmail;
  requestNewVerificationCode: typeof api.requestNewVerificationCode;
  requestNewVerificationEmail: typeof api.requestNewVerificationCode;
  refetchUser: () => Promise<void>;
  changePassword: typeof api.changePassword;
  refreshAccessToken: typeof api.refreshToken;
  requestPasswordReset: (email: string, hashedSecret: string) => Promise<void>;
  confirmPasswordReset: (
    email: string,
    alphanumericCode: string,
    plaintextSecret: string,
    newPassword: string
  ) => Promise<void>;
  initiateGitHubAuth: (inviteCode: string) => Promise<api.GithubAuthResponse>;
  handleGitHubCallback: (code: string, state: string, inviteCode: string) => Promise<void>;
  initiateGoogleAuth: (inviteCode: string) => Promise<api.GoogleAuthResponse>;
  handleGoogleCallback: (code: string, state: string, inviteCode: string) => Promise<void>;

  /**
   * Retrieves the user's private key mnemonic phrase
   * @returns A promise resolving to the private key response
   * @throws {Error} If the private key cannot be retrieved
   */
  getPrivateKey: typeof api.fetchPrivateKey;

  /**
   * Retrieves the private key bytes for a given derivation path
   * @param derivationPath - Optional BIP32 derivation path (e.g. "m/44'/0'/0'/0/0")
   * @returns A promise resolving to the private key bytes response
   * @throws {Error} If:
   * - The private key bytes cannot be retrieved
   * - The derivation path is invalid
   *
   * @description
   * - If no derivation path is provided, returns the master private key bytes
   * - Supports both absolute (starting with "m/") and relative paths
   * - Supports hardened derivation using either ' or h notation
   * - Common paths:
   *   - BIP44 (Legacy): m/44'/0'/0'/0/0
   *   - BIP49 (SegWit): m/49'/0'/0'/0/0
   *   - BIP84 (Native SegWit): m/84'/0'/0'/0/0
   *   - BIP86 (Taproot): m/86'/0'/0'/0/0
   */
  getPrivateKeyBytes: typeof api.fetchPrivateKeyBytes;

  /**
   * Retrieves the user's public key for the specified algorithm
   * @param algorithm - The signing algorithm ('schnorr' or 'ecdsa')
   * @param derivationPath - Optional BIP32 derivation path
   * @returns A promise resolving to the public key response
   * @throws {Error} If the public key cannot be retrieved
   */
  getPublicKey: typeof api.fetchPublicKey;

  /**
   * Signs a message using the specified algorithm
   * @param messageBytes - The message to sign as a Uint8Array
   * @param algorithm - The signing algorithm ('schnorr' or 'ecdsa')
   * @param derivationPath - Optional BIP32 derivation path
   * @returns A promise resolving to the signature response
   * @throws {Error} If the message signing fails
   */
  signMessage: typeof api.signMessage;

  /**
   * Custom fetch function for AI requests that handles encryption
   * and token refreshing.
   *
   * Meant to be used with the OpenAI JS library
   *
   * Example:
   * ```tsx
   * const openai = new OpenAI({
   *   baseURL: `${os.apiUrl}/v1/`,
   *   dangerouslyAllowBrowser: true,
   *   apiKey: "the-api-key-doesnt-matter",
   *   defaultHeaders: {
   *     "Accept-Encoding": "identity"
   *   },
   *   fetch: os.aiCustomFetch
   * });
   * ```
   */
  aiCustomFetch: (url: RequestInfo, init?: RequestInit) => Promise<Response>;

  /**
   * Returns the current OpenSecret enclave API URL being used
   * @returns The current API URL
   */
  apiUrl: string;

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
   * Generates a JWT token for use with authorized third-party services
   * @param audience - The URL of the authorized service (e.g. "https://billing.opensecret.cloud")
   * @returns A promise resolving to the token response
   * @throws {Error} If:
   * - The user is not authenticated
   * - The audience URL is invalid
   * - The audience URL is not authorized
   *
   * @description
   * - Generates a signed JWT token for use with specific authorized third-party services
   * - The audience must be an pre-authorized URL registered by the developer (e.g. api.devservice.com)
   * - Requires an active authentication session
   * - Token can be used to authenticate with the specified service
   */
  generateThirdPartyToken: (audience: string) => Promise<ThirdPartyTokenResponse>;

  /**
   * Encrypts arbitrary string data using the user's private key
   * @param data - String content to be encrypted
   * @param derivationPath - Optional BIP32 derivation path (e.g., 'm/44'/0'/0'/0/0')
   * @returns A promise resolving to the encrypted data response
   * @throws {Error} If:
   * - The derivation path is invalid
   * - Authentication fails
   * - Server-side encryption error occurs
   *
   * @description
   * - Encrypts data with AES-256-GCM
   * - A random nonce is generated for each encryption operation (included in the result)
   * - If derivation_path is provided, encryption uses the derived key at that path
   * - If derivation_path is omitted, encryption uses the master key
   * - The encrypted_data format:
   *   - First 12 bytes: Nonce used for encryption (prepended to ciphertext)
   *   - Remaining bytes: AES-256-GCM encrypted ciphertext + authentication tag
   *   - The entire payload is base64-encoded for safe transport
   */
  encryptData: typeof api.encryptData;

  /**
   * Decrypts data that was previously encrypted with the user's key
   * @param encryptedData - Base64-encoded encrypted data string
   * @param derivationPath - Optional BIP32 derivation path (e.g., 'm/44'/0'/0'/0/0')
   * @returns A promise resolving to the decrypted string
   * @throws {Error} If:
   * - The encrypted data is malformed
   * - The derivation path is invalid
   * - Authentication fails
   * - Server-side decryption error occurs
   *
   * @description
   * - Uses AES-256-GCM decryption with authentication tag verification
   * - Extracts the nonce from the first 12 bytes of the encrypted data
   * - If derivation_path is provided, decryption uses the derived key at that path
   * - If derivation_path is omitted, decryption uses the master key
   * - The encrypted_data must be in the exact format returned by the encrypt endpoint
   */
  decryptData: typeof api.decryptData;
};

export const OpenSecretContext = createContext<OpenSecretContextType>({
  auth: {
    loading: true,
    user: undefined
  },
  clientId: "",
  signIn: async () => {},
  signUp: async () => {},
  signInGuest: async () => {},
  signUpGuest: async (): Promise<LoginResponse> => ({
    id: "",
    email: undefined,
    access_token: "",
    refresh_token: ""
  }),
  convertGuestToUserAccount: async () => {},
  signOut: async () => {},
  get: api.fetchGet,
  put: api.fetchPut,
  list: api.fetchList,
  del: api.fetchDelete,
  verifyEmail: api.verifyEmail,
  requestNewVerificationCode: api.requestNewVerificationCode,
  requestNewVerificationEmail: api.requestNewVerificationCode,
  refetchUser: async () => {},
  changePassword: api.changePassword,
  refreshAccessToken: api.refreshToken,
  requestPasswordReset: async () => {},
  confirmPasswordReset: async () => {},
  initiateGitHubAuth: async () => ({ auth_url: "", csrf_token: "" }),
  handleGitHubCallback: async () => {},
  initiateGoogleAuth: async () => ({ auth_url: "", csrf_token: "" }),
  handleGoogleCallback: async () => {},
  getPrivateKey: api.fetchPrivateKey,
  getPrivateKeyBytes: api.fetchPrivateKeyBytes,
  getPublicKey: api.fetchPublicKey,
  signMessage: api.signMessage,
  aiCustomFetch: async () => new Response(),
  apiUrl: "",
  pcrConfig: {},
  getAttestation,
  authenticate,
  parseAttestationForView,
  awsRootCertDer: AWS_ROOT_CERT_DER,
  expectedRootCertHash: EXPECTED_ROOT_CERT_HASH,
  getAttestationDocument: async () => {
    throw new Error("getAttestationDocument called outside of OpenSecretProvider");
  },
  generateThirdPartyToken: async () => ({ token: "" }),
  encryptData: api.encryptData,
  decryptData: api.decryptData
});

/**
 * Provider component for OpenSecret authentication and key-value storage.
 *
 * @param props - Configuration properties for the OpenSecret provider
 * @param props.children - React child components to be wrapped by the provider
 * @param props.apiUrl - URL of OpenSecret enclave backend
 * @param props.clientId - UUID identifying which project/tenant this instance belongs to
 * @param props.pcrConfig - Optional PCR configuration for attestation validation
 *
 * @remarks
 * This provider manages:
 * - User authentication state
 * - Authentication methods (sign in, sign up, sign out)
 * - Key-value storage operations
 * - Project/tenant identification via clientId
 *
 * @example
 * ```tsx
 * <OpenSecretProvider
 *   apiUrl='https://preview.opensecret.ai'
 *   clientId='550e8400-e29b-41d4-a716-446655440000'
 * >
 *   <App />
 * </OpenSecretProvider>
 * ```
 */
export function OpenSecretProvider({
  children,
  apiUrl,
  clientId,
  pcrConfig = {}
}: {
  children: React.ReactNode;
  apiUrl: string;
  clientId: string;
  pcrConfig?: PcrConfig;
}) {
  const [auth, setAuth] = useState<OpenSecretAuthState>({
    loading: true,
    user: undefined
  });
  const [aiCustomFetch, setAiCustomFetch] = useState<OpenSecretContextType["aiCustomFetch"]>();

  useEffect(() => {
    if (!apiUrl || apiUrl.trim() === "") {
      throw new Error(
        "OpenSecretProvider requires a non-empty apiUrl. Please provide a valid API endpoint URL."
      );
    }
    if (!clientId || clientId.trim() === "") {
      throw new Error(
        "OpenSecretProvider requires a non-empty clientId. Please provide a valid project UUID."
      );
    }
    api.setApiUrl(apiUrl);

    // Configure the apiConfig service with the app URL
    // Using dynamic import to avoid circular dependencies
    import("./apiConfig").then(({ apiConfig }) => {
      const platformUrl = apiConfig.platformApiUrl || "";
      apiConfig.configure(apiUrl, platformUrl);
    });
  }, [apiUrl, clientId]);

  // Create aiCustomFetch when user is authenticated
  useEffect(() => {
    if (auth.user) {
      setAiCustomFetch(() => createCustomFetch());
    } else {
      setAiCustomFetch(undefined);
    }
  }, [auth.user]);

  async function fetchUser() {
    const access_token = window.localStorage.getItem("access_token");
    const refresh_token = window.localStorage.getItem("refresh_token");
    if (!access_token || !refresh_token) {
      setAuth({
        loading: false,
        user: undefined
      });
      return;
    }

    try {
      const user = await api.fetchUser();
      setAuth({
        loading: false,
        user
      });
    } catch (error) {
      console.error("Failed to fetch user:", error);
      setAuth({
        loading: false,
        user: undefined
      });
    }
  }

  useEffect(() => {
    fetchUser();
  }, []);

  async function signIn(email: string, password: string) {
    console.log("Signing in");
    try {
      const { access_token, refresh_token } = await api.fetchLogin(email, password, clientId);
      window.localStorage.setItem("access_token", access_token);
      window.localStorage.setItem("refresh_token", refresh_token);
      await fetchUser();
    } catch (error) {
      console.error(error);
      throw error;
    }
  }

  async function signUp(email: string, password: string, inviteCode: string, name?: string) {
    try {
      const { access_token, refresh_token } = await api.fetchSignUp(
        email,
        password,
        inviteCode,
        clientId,
        name || null
      );
      window.localStorage.setItem("access_token", access_token);
      window.localStorage.setItem("refresh_token", refresh_token);
      await fetchUser();
    } catch (error) {
      console.error(error);
      throw error;
    }
  }

  async function signInGuest(id: string, password: string) {
    console.log("Signing in Guest");
    try {
      const { access_token, refresh_token } = await api.fetchGuestLogin(id, password, clientId);
      window.localStorage.setItem("access_token", access_token);
      window.localStorage.setItem("refresh_token", refresh_token);
      await fetchUser();
    } catch (error) {
      console.error(error);
      throw error;
    }
  }

  async function signUpGuest(password: string, inviteCode: string) {
    try {
      const { access_token, refresh_token, id } = await api.fetchGuestSignUp(
        password,
        inviteCode,
        clientId
      );
      window.localStorage.setItem("access_token", access_token);
      window.localStorage.setItem("refresh_token", refresh_token);
      await fetchUser();
      return { access_token, refresh_token, id };
    } catch (error) {
      console.error(error);
      throw error;
    }
  }

  async function convertGuestToUserAccount(email: string, password: string, name?: string | null) {
    try {
      await api.convertGuestToEmailAccount(email, password, name);
      await fetchUser();
    } catch (error) {
      console.error(error);
      throw error;
    }
  }

  async function signOut() {
    const refresh_token = window.localStorage.getItem("refresh_token");
    if (refresh_token) {
      try {
        await api.fetchLogout(refresh_token);
      } catch (error) {
        console.error("Error during logout:", error);
      }
    }
    localStorage.removeItem("access_token");
    localStorage.removeItem("refresh_token");
    sessionStorage.removeItem("sessionKey");
    sessionStorage.removeItem("sessionId");
    setAuth({
      loading: false,
      user: undefined
    });
  }

  const initiateGitHubAuth = async (inviteCode: string) => {
    try {
      return await api.initiateGitHubAuth(clientId, inviteCode);
    } catch (error) {
      console.error("Failed to initiate GitHub auth:", error);
      throw error;
    }
  };

  const handleGitHubCallback = async (code: string, state: string, inviteCode: string) => {
    try {
      const { access_token, refresh_token } = await api.handleGitHubCallback(
        code,
        state,
        inviteCode
      );
      window.localStorage.setItem("access_token", access_token);
      window.localStorage.setItem("refresh_token", refresh_token);
      await fetchUser();
    } catch (error) {
      console.error("GitHub callback error:", error);
      throw error;
    }
  };

  const initiateGoogleAuth = async (inviteCode: string) => {
    try {
      return await api.initiateGoogleAuth(clientId, inviteCode);
    } catch (error) {
      console.error("Failed to initiate Google auth:", error);
      throw error;
    }
  };

  const handleGoogleCallback = async (code: string, state: string, inviteCode: string) => {
    try {
      const { access_token, refresh_token } = await api.handleGoogleCallback(
        code,
        state,
        inviteCode
      );
      window.localStorage.setItem("access_token", access_token);
      window.localStorage.setItem("refresh_token", refresh_token);
      await fetchUser();
    } catch (error) {
      console.error("Google callback error:", error);
      throw error;
    }
  };

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

  const value: OpenSecretContextType = {
    auth,
    clientId,
    signIn,
    signInGuest,
    signOut,
    signUp,
    signUpGuest,
    convertGuestToUserAccount,
    get: api.fetchGet,
    put: api.fetchPut,
    list: api.fetchList,
    del: api.fetchDelete,
    refetchUser: fetchUser,
    verifyEmail: api.verifyEmail,
    requestNewVerificationCode: api.requestNewVerificationCode,
    requestNewVerificationEmail: api.requestNewVerificationCode,
    changePassword: api.changePassword,
    refreshAccessToken: api.refreshToken,
    requestPasswordReset: (email: string, hashedSecret: string) =>
      api.requestPasswordReset(email, hashedSecret, clientId),
    confirmPasswordReset: (
      email: string,
      alphanumericCode: string,
      plaintextSecret: string,
      newPassword: string
    ) => api.confirmPasswordReset(email, alphanumericCode, plaintextSecret, newPassword, clientId),
    initiateGitHubAuth,
    handleGitHubCallback,
    initiateGoogleAuth,
    handleGoogleCallback,
    getPrivateKey: api.fetchPrivateKey,
    getPrivateKeyBytes: api.fetchPrivateKeyBytes,
    getPublicKey: api.fetchPublicKey,
    signMessage: api.signMessage,
    aiCustomFetch: aiCustomFetch || (async () => new Response()),
    apiUrl,
    pcrConfig,
    getAttestation,
    authenticate,
    parseAttestationForView,
    awsRootCertDer: AWS_ROOT_CERT_DER,
    expectedRootCertHash: EXPECTED_ROOT_CERT_HASH,
    getAttestationDocument,
    generateThirdPartyToken: api.generateThirdPartyToken,
    encryptData: api.encryptData,
    decryptData: api.decryptData
  };

  return <OpenSecretContext.Provider value={value}>{children}</OpenSecretContext.Provider>;
}
