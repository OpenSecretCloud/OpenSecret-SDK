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

export async function fetchAttestationDocument(nonce: string, explicitApiUrl?: string): Promise<string> {
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

export async function fetchPrivateKey(): Promise<PrivateKeyResponse> {
  return authenticatedApiCall<void, PrivateKeyResponse>(
    `${apiUrl}/protected/private_key`,
    "GET",
    undefined,
    "Failed to fetch private key"
  );
}

/**
 * Fetches private key bytes for a given derivation path
 * @param derivationPath - Optional BIP32 derivation path
 *
 * Supports both absolute and relative paths with hardened derivation:
 * - Absolute path: "m/44'/0'/0'/0/0"
 * - Relative path: "0'/0'/0'/0/0"
 * - Hardened notation: "44'" or "44h"
 *
 * Common paths:
 * - BIP44 (Legacy): m/44'/0'/0'/0/0
 * - BIP49 (SegWit): m/49'/0'/0'/0/0
 * - BIP84 (Native SegWit): m/84'/0'/0'/0/0
 * - BIP86 (Taproot): m/86'/0'/0'/0/0
 */
export async function fetchPrivateKeyBytes(
  derivationPath?: string
): Promise<PrivateKeyBytesResponse> {
  const url = derivationPath
    ? `${apiUrl}/protected/private_key_bytes?derivation_path=${encodeURIComponent(derivationPath)}`
    : `${apiUrl}/protected/private_key_bytes`;

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
  /** Optional BIP32 derivation path (e.g., "m/44'/0'/0'/0/0") */
  derivation_path?: string;
};

/**
 * Signs a message using the specified algorithm and derivation path
 * @param message_bytes - Message to sign as Uint8Array
 * @param algorithm - Signing algorithm (schnorr or ecdsa)
 * @param derivationPath - Optional BIP32 derivation path
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
  derivationPath?: string
): Promise<SignMessageResponse> {
  const message_base64 = encode(message_bytes);
  return authenticatedApiCall<SignMessageRequest, SignMessageResponse>(
    `${apiUrl}/protected/sign_message`,
    "POST",
    {
      message_base64,
      algorithm,
      derivation_path: derivationPath
    },
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
 * Retrieves the public key for a given algorithm and derivation path
 * @param algorithm - Signing algorithm (schnorr or ecdsa)
 * @param derivationPath - Optional BIP32 derivation path
 *
 * The derivation path determines which child key pair is used,
 * allowing different public keys to be generated from the same master key.
 * This is useful for:
 * - Separating keys by purpose (e.g., different chains or applications)
 * - Generating deterministic addresses
 * - Supporting different address formats (Legacy, SegWit, Native SegWit, Taproot)
 */
export async function fetchPublicKey(
  algorithm: SigningAlgorithm,
  derivationPath?: string
): Promise<PublicKeyResponse> {
  const url = derivationPath
    ? `${apiUrl}/protected/public_key?algorithm=${algorithm}&derivation_path=${encodeURIComponent(derivationPath)}`
    : `${apiUrl}/protected/public_key?algorithm=${algorithm}`;

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
  audience: string;
};

export type ThirdPartyTokenResponse = {
  token: string;
};

export async function generateThirdPartyToken(audience: string): Promise<ThirdPartyTokenResponse> {
  return authenticatedApiCall<ThirdPartyTokenRequest, ThirdPartyTokenResponse>(
    `${apiUrl}/protected/third_party_token`,
    "POST",
    { audience },
    "Failed to generate third party token"
  );
}
