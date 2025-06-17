import { encode } from "@stablelib/base64";
import { authenticatedApiCall, encryptedApiCall } from "./encryptedApi";
import type { Model } from "openai/resources/models.js";

let apiUrl = "";

export function setApiUrl(url: string) {
  apiUrl = url;
}

export function getApiUrl(): string {
  return apiUrl;
}

export type LoginResponse = {
  id: string;
  email?: string;
  access_token: string;
  refresh_token: string;
};

export type UserResponse = {
  user: {
    id: string;
    name: string | null;
    email?: string;
    email_verified: boolean;
    login_method: string;
    created_at: string;
    updated_at: string;
  };
};

type RefreshResponse = {
  access_token: string;
  refresh_token: string;
};

export type KVListItem = {
  key: string;
  value: string;
  created_at: number;
  updated_at: number;
};

export async function fetchLogin(
  email: string,
  password: string,
  client_id: string
): Promise<LoginResponse> {
  return encryptedApiCall<{ email: string; password: string; client_id: string }, LoginResponse>(
    `${apiUrl}/login`,
    "POST",
    { email, password, client_id }
  );
}

export async function fetchGuestLogin(
  id: string,
  password: string,
  client_id: string
): Promise<LoginResponse> {
  return encryptedApiCall<{ id: string; password: string; client_id: string }, LoginResponse>(
    `${apiUrl}/login`,
    "POST",
    { id, password, client_id }
  );
}

export async function fetchSignUp(
  email: string,
  password: string,
  inviteCode: string,
  client_id: string,
  name?: string | null
): Promise<LoginResponse> {
  return encryptedApiCall<
    {
      email: string;
      password: string;
      inviteCode: string;
      name?: string | null;
      client_id: string;
    },
    LoginResponse
  >(`${apiUrl}/register`, "POST", {
    email,
    password,
    inviteCode: inviteCode.toLowerCase(),
    client_id,
    name
  });
}

export async function fetchGuestSignUp(
  password: string,
  inviteCode: string,
  client_id: string
): Promise<LoginResponse> {
  return encryptedApiCall<
    { password: string; inviteCode: string; client_id: string },
    LoginResponse
  >(`${apiUrl}/register`, "POST", {
    password,
    inviteCode: inviteCode.toLowerCase(),
    client_id
  });
}

export async function refreshToken(): Promise<RefreshResponse> {
  const refresh_token = window.localStorage.getItem("refresh_token");
  if (!refresh_token) throw new Error("No refresh token available");

  const refreshData = { refresh_token };

  try {
    const response = await encryptedApiCall<typeof refreshData, RefreshResponse>(
      `${apiUrl}/refresh`,
      "POST",
      refreshData,
      undefined,
      "Failed to refresh token"
    );

    window.localStorage.setItem("access_token", response.access_token);
    window.localStorage.setItem("refresh_token", response.refresh_token);
    return response;
  } catch (error) {
    console.error("Error refreshing token:", error);
    throw error;
  }
}

export async function fetchUser(): Promise<UserResponse> {
  return authenticatedApiCall<void, UserResponse>(
    `${apiUrl}/protected/user`,
    "GET",
    undefined,
    "Failed to fetch user"
  );
}

export async function fetchPut(key: string, value: string): Promise<string> {
  return authenticatedApiCall<string, string>(
    `${apiUrl}/protected/kv/${key}`,
    "PUT",
    value,
    "Failed to put key-value pair"
  );
}

export async function fetchDelete(key: string): Promise<void> {
  return authenticatedApiCall<void, void>(
    `${apiUrl}/protected/kv/${key}`,
    "DELETE",
    undefined,
    "Failed to delete key-value pair"
  );
}

export async function fetchGet(key: string): Promise<string | undefined> {
  try {
    const data = await authenticatedApiCall<void, string>(
      `${apiUrl}/protected/kv/${key}`,
      "GET",
      undefined,
      "Failed to get key-value pair"
    );
    return data;
  } catch (error) {
    console.error(`Error fetching key "${key}":`, error);
    return undefined;
  }
}

export async function fetchList(): Promise<KVListItem[]> {
  return authenticatedApiCall<void, KVListItem[]>(
    `${apiUrl}/protected/kv`,
    "GET",
    undefined,
    "Failed to list key-value pairs"
  );
}

export async function fetchLogout(refresh_token: string): Promise<void> {
  const refreshData = { refresh_token };
  return encryptedApiCall<typeof refreshData, void>(`${apiUrl}/logout`, "POST", refreshData);
}

export async function verifyEmail(code: string): Promise<void> {
  return encryptedApiCall<void, void>(
    `${apiUrl}/verify-email/${code}`,
    "GET",
    undefined,
    undefined,
    "Failed to verify email"
  );
}

export async function requestNewVerificationCode(): Promise<void> {
  return authenticatedApiCall<void, void>(
    `${apiUrl}/protected/request_verification`,
    "POST",
    undefined,
    "Failed to request new verification code"
  );
}

export async function fetchAttestationDocument(
  nonce: string,
  explicitApiUrl?: string
): Promise<string> {
  const url = explicitApiUrl || apiUrl;
  const response = await fetch(`${url}/attestation/${nonce}`);
  if (!response.ok) {
    throw new Error(`Request failed with status ${response.status}`);
  }
  const data = await response.json();
  return data.attestation_document;
}

export async function keyExchange(
  clientPublicKey: string,
  nonce: string,
  explicitApiUrl?: string
): Promise<{ encrypted_session_key: string; session_id: string }> {
  const url = explicitApiUrl || apiUrl;
  const response = await fetch(`${url}/key_exchange`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify({ client_public_key: clientPublicKey, nonce })
  });

  if (!response.ok) {
    throw new Error("Key exchange failed");
  }

  return response.json();
}

export async function requestPasswordReset(
  email: string,
  hashedSecret: string,
  client_id: string
): Promise<void> {
  const resetData = {
    email,
    hashed_secret: hashedSecret,
    client_id
  };
  return encryptedApiCall<typeof resetData, void>(
    `${apiUrl}/password-reset/request`,
    "POST",
    resetData,
    undefined,
    "Failed to request password reset"
  );
}

export async function confirmPasswordReset(
  email: string,
  alphanumericCode: string,
  plaintextSecret: string,
  newPassword: string,
  client_id: string
): Promise<void> {
  const confirmData = {
    email,
    alphanumeric_code: alphanumericCode,
    plaintext_secret: plaintextSecret,
    new_password: newPassword,
    client_id
  };
  return encryptedApiCall<typeof confirmData, void>(
    `${apiUrl}/password-reset/confirm`,
    "POST",
    confirmData,
    undefined,
    "Failed to confirm password reset"
  );
}

export async function changePassword(currentPassword: string, newPassword: string): Promise<void> {
  const changePasswordData = {
    current_password: currentPassword,
    new_password: newPassword
  };
  return authenticatedApiCall<typeof changePasswordData, void>(
    `${apiUrl}/protected/change_password`,
    "POST",
    changePasswordData,
    "Failed to change password"
  );
}

export async function initiateGitHubAuth(
  client_id: string,
  inviteCode?: string
): Promise<GithubAuthResponse> {
  try {
    return await encryptedApiCall<{ invite_code?: string; client_id: string }, GithubAuthResponse>(
      `${apiUrl}/auth/github`,
      "POST",
      inviteCode ? { invite_code: inviteCode, client_id } : { client_id },
      undefined,
      "Failed to initiate GitHub auth"
    );
  } catch (error) {
    if (error instanceof Error) {
      if (error.message.includes("Invalid invite code")) {
        throw new Error("Invalid invite code. Please check and try again.");
      }
    }
    throw error;
  }
}

export async function handleGitHubCallback(
  code: string,
  state: string,
  inviteCode: string
): Promise<LoginResponse> {
  const callbackData = { code, state, invite_code: inviteCode };
  try {
    return await encryptedApiCall<typeof callbackData, LoginResponse>(
      `${apiUrl}/auth/github/callback`,
      "POST",
      callbackData,
      undefined,
      "GitHub callback failed"
    );
  } catch (error) {
    console.error("Detailed GitHub callback error:", error);
    if (error instanceof Error) {
      if (
        error.message.includes("User exists") ||
        error.message.includes("Email already registered")
      ) {
        throw new Error(
          "An account with this email already exists. Please sign in using your existing account."
        );
      } else if (error.message.includes("Invalid invite code")) {
        throw new Error("Invalid invite code. Please try signing up with a valid invite code.");
      } else if (error.message.includes("User not found")) {
        throw new Error(
          "User not found. Please sign up first before attempting to log in with GitHub."
        );
      } else {
        throw new Error("Failed to authenticate with GitHub. Please try again.");
      }
    }
    throw error;
  }
}

export type GithubAuthResponse = {
  auth_url: string;
  csrf_token: string;
};

export type GoogleAuthResponse = {
  auth_url: string;
  csrf_token: string;
};

/**
 * Response from initiating Apple OAuth authentication
 * @property auth_url - The Apple authorization URL to redirect the user to
 * @property state - The state parameter used to prevent CSRF attacks
 */
export type AppleAuthResponse = {
  auth_url: string;
  state: string;
};

export async function initiateGoogleAuth(
  client_id: string,
  inviteCode?: string
): Promise<GoogleAuthResponse> {
  try {
    return await encryptedApiCall<{ invite_code?: string; client_id: string }, GoogleAuthResponse>(
      `${apiUrl}/auth/google`,
      "POST",
      inviteCode ? { invite_code: inviteCode, client_id } : { client_id },
      undefined,
      "Failed to initiate Google auth"
    );
  } catch (error) {
    if (error instanceof Error) {
      if (error.message.includes("Invalid invite code")) {
        throw new Error("Invalid invite code. Please check and try again.");
      }
    }
    throw error;
  }
}

export async function handleGoogleCallback(
  code: string,
  state: string,
  inviteCode: string
): Promise<LoginResponse> {
  const callbackData = { code, state, invite_code: inviteCode };
  try {
    return await encryptedApiCall<typeof callbackData, LoginResponse>(
      `${apiUrl}/auth/google/callback`,
      "POST",
      callbackData,
      undefined,
      "Google callback failed"
    );
  } catch (error) {
    console.error("Detailed Google callback error:", error);
    if (error instanceof Error) {
      if (
        error.message.includes("User exists") ||
        error.message.includes("Email already registered")
      ) {
        throw new Error(
          "An account with this email already exists. Please sign in using your existing account."
        );
      } else if (error.message.includes("Invalid invite code")) {
        throw new Error("Invalid invite code. Please try signing up with a valid invite code.");
      } else if (error.message.includes("User not found")) {
        throw new Error(
          "User not found. Please sign up first before attempting to log in with Google."
        );
      } else {
        throw new Error("Failed to authenticate with Google. Please try again.");
      }
    }
    throw error;
  }
}

/**
 * Initiates Apple OAuth authentication flow
 * @param client_id - The client ID for your OpenSecret project
 * @param inviteCode - Optional invite code for new user registration
 * @returns A promise resolving to the Apple auth response containing auth URL and state
 * @description
 * This function starts the Apple OAuth authentication process by:
 * 1. Generating a secure state parameter to prevent CSRF attacks
 * 2. Getting an authorization URL from the OpenSecret backend
 * 3. Returning the URL that the client should redirect to
 *
 * After the user authenticates with Apple, they will be redirected back to your application.
 * The handleAppleCallback function should be used to complete the authentication process.
 */
export async function initiateAppleAuth(
  client_id: string,
  inviteCode?: string
): Promise<AppleAuthResponse> {
  try {
    return await encryptedApiCall<{ invite_code?: string; client_id: string }, AppleAuthResponse>(
      `${apiUrl}/auth/apple`,
      "POST",
      inviteCode ? { invite_code: inviteCode, client_id } : { client_id },
      undefined,
      "Failed to initiate Apple auth"
    );
  } catch (error) {
    if (error instanceof Error) {
      if (error.message.includes("Invalid invite code")) {
        throw new Error("Invalid invite code. Please check and try again.");
      }
    }
    throw error;
  }
}

/**
 * Completes Apple OAuth authentication after user is redirected back to your app
 * @param code - The authorization code from Apple
 * @param state - The state parameter returned by Apple (should match the original state)
 * @param inviteCode - Invite code for new user registration
 * @returns A promise resolving to login response with access and refresh tokens
 * @description
 * This function completes the Apple OAuth authentication process by:
 * 1. Validating the state parameter to prevent CSRF attacks
 * 2. Exchanging the authorization code for tokens
 * 3. Creating or authenticating the user account
 *
 * This function should be called in your OAuth callback route after
 * the user is redirected back from Apple's authentication page.
 */
export async function handleAppleCallback(
  code: string,
  state: string,
  inviteCode: string
): Promise<LoginResponse> {
  const callbackData = { code, state, invite_code: inviteCode };
  try {
    return await encryptedApiCall<typeof callbackData, LoginResponse>(
      `${apiUrl}/auth/apple/callback`,
      "POST",
      callbackData,
      undefined,
      "Apple callback failed"
    );
  } catch (error) {
    console.error("Detailed Apple callback error:", error);
    if (error instanceof Error) {
      if (
        error.message.includes("User exists") ||
        error.message.includes("Email already registered")
      ) {
        throw new Error(
          "An account with this email already exists. Please sign in using your existing account."
        );
      } else if (error.message.includes("Invalid invite code")) {
        throw new Error("Invalid invite code. Please try signing up with a valid invite code.");
      } else if (error.message.includes("User not found")) {
        throw new Error(
          "User not found. Please sign up first before attempting to log in with Apple."
        );
      } else {
        throw new Error("Failed to authenticate with Apple. Please try again.");
      }
    }
    throw error;
  }
}

/**
 * Apple user information returned from native Apple Sign-In
 * @property user_identifier - The user's unique ID from Apple
 * @property identity_token - The JWT token from Apple used for authentication
 * @property email - Optional email address (only provided on first sign-in)
 * @property given_name - Optional user's first name (only provided on first sign-in)
 * @property family_name - Optional user's last name (only provided on first sign-in)
 * @property nonce - Optional nonce for preventing replay attacks
 */
export type AppleUser = {
  user_identifier: string; // The user's unique ID from Apple
  identity_token: string; // The JWT token from Apple
  email?: string; // Optional: only provided on first sign-in
  given_name?: string; // Optional: only provided on first sign-in
  family_name?: string; // Optional: only provided on first sign-in
  nonce?: string; // Optional: nonce for validating the token
};

/**
 * Handles native Apple Sign-In for iOS devices
 * @param appleUser - Apple user data from the native Sign in with Apple API
 * @param client_id - The client ID for your OpenSecret project
 * @param inviteCode - Optional invite code for new user registration
 * @returns A promise resolving to login response with access and refresh tokens
 * @description
 * This function is specifically for use with iOS native Sign in with Apple:
 * 1. Validates the Apple identity token and user information
 * 2. Creates or authenticates the user account
 * 3. Returns authentication tokens
 *
 * Unlike OAuth flow, this method doesn't require redirects and is used
 * directly with the credential data from Apple's native authentication.
 *
 * Note: Email and name information are only provided by Apple on the first
 * authentication. Your backend should store this information for future use.
 *
 * The nonce parameter (optional) can be provided as part of the appleUser object.
 * When using Sign in with Apple, you can generate a nonce on your client and pass
 * it both to Apple during authentication initiation and to this function for validation.
 * The backend will verify that the nonce in the JWT matches what was provided.
 */
export async function handleAppleNativeSignIn(
  appleUser: AppleUser,
  client_id: string,
  inviteCode?: string
): Promise<LoginResponse> {
  // Combine the Apple user data with our app's client ID
  const signInData = {
    ...appleUser,
    client_id,
    ...(inviteCode ? { invite_code: inviteCode } : {})
  };

  try {
    return await encryptedApiCall<typeof signInData, LoginResponse>(
      `${apiUrl}/auth/apple/native`,
      "POST",
      signInData,
      undefined,
      "Apple Sign-In failed"
    );
  } catch (error) {
    console.error("Detailed Apple Sign-In error:", error);
    if (error instanceof Error) {
      if (
        error.message.includes("User exists") ||
        error.message.includes("Email already registered")
      ) {
        throw new Error(
          "An account with this email already exists. Please sign in using your existing account."
        );
      } else if (error.message.includes("Invalid invite code")) {
        throw new Error("Invalid invite code. Please try signing up with a valid invite code.");
      } else if (error.message.includes("User not found")) {
        throw new Error(
          "User not found. Please sign up first before attempting to log in with Apple."
        );
      } else if (error.message.includes("No email found")) {
        throw new Error("Unable to retrieve email from Apple. Please try another sign-in method.");
      } else {
        throw new Error("Failed to authenticate with Apple. Please try again.");
      }
    }
    throw error;
  }
}

export type PrivateKeyResponse = {
  /** 12-word BIP39 mnemonic phrase */
  mnemonic: string;
};

export type PrivateKeyBytesResponse = {
  /** 32-byte hex string (64 characters) representing the private key */
  private_key: string;
};

/**
 * Options for key derivation operations
 */
export type KeyOptions = {
  /**
   * BIP-85 derivation path to derive a child mnemonic
   * Examples: "m/83696968'/39'/0'/12'/0'"
   */
  seed_phrase_derivation_path?: string;

  /**
   * BIP-32 derivation path to derive a child key from the master (or BIP-85 derived) seed
   * Examples: "m/44'/0'/0'/0/0"
   */
  private_key_derivation_path?: string;
};

/**
 * Fetches the private key as a mnemonic phrase with optional derivation paths
 * @param key_options - Optional key derivation options (see KeyOptions type)
 *
 * @returns A promise resolving to the private key response containing the mnemonic
 *
 * @description
 * - If seed_phrase_derivation_path is provided, a child mnemonic is derived via BIP-85
 * - If seed_phrase_derivation_path is omitted, the master mnemonic is returned
 * - BIP-85 allows deriving child mnemonics from a master seed in a deterministic way
 * - Common BIP-85 path format: m/83696968'/39'/0'/[entropy in bits]'/[index]'
 *   where entropy is typically 12' for 12-word mnemonics
 */
export async function fetchPrivateKey(key_options?: KeyOptions): Promise<PrivateKeyResponse> {
  // Build URL with query parameters
  let url = `${apiUrl}/protected/private_key`;
  const queryParams = [];

  // Add seed phrase derivation path if present
  if (key_options?.seed_phrase_derivation_path) {
    queryParams.push(
      `seed_phrase_derivation_path=${encodeURIComponent(key_options.seed_phrase_derivation_path)}`
    );
  }

  // Add private key derivation path if present
  if (key_options?.private_key_derivation_path) {
    queryParams.push(
      `private_key_derivation_path=${encodeURIComponent(key_options.private_key_derivation_path)}`
    );
  }

  // Append query parameters if any exist
  if (queryParams.length > 0) {
    url += `?${queryParams.join("&")}`;
  }

  return authenticatedApiCall<void, PrivateKeyResponse>(
    url,
    "GET",
    undefined,
    "Failed to fetch private key"
  );
}

/**
 * Fetches private key bytes for the given derivation options
 * @param key_options - Key derivation options (see KeyOptions type)
 *
 * @returns A promise resolving to the private key bytes response
 *
 * @description
 * This function supports multiple derivation approaches:
 *
 * 1. Master key only (no parameters)
 *    - Returns the master private key bytes
 *
 * 2. BIP-32 derivation only
 *    - Uses a single derivation path to derive a child key from the master seed
 *    - Supports both absolute and relative paths with hardened derivation:
 *      - Absolute path: "m/44'/0'/0'/0/0"
 *      - Relative path: "0'/0'/0'/0/0"
 *      - Hardened notation: "44'" or "44h"
 *    - Common paths:
 *      - BIP44 (Legacy): m/44'/0'/0'/0/0
 *      - BIP49 (SegWit): m/49'/0'/0'/0/0
 *      - BIP84 (Native SegWit): m/84'/0'/0'/0/0
 *      - BIP86 (Taproot): m/86'/0'/0'/0/0
 *
 * 3. BIP-85 derivation only
 *    - Derives a child mnemonic from the master seed using BIP-85
 *    - Then returns the master private key of that derived seed
 *    - Example path: "m/83696968'/39'/0'/12'/0'"
 *
 * 4. Combined BIP-85 and BIP-32 derivation
 *    - First derives a child mnemonic via BIP-85
 *    - Then applies BIP-32 derivation to that derived seed
 *    - This allows for more complex derivation schemes
 */
export async function fetchPrivateKeyBytes(
  key_options?: KeyOptions
): Promise<PrivateKeyBytesResponse> {
  // Build URL with query parameters
  let url = `${apiUrl}/protected/private_key_bytes`;
  const queryParams = [];

  // Add seed phrase derivation path if present
  if (key_options?.seed_phrase_derivation_path) {
    queryParams.push(
      `seed_phrase_derivation_path=${encodeURIComponent(key_options.seed_phrase_derivation_path)}`
    );
  }

  // Add private key derivation path if present
  if (key_options?.private_key_derivation_path) {
    queryParams.push(
      `private_key_derivation_path=${encodeURIComponent(key_options.private_key_derivation_path)}`
    );
  }

  // Append query parameters if any exist
  if (queryParams.length > 0) {
    url += `?${queryParams.join("&")}`;
  }

  return authenticatedApiCall<void, PrivateKeyBytesResponse>(
    url,
    "GET",
    undefined,
    "Failed to fetch private key bytes"
  );
}

export type SignMessageResponse = {
  /** Signature in hex format */
  signature: string;
  /** Message hash in hex format */
  message_hash: string;
};

type SigningAlgorithm = "schnorr" | "ecdsa";

export type SignMessageRequest = {
  /** Base64-encoded message to sign */
  message_base64: string;
  /** Signing algorithm to use (schnorr or ecdsa) */
  algorithm: SigningAlgorithm;
  /** Optional key derivation options */
  key_options?: {
    /** Optional BIP32 derivation path (e.g., "m/44'/0'/0'/0/0") */
    private_key_derivation_path?: string;
    /** Optional BIP-85 seed phrase derivation path (e.g., "m/83696968'/39'/0'/12'/0'") */
    seed_phrase_derivation_path?: string;
  };
};

/**
 * Signs a message using the specified algorithm and derivation options
 * @param message_bytes - Message to sign as Uint8Array
 * @param algorithm - Signing algorithm (schnorr or ecdsa)
 * @param key_options - Key derivation options (see KeyOptions type)
 *
 * @returns A promise resolving to the signature response
 *
 * @description
 * This function supports multiple signing approaches:
 *
 * 1. Sign with master key (no derivation parameters)
 *
 * 2. Sign with BIP-32 derived key
 *    - Derives a child key from the master seed using BIP-32
 *
 * 3. Sign with BIP-85 derived key
 *    - Derives a child mnemonic using BIP-85, then uses its master key
 *
 * 4. Sign with combined BIP-85 and BIP-32 derivation
 *    - First derives a child mnemonic via BIP-85
 *    - Then applies BIP-32 derivation to derive a key from that seed
 *
 * Example message preparation:
 * ```typescript
 * // From string
 * const messageBytes = new TextEncoder().encode("Hello, World!");
 *
 * // From hex
 * const messageBytes = new Uint8Array(Buffer.from("deadbeef", "hex"));
 * ```
 */
export async function signMessage(
  message_bytes: Uint8Array,
  algorithm: SigningAlgorithm,
  key_options?: KeyOptions
): Promise<SignMessageResponse> {
  const message_base64 = encode(message_bytes);

  const requestData = {
    message_base64,
    algorithm,
    ...(key_options && Object.keys(key_options).length > 0 && { key_options })
  };

  return authenticatedApiCall<typeof requestData, SignMessageResponse>(
    `${apiUrl}/protected/sign_message`,
    "POST",
    requestData,
    "Failed to sign message"
  );
}

export type PublicKeyResponse = {
  /** Public key in hex format */
  public_key: string;
  /** The algorithm used (schnorr or ecdsa) */
  algorithm: SigningAlgorithm;
};

/**
 * Retrieves the public key for a given algorithm and derivation options
 * @param algorithm - Signing algorithm (schnorr or ecdsa)
 * @param key_options - Key derivation options (see KeyOptions type)
 *
 * @returns A promise resolving to the public key response
 *
 * @description
 * The derivation paths determine which key is used to generate the public key:
 *
 * 1. Master key (no derivation parameters)
 *    - Returns the public key corresponding to the master private key
 *
 * 2. BIP-32 derived key
 *    - Returns the public key for a derived child key
 *    - Useful for:
 *      - Separating keys by purpose (e.g., different chains or applications)
 *      - Generating deterministic addresses
 *      - Supporting different address formats (Legacy, SegWit, Native SegWit, Taproot)
 *
 * 3. BIP-85 derived key
 *    - Returns the public key for the master key of a BIP-85 derived seed
 *
 * 4. Combined BIP-85 and BIP-32 derivation
 *    - First derives a child mnemonic via BIP-85
 *    - Then applies BIP-32 derivation to get the corresponding public key
 */
export async function fetchPublicKey(
  algorithm: SigningAlgorithm,
  key_options?: KeyOptions
): Promise<PublicKeyResponse> {
  // Build URL with query parameters
  let url = `${apiUrl}/protected/public_key?algorithm=${algorithm}`;

  // Add seed phrase derivation path if present
  if (key_options?.seed_phrase_derivation_path) {
    url += `&seed_phrase_derivation_path=${encodeURIComponent(key_options.seed_phrase_derivation_path)}`;
  }

  // Add private key derivation path if present
  if (key_options?.private_key_derivation_path) {
    url += `&private_key_derivation_path=${encodeURIComponent(key_options.private_key_derivation_path)}`;
  }

  return authenticatedApiCall<void, PublicKeyResponse>(
    url,
    "GET",
    undefined,
    "Failed to fetch public key"
  );
}

export async function convertGuestToEmailAccount(
  email: string,
  password: string,
  name?: string | null
): Promise<void> {
  const conversionData = {
    email,
    password,
    ...(name !== undefined && { name })
  };

  return authenticatedApiCall<typeof conversionData, void>(
    `${apiUrl}/protected/convert_guest`,
    "POST",
    conversionData,
    "Failed to convert guest account"
  );
}

export type ThirdPartyTokenRequest = {
  audience?: string;
};

export type ThirdPartyTokenResponse = {
  token: string;
};

/**
 * Generates a JWT token for use with third-party services
 * @param audience - Optional URL of the service
 * @returns A promise resolving to the token response containing the JWT
 *
 * @description
 * - If audience is provided, it can be any valid URL
 * - If audience is omitted, a token with no audience restriction will be generated
 */
export async function generateThirdPartyToken(audience?: string): Promise<ThirdPartyTokenResponse> {
  return authenticatedApiCall<ThirdPartyTokenRequest, ThirdPartyTokenResponse>(
    `${apiUrl}/protected/third_party_token`,
    "POST",
    audience ? { audience } : {},
    "Failed to generate third party token"
  );
}

export type EncryptDataRequest = {
  data: string;
  key_options?: {
    private_key_derivation_path?: string;
    seed_phrase_derivation_path?: string;
  };
};

export type EncryptDataResponse = {
  encrypted_data: string;
};

/**
 * Encrypts arbitrary string data using the user's private key
 * @param data - String content to be encrypted
 * @param key_options - Key derivation options (see KeyOptions type)
 * @returns A promise resolving to the encrypted data response
 * @throws {Error} If:
 * - The derivation paths are invalid
 * - Authentication fails
 * - Server-side encryption error occurs
 *
 * @description
 * This function supports multiple encryption approaches:
 *
 * 1. Encrypt with master key (no derivation parameters)
 *
 * 2. Encrypt with BIP-32 derived key
 *    - Derives a child key from the master seed using BIP-32
 *
 * 3. Encrypt with BIP-85 derived key
 *    - Derives a child mnemonic using BIP-85, then uses its master key
 *
 * 4. Encrypt with combined BIP-85 and BIP-32 derivation
 *    - First derives a child mnemonic via BIP-85
 *    - Then applies BIP-32 derivation to derive a key from that seed
 *
 * Technical details:
 * - Encrypts data with AES-256-GCM
 * - A random nonce is generated for each encryption operation (included in the result)
 * - The encrypted_data format:
 *   - First 12 bytes: Nonce used for encryption (prepended to ciphertext)
 *   - Remaining bytes: AES-256-GCM encrypted ciphertext + authentication tag
 *   - The entire payload is base64-encoded for safe transport
 */
export async function encryptData(
  data: string,
  key_options?: KeyOptions
): Promise<EncryptDataResponse> {
  const requestData = {
    data,
    ...(key_options && Object.keys(key_options).length > 0 && { key_options })
  };

  return authenticatedApiCall<typeof requestData, EncryptDataResponse>(
    `${apiUrl}/protected/encrypt`,
    "POST",
    requestData,
    "Failed to encrypt data"
  );
}

export type DecryptDataRequest = {
  encrypted_data: string;
  key_options?: {
    private_key_derivation_path?: string;
    seed_phrase_derivation_path?: string;
  };
};

/**
 * Decrypts data that was previously encrypted with the user's key
 * @param encryptedData - Base64-encoded encrypted data string
 * @param key_options - Key derivation options (see KeyOptions type)
 * @returns A promise resolving to the decrypted string
 * @throws {Error} If:
 * - The encrypted data is malformed
 * - The derivation paths are invalid
 * - Authentication fails
 * - Server-side decryption error occurs
 *
 * @description
 * This function supports multiple decryption approaches:
 *
 * 1. Decrypt with master key (no derivation parameters)
 *
 * 2. Decrypt with BIP-32 derived key
 *    - Derives a child key from the master seed using BIP-32
 *
 * 3. Decrypt with BIP-85 derived key
 *    - Derives a child mnemonic using BIP-85, then uses its master key
 *
 * 4. Decrypt with combined BIP-85 and BIP-32 derivation
 *    - First derives a child mnemonic via BIP-85
 *    - Then applies BIP-32 derivation to derive a key from that seed
 *
 * IMPORTANT: You must use the exact same derivation paths for decryption
 * that were used for encryption.
 *
 * Technical details:
 * - Uses AES-256-GCM decryption with authentication tag verification
 * - Extracts the nonce from the first 12 bytes of the encrypted data
 * - The encrypted_data must be in the exact format returned by the encrypt endpoint
 */
export async function decryptData(
  encryptedData: string,
  key_options?: KeyOptions
): Promise<string> {
  const requestData = {
    encrypted_data: encryptedData,
    ...(key_options && Object.keys(key_options).length > 0 && { key_options })
  };

  return authenticatedApiCall<typeof requestData, string>(
    `${apiUrl}/protected/decrypt`,
    "POST",
    requestData,
    "Failed to decrypt data"
  );
}

/**
 * Initiates the account deletion process for logged-in users
 * @param hashedSecret - Client-side hashed secret for verification
 * @returns A promise resolving to void
 *
 * @description
 * This function:
 * 1. Requires the user to be logged in (uses authenticatedApiCall)
 * 2. Sends a verification email to the user's email address
 * 3. The email contains a confirmation code that will be needed for confirmation
 * 4. The client must store the plaintext secret for confirmation
 */
export async function requestAccountDeletion(hashedSecret: string): Promise<void> {
  const deleteData = {
    hashed_secret: hashedSecret
  };
  return authenticatedApiCall<typeof deleteData, void>(
    `${apiUrl}/protected/delete-account/request`,
    "POST",
    deleteData,
    "Failed to request account deletion"
  );
}

/**
 * Confirms and completes the account deletion process
 * @param confirmationCode - The confirmation code from the verification email
 * @param plaintextSecret - The plaintext secret that was hashed in the request step
 * @returns A promise resolving to void
 *
 * @description
 * This function:
 * 1. Requires the user to be logged in (uses authenticatedApiCall)
 * 2. Verifies both the confirmation code from email and the secret known only to the client
 * 3. Permanently deletes the user account and all associated data
 * 4. After successful deletion, the client should clear all local storage and tokens
 */
export async function confirmAccountDeletion(
  confirmationCode: string,
  plaintextSecret: string
): Promise<void> {
  const confirmData = {
    confirmation_code: confirmationCode,
    plaintext_secret: plaintextSecret
  };
  return authenticatedApiCall<typeof confirmData, void>(
    `${apiUrl}/protected/delete-account/confirm`,
    "POST",
    confirmData,
    "Failed to confirm account deletion"
  );
}

type ModelsListResponse = {
  object: "list";
  data: Model[];
};

/**
 * Fetches available AI models from the OpenAI-compatible API
 * @returns A promise resolving to an array of Model objects
 * @throws {Error} If:
 * - The user is not authenticated
 * - The request fails
 * - The response format is invalid
 */
export async function fetchModels(): Promise<Model[]> {
  try {
    const response = await authenticatedApiCall<void, ModelsListResponse>(
      `${apiUrl}/v1/models`,
      "GET",
      undefined,
      "Failed to fetch models"
    );

    // Validate response structure
    if (!response || typeof response !== "object") {
      throw new Error("Invalid response from models endpoint");
    }

    if (response.object !== "list" || !Array.isArray(response.data)) {
      throw new Error("Models response missing expected 'object' or 'data' fields");
    }

    return response.data;
  } catch (error) {
    console.error("Error fetching models:", error);
    throw error;
  }
}

export type DocumentUploadRequest = {
  filename: string;
  content_base64: string;
};

export type DocumentResponse = {
  text: string;
  filename: string;
  size: number;
};

const MAX_FILE_SIZE = 10 * 1024 * 1024; // 10MB

/**
 * Uploads a document for text extraction and processing
 * @param file - The file to upload (File or Blob object)
 * @returns A promise resolving to the extracted document text and metadata
 * @throws {Error} If:
 * - The file exceeds 10MB size limit
 * - The user is not authenticated
 * - The user is a guest (401)
 * - Usage limits are exceeded (403)
 * - Processing fails (500)
 *
 * @description
 * This function uploads a document to the Tinfoil processing service which:
 * 1. Extracts text from various document formats (PDF, DOCX, TXT, etc.)
 * 2. Returns the extracted text ready for use in chat prompts
 * 3. Maintains end-to-end encryption using session keys
 *
 * The file is converted to base64 before upload due to encryption requirements.
 * Common supported formats include PDF, DOCX, XLSX, PPTX, TXT, RTF, and more.
 *
 * Example usage:
 * ```typescript
 * const file = new File(["content"], "document.pdf", { type: "application/pdf" });
 * const result = await uploadDocument(file);
 * console.log(result.text); // Extracted text from the document
 * ```
 */
export async function uploadDocument(file: File | Blob): Promise<DocumentResponse> {
  // Validate file size
  if (file.size > MAX_FILE_SIZE) {
    throw new Error(`File size exceeds maximum limit of ${MAX_FILE_SIZE / 1024 / 1024}MB`);
  }

  // Convert file to base64
  const arrayBuffer = await file.arrayBuffer();
  const bytes = new Uint8Array(arrayBuffer);
  const base64Data = encode(bytes);

  // Get filename
  const filename = file instanceof File ? file.name : "document";

  const requestData: DocumentUploadRequest = {
    filename,
    content_base64: base64Data
  };

  return authenticatedApiCall<DocumentUploadRequest, DocumentResponse>(
    `${apiUrl}/v1/documents/upload`,
    "POST",
    requestData,
    "Failed to upload document"
  );
}
