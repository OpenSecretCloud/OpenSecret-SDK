import { encode } from "@stablelib/base64";
import { authenticatedApiCall, encryptedApiCall } from "./encryptedApi";

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
