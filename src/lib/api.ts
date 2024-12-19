import { encode } from "@stablelib/base64";
import { authenticatedApiCall, encryptedApiCall } from "./encryptedApi";

let API_URL = "http://localhost:3000";

console.log("API_URL:", API_URL);

export function setApiUrl(url: string) {
  API_URL = url;
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
  password: string
): Promise<LoginResponse> {
  return encryptedApiCall<{ email: string; password: string }, LoginResponse>(
    `${API_URL}/login`,
    "POST",
    { email, password }
  );
}

export async function fetchGuestLogin(
  id: string,
  password: string
): Promise<LoginResponse> {
  return encryptedApiCall<{ id: string; password: string }, LoginResponse>(
    `${API_URL}/login`,
    "POST",
    { id, password }
  );
}

export async function fetchSignUp(
  email: string,
  password: string,
  inviteCode: string,
  name?: string | null
): Promise<LoginResponse> {
  return encryptedApiCall<
    { email: string; password: string; inviteCode: string; name?: string | null },
    LoginResponse
  >(`${API_URL}/register`, "POST", {
    email,
    password,
    inviteCode: inviteCode.toLowerCase(),
    name
  });
}

export async function fetchGuestSignUp(
  password: string,
  inviteCode: string
): Promise<LoginResponse> {
  return encryptedApiCall<
    { password: string; inviteCode: string },
    LoginResponse
  >(`${API_URL}/register`, "POST", {
    password,
    inviteCode: inviteCode.toLowerCase()
  });
}

export async function refreshToken(): Promise<RefreshResponse> {
  const refresh_token = window.localStorage.getItem("refresh_token");
  if (!refresh_token) throw new Error("No refresh token available");

  const refreshData = { refresh_token };

  try {
    const response = await encryptedApiCall<typeof refreshData, RefreshResponse>(
      `${API_URL}/refresh`,
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
    `${API_URL}/protected/user`,
    "GET",
    undefined,
    "Failed to fetch user"
  );
}

export async function fetchPut(key: string, value: string): Promise<string> {
  return authenticatedApiCall<string, string>(
    `${API_URL}/protected/kv/${key}`,
    "PUT",
    value,
    "Failed to put key-value pair"
  );
}

export async function fetchDelete(key: string): Promise<void> {
  return authenticatedApiCall<void, void>(
    `${API_URL}/protected/kv/${key}`,
    "DELETE",
    undefined,
    "Failed to delete key-value pair"
  );
}

export async function fetchGet(key: string): Promise<string | undefined> {
  try {
    const data = await authenticatedApiCall<void, string>(
      `${API_URL}/protected/kv/${key}`,
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
    `${API_URL}/protected/kv`,
    "GET",
    undefined,
    "Failed to list key-value pairs"
  );
}

export async function fetchLogout(refresh_token: string): Promise<void> {
  const refreshData = { refresh_token };
  return encryptedApiCall<typeof refreshData, void>(`${API_URL}/logout`, "POST", refreshData);
}

export async function verifyEmail(code: string): Promise<void> {
  return encryptedApiCall<void, void>(
    `${API_URL}/verify-email/${code}`,
    "GET",
    undefined,
    undefined,
    "Failed to verify email"
  );
}

export async function requestNewVerificationCode(): Promise<void> {
  return authenticatedApiCall<void, void>(
    `${API_URL}/protected/request_verification`,
    "POST",
    undefined,
    "Failed to request new verification code"
  );
}

export async function fetchAttestationDocument(nonce: string): Promise<string> {
  const response = await fetch(`${API_URL}/attestation/${nonce}`);
  if (!response.ok) {
    throw new Error(`Request failed with status ${response.status}`);
  }
  const data = await response.json();
  return data.attestation_document;
}

export async function keyExchange(
  clientPublicKey: string,
  nonce: string
): Promise<{ encrypted_session_key: string; session_id: string }> {
  const response = await fetch(`${API_URL}/key_exchange`, {
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

export async function requestPasswordReset(email: string, hashedSecret: string): Promise<void> {
  const resetData = { email, hashed_secret: hashedSecret };
  return encryptedApiCall<typeof resetData, void>(
    `${API_URL}/password-reset/request`,
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
  newPassword: string
): Promise<void> {
  const confirmData = {
    email,
    alphanumeric_code: alphanumericCode,
    plaintext_secret: plaintextSecret,
    new_password: newPassword
  };
  return encryptedApiCall<typeof confirmData, void>(
    `${API_URL}/password-reset/confirm`,
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
    `${API_URL}/protected/change_password`,
    "POST",
    changePasswordData,
    "Failed to change password"
  );
}

export async function initiateGitHubAuth(inviteCode?: string): Promise<GithubAuthResponse> {
  try {
    return await encryptedApiCall<{ invite_code?: string }, GithubAuthResponse>(
      `${API_URL}/auth/github`,
      "POST",
      inviteCode ? { invite_code: inviteCode } : {},
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
      `${API_URL}/auth/github/callback`,
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

export async function initiateGoogleAuth(inviteCode?: string): Promise<GoogleAuthResponse> {
  try {
    return await encryptedApiCall<{ invite_code?: string }, GoogleAuthResponse>(
      `${API_URL}/auth/google`,
      "POST",
      inviteCode ? { invite_code: inviteCode } : {},
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
      `${API_URL}/auth/google/callback`,
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
  mnemonic: string;
};

export async function fetchPrivateKey(): Promise<PrivateKeyResponse> {
  return authenticatedApiCall<void, PrivateKeyResponse>(
    `${API_URL}/protected/private_key`,
    "GET",
    undefined,
    "Failed to fetch private key"
  );
}

export type SignMessageResponse = {
  signature: string;
  message_hash: string;
};

type SigningAlgorithm = "schnorr" | "ecdsa"

export async function signMessage(message_bytes: Uint8Array, algorithm: SigningAlgorithm): Promise<SignMessageResponse> {
  const message_base64 = encode(message_bytes);
  return authenticatedApiCall<{message_base64: string, algorithm: SigningAlgorithm}, SignMessageResponse>(
    `${API_URL}/protected/sign_message`,
    "POST",
    { 
      message_base64,
      algorithm
    },
    "Failed to sign message"
  );
}

export type PublicKeyResponse = {
  public_key: string;
  algorithm: SigningAlgorithm;
};

export async function fetchPublicKey(algorithm: SigningAlgorithm): Promise<PublicKeyResponse> {
  return authenticatedApiCall<void, PublicKeyResponse>(
    `${API_URL}/protected/public_key?algorithm=${algorithm}`,
    "GET",
    undefined,
    "Failed to fetch public key"
  );
}

export async function convertGuestToEmailAccount(
  email: string,
  password: string,
  name?: string
): Promise<void> {
  const conversionData = {
    email,
    password,
    name
  };

  return authenticatedApiCall<typeof conversionData, void>(
    `${API_URL}/protected/convert_guest`,
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
    `${API_URL}/protected/third_party_token`,
    "POST",
    { audience },
    "Failed to generate third party token"
  );
}
