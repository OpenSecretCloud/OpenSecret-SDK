import React, { createContext, useState, useEffect } from "react";
import * as api from "./api";

export interface OpenSecretConfig {
  apiUrl?: string;
}

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
  handleGoogleCallback: async () => {}
});

export function OpenSecretProvider({ 
  children, 
  config = {} 
}: { 
  children: React.ReactNode;
  config?: OpenSecretConfig;
}) {
  const [auth, setAuth] = useState<OpenSecretAuthState>({
    loading: true,
    user: undefined
  });

  useEffect(() => {
    if (config.apiUrl) {
      api.setApiUrl(config.apiUrl);
    }
  }, [config.apiUrl]);

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
    handleGoogleCallback
  };

  return <OpenSecretContext.Provider value={value}>{children}</OpenSecretContext.Provider>;
}
