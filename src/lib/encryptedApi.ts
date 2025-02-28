import { encryptMessage, decryptMessage } from "./encryption";
import { getAttestation } from "./getAttestation";
import { refreshToken } from "./api";
import { platformRefreshToken } from "./platformApi";

interface EncryptedResponse {
  encrypted: string;
}

interface ApiResponse<T> {
  status: number;
  data?: T;
  error?: string;
}

export async function authenticatedApiCall<T, U>(
  url: string,
  method: string,
  data: T,
  errorMessage?: string
): Promise<U> {
  const tryAuthenticatedRequest = async (forceRefresh: boolean = false): Promise<U> => {
    try {
      if (forceRefresh) {
        console.log("Refreshing access token");
        // Determine which refresh function to use based on the URL
        // If it's a platform API call, use platformRefreshToken, otherwise use regular refreshToken
        if (url.includes("/platform/")) {
          console.log("Using platform refresh token");
          await platformRefreshToken();
        } else {
          console.log("Using regular refresh token");
          await refreshToken();
        }
      }

      // Always get the latest token from localStorage
      const accessToken = window.localStorage.getItem("access_token");
      if (!accessToken) {
        throw new Error("No access token available");
      }

      const response = await internalEncryptedApiCall<T, U>(
        url,
        method,
        data,
        accessToken,
        errorMessage
      );

      // Attempt to refresh token once if we get a 401
      if (response.status === 401 && !forceRefresh) {
        console.log(`Received 401 for URL ${url}, attempting to refresh token`);
        return tryAuthenticatedRequest(true);
      }

      // Throw an error if the response contains an error message
      if (response.error) {
        throw new Error(response.error);
      }

      // Throw an error if no data was received
      if (!response.data) {
        throw new Error("No data received from the server");
      }

      return response.data;
    } catch (error) {
      console.error(error);
      throw error;
    }
  };

  return tryAuthenticatedRequest();
}

// This internal version can return a specific
async function internalEncryptedApiCall<T, U>(
  url: string,
  method: string,
  data: T,
  accessToken?: string,
  errorMessage?: string
): Promise<ApiResponse<U>> {
  // Check if we're using the platform API
  const isPlatformApiCall = url.includes("/platform/");
  const platformApiUrl = typeof window !== "undefined" ? window.__PLATFORM_API_URL__ : "";

  // Use the platform API URL for attestation if this is a platform API call
  const explicitApiUrl = isPlatformApiCall ? platformApiUrl : undefined;

  let { sessionKey, sessionId } = await getAttestation(false, explicitApiUrl);

  const makeRequest = async (token: string | undefined, forceNewAttestation: boolean = false) => {
    if (forceNewAttestation || !sessionKey || !sessionId) {
      const newAttestation = await getAttestation(true, explicitApiUrl);
      sessionKey = newAttestation.sessionKey;
      sessionId = newAttestation.sessionId;
    }

    if (!sessionKey || !sessionId) {
      throw new Error("Failed to make encrypted API call, no attestation available.");
    }

    const jsonData = data ? JSON.stringify(data) : undefined;
    const encryptedData = jsonData ? encryptMessage(sessionKey, jsonData) : undefined;

    const headers: Record<string, string> = {
      "Content-Type": "application/json",
      "x-session-id": sessionId
    };

    // Only add Authorization header if a token is provided
    if (token) {
      headers["Authorization"] = `Bearer ${token}`;
    }

    const response = await fetch(url, {
      method,
      headers,
      body: encryptedData ? JSON.stringify({ encrypted: encryptedData }) : undefined
    });

    const result: ApiResponse<U> = {
      status: response.status
    };

    if (!response.ok) {
      try {
        const errorBody = await response.json();
        result.error =
          errorBody.message || errorMessage || `HTTP error! Status: ${response.status}`;
      } catch {
        result.error = errorMessage || `HTTP error! Status: ${response.status}`;
      }
    } else {
      try {
        const encryptedResponse: EncryptedResponse = await response.json();
        const decryptedResponse = decryptMessage(sessionKey, encryptedResponse.encrypted);
        result.data = JSON.parse(decryptedResponse);
      } catch (error) {
        console.error("Error decrypting or parsing response:", error);
        result.status = 500;
        result.error = "Failed to decrypt or parse the response";
      }
    }

    return result;
  };

  const tryEncryptedRequest = async (
    token: string | undefined,
    forceNewAttestation: boolean = false
  ): Promise<ApiResponse<U>> => {
    try {
      const response = await makeRequest(token, forceNewAttestation);

      // Retry with new attestation if we get a 400 or encryption error, but only once
      if (response.status === 400 || response.error?.includes("Encryption error")) {
        if (!forceNewAttestation) {
          console.log("Encryption error or Bad Request, attempting to renew attestation");
          return tryEncryptedRequest(token, true);
        }
      }

      return response;
    } catch (error) {
      return {
        status: 500,
        error: error instanceof Error ? error.message : "Unknown error occurred"
      };
    }
  };

  return tryEncryptedRequest(accessToken);
}

export async function encryptedApiCall<T, U>(
  url: string,
  method: string,
  data: T,
  accessToken?: string,
  errorMessage?: string
): Promise<U> {
  const response = await internalEncryptedApiCall<T, U>(
    url,
    method,
    data,
    accessToken,
    errorMessage
  );

  // Throw an error if the response contains an error message
  if (response.error) {
    throw new Error(response.error);
  }

  // Throw an error if no data was received
  if (!response.data) {
    throw new Error("No data received from the server");
  }

  return response.data;
}
