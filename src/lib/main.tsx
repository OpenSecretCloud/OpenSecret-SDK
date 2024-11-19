import React, { createContext, useState, useEffect } from "react";
import * as api from "./api";
import { createCustomFetch } from "./ai";

export type OpenSecretAuthState = {
  loading: boolean;
  user?: api.UserResponse;
};

export type OpenSecretContextType = {
  auth: OpenSecretAuthState;

  /**
   * Authenticates a user with email and password
   * @param email - User's email address
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
  signIn: (email: string, password: string) => Promise<void>;

  /**
   * Creates a new user account
   * @param name - User's full name
   * @param email - User's email address
   * @param password - User's chosen password
   * @param inviteCode - Invitation code for registration
   * @returns A promise that resolves when account creation is complete
   * @throws {Error} If signup fails
   *
   * @description
   * - Calls the registration API endpoint
   * - Stores access_token and refresh_token in localStorage
   * - Updates the auth state with new user information
   * - Throws an error if account creation fails
   */
  signUp: (name: string, email: string, password: string, inviteCode: string) => Promise<void>;

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
   * Retrieves the user's public key for the specified algorithm
   * @param algorithm - The signing algorithm ('schnorr' or 'ecdsa')
   * @returns A promise resolving to the public key response
   * @throws {Error} If the public key cannot be retrieved
   */
  getPublicKey: typeof api.fetchPublicKey;

  /**
   * Signs a message using the specified algorithm
   * @param messageBytes - The message to sign as a Uint8Array
   * @param algorithm - The signing algorithm ('schnorr' or 'ecdsa')
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
};

export const OpenSecretContext = createContext<OpenSecretContextType>({
  auth: {
    loading: true,
    user: undefined
  },
  signIn: async () => {},
  signUp: async () => {},
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
  initiateGitHubAuth: async () => ({ auth_url: "", csrf_token: "" }),
  handleGitHubCallback: async () => {},
  initiateGoogleAuth: async () => ({ auth_url: "", csrf_token: "" }),
  handleGoogleCallback: async () => {},
  getPrivateKey: api.fetchPrivateKey,
  getPublicKey: api.fetchPublicKey,
  signMessage: api.signMessage,
  aiCustomFetch: async () => new Response(),
  apiUrl: ""
});

/**
 * Provider component for OpenSecret authentication and key-value storage.
 *
 * @param props - Configuration properties for the OpenSecret provider
 * @param props.children - React child components to be wrapped by the provider
 * @param props.apiUrl - URL of OpenSecret enclave backend
 *
 * @remarks
 * This provider manages:
 * - User authentication state
 * - Authentication methods (sign in, sign up, sign out)
 * - Key-value storage operations
 *
 * @example
 * ```tsx
 * <OpenSecretProvider apiUrl='https://preview.opensecret.ai'>
 *   <App />
 * </OpenSecretProvider>
 * ```
 */
export function OpenSecretProvider({
  children,
  apiUrl
}: {
  children: React.ReactNode;
  apiUrl: string;
}) {
  const [auth, setAuth] = useState<OpenSecretAuthState>({
    loading: true,
    user: undefined
  });
  const [aiCustomFetch, setAiCustomFetch] = useState<OpenSecretContextType['aiCustomFetch']>();

  useEffect(() => {
    if (!apiUrl || apiUrl.trim() === '') {
      throw new Error('OpenSecretProvider requires a non-empty apiUrl. Please provide a valid API endpoint URL.');
    }
    api.setApiUrl(apiUrl);
  }, [apiUrl]);

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
      const { access_token, refresh_token } = await api.fetchLogin(email, password);
      window.localStorage.setItem("access_token", access_token);
      window.localStorage.setItem("refresh_token", refresh_token);
      await fetchUser();
    } catch (error) {
      console.error(error);
      throw error;
    }
  }

  async function signUp(name: string, email: string, password: string, inviteCode: string) {
    try {
      const { access_token, refresh_token } = await api.fetchSignUp(
        name,
        email,
        password,
        inviteCode
      );
      window.localStorage.setItem("access_token", access_token);
      window.localStorage.setItem("refresh_token", refresh_token);
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
      return await api.initiateGitHubAuth(inviteCode);
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
      return await api.initiateGoogleAuth(inviteCode);
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

  const value: OpenSecretContextType = {
    auth,
    signIn,
    signOut,
    signUp,
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
    initiateGitHubAuth,
    handleGitHubCallback,
    initiateGoogleAuth,
    handleGoogleCallback,
    getPrivateKey: api.fetchPrivateKey,
    getPublicKey: api.fetchPublicKey,
    signMessage: api.signMessage,
    aiCustomFetch: aiCustomFetch || (async () => new Response()),
    apiUrl
  };

  return <OpenSecretContext.Provider value={value}>{children}</OpenSecretContext.Provider>;
}
