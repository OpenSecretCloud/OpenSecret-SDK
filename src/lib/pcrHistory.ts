import { decode } from "@stablelib/base64";
import { Pcr0ValidationResult, PcrConfig } from "./pcr";

// The PCR verification public key provided in DER format, base64-encoded
const PCR_VERIFICATION_PUBLIC_KEY_B64 =
  "MHYwEAYHKoZIzj0CAQYFK4EEACIDYgAEsT4fLLWwA2IyUQbRjhsjz46Ts14mxVzvu8eC68rM7r9b3tZ1yYX311WaQcDOhNbT5vCYivkqA0EXN3aDFSmXHyFzKKxqyOEGBgnRxSBpMQNrc2yumBMDvseiEdCSpQwR";

// Interface for PCR history entries
export interface PcrEntry {
  HashAlgorithm: string;
  PCR0: string;
  PCR1: string;
  PCR2: string;
  timestamp: number;
  signature: string;
}

// Cache for PCR history to avoid repeated network requests
const pcrHistoryCache: Record<string, { timestamp: number; data: Array<PcrEntry> }> = {};

// Cache expiration time (15 minutes in milliseconds)
const CACHE_EXPIRATION = 15 * 60 * 1000;

/**
 * Clears the PCR history cache - primarily used for testing
 */
export function clearPcrHistoryCache(): void {
  Object.keys(pcrHistoryCache).forEach((key) => {
    delete pcrHistoryCache[key];
  });
}

/**
 * Fetch PCR history list from the repository with caching
 * @param env - Environment type ('dev' or 'prod')
 * @param customUrl - Optional custom URL for PCR history
 * @returns Promise of array of PCR entries
 */
export async function getPcrHistoryList(
  env: "dev" | "prod",
  customUrl?: string
): Promise<Array<PcrEntry>> {
  // If a custom URL is provided, use it. Otherwise use the default URLs.
  const url =
    customUrl ||
    (env === "dev"
      ? "https://raw.githubusercontent.com/OpenSecretCloud/opensecret/master/pcrDevHistory.json"
      : "https://raw.githubusercontent.com/OpenSecretCloud/opensecret/master/pcrProdHistory.json");

  // Check if we have a valid cached version
  const now = Date.now();
  if (pcrHistoryCache[url] && now - pcrHistoryCache[url].timestamp < CACHE_EXPIRATION) {
    return pcrHistoryCache[url].data;
  }

  try {
    const resp = await fetch(url);
    if (!resp.ok) {
      throw new Error(`Couldn't fetch PCR list: ${resp.status}`);
    }

    const data = await resp.json();

    // Cache the result
    pcrHistoryCache[url] = {
      timestamp: now,
      data
    };

    return data;
  } catch (error) {
    console.error("Error fetching PCR history:", error);

    // Return empty array, which will cause validation to fail
    return [];
  }
}

/**
 * Import the public key for PCR signature verification
 * @returns Promise resolving to CryptoKey
 */
export async function loadPcrPublicKey(): Promise<CryptoKey> {
  try {
    // Decode the base64 key
    const binaryDer = decode(PCR_VERIFICATION_PUBLIC_KEY_B64);

    // Import the key
    return await crypto.subtle.importKey(
      "spki", // SubjectPublicKeyInfo format
      binaryDer,
      {
        name: "ECDSA",
        namedCurve: "P-384" // Must match the curve used to generate the key
      },
      true, // extractable
      ["verify"] // only need verification
    );
  } catch (error) {
    console.error("Error loading PCR verification public key:", error);
    throw new Error("Failed to load PCR verification key");
  }
}

/**
 * Verify the signature of a PCR entry
 * @param entry - PCR entry to verify
 * @param publicKey - CryptoKey for verification
 * @returns Promise resolving to boolean indicating if signature is valid
 */
export async function verifyPcrSignature(entry: PcrEntry, publicKey: CryptoKey): Promise<boolean> {
  try {
    const encoder = new TextEncoder();

    // Create the same message format that was signed on the backend
    // Note: To match signature implementation, we need to exclude the signature field
    const dataToSign = {
      HashAlgorithm: entry.HashAlgorithm,
      PCR0: entry.PCR0,
      PCR1: entry.PCR1,
      PCR2: entry.PCR2,
      timestamp: entry.timestamp
    };

    // Convert to JSON string with deterministic ordering to match backend signature
    // This ensures key ordering is consistent for proper signature verification
    const orderedJson = JSON.stringify(dataToSign, Object.keys(dataToSign).sort());
    const message = encoder.encode(orderedJson);

    // Decode the base64 signature
    const signatureBuf = decode(entry.signature);

    // Verify using Web Crypto API
    return await crypto.subtle.verify(
      { name: "ECDSA", hash: { name: "SHA-384" } },
      publicKey,
      signatureBuf,
      message
    );
  } catch (error) {
    console.error("Error verifying PCR signature:", error);
    return false;
  }
}

/**
 * Validates PCR values against history
 * @param pcr - PCR values to validate
 * @param config - Optional PCR configuration
 * @returns Promise resolving to boolean indicating if PCRs are valid
 */
export async function validatePcrAgainstHistory(
  pcr: { PCR0: string; PCR1: string; PCR2: string },
  config?: PcrConfig
): Promise<Pcr0ValidationResult> {
  try {
    // Determine environment (dev or prod) based on where the SDK is running
    const env = isDev() ? "dev" : "prod";

    // Fetch the history
    const history = await getPcrHistoryList(env, config?.remoteValidationUrl);

    if (history.length === 0) {
      return {
        isMatch: false,
        text: "Couldn't validate against PCR history"
      };
    }

    // Load public key for verification
    const publicKey = await loadPcrPublicKey();

    for (const entry of history) {
      // Check if PCRs match
      if (entry.PCR0 === pcr.PCR0 && entry.PCR1 === pcr.PCR1 && entry.PCR2 === pcr.PCR2) {
        // Verify signature
        const isValid = await verifyPcrSignature(entry, publicKey);
        if (isValid) {
          return {
            isMatch: true,
            text: "PCR matches history with valid signature"
          };
        }
      }
    }

    return {
      isMatch: false,
      text: "PCR not found in verified history"
    };
  } catch (error) {
    console.error("Error validating PCR against history:", error);
    return {
      isMatch: false,
      text: "Error validating PCR against history"
    };
  }
}

/**
 * Checks if the current environment is development
 * @returns true if running in development environment
 */
function isDev(): boolean {
  // Check if the API URL contains development indicators
  const apiUrl = window.localStorage.getItem("api_url") || "";
  return (
    apiUrl.includes("localhost") ||
    apiUrl.includes("127.0.0.1") ||
    apiUrl.includes("dev.") ||
    apiUrl.includes(".dev")
  );
}
