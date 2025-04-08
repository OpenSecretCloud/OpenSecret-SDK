/**
 * Valid PCR0 values for production environments
 */
const DEFAULT_PCR0_VALUES = [
  "eeddbb58f57c38894d6d5af5e575fbe791c5bf3bbcfb5df8da8cfcf0c2e1da1913108e6a762112444740b88c163d7f4b",
  "74ed417f88cb0ca76c4a3d10f278bd010f1d3f95eafb254d4732511bb50e404507a4049b779c5230137e4091a5582271",
  "9043fcab93b972d3c14ad2dc8fa78ca7ad374fc937c02435681772a003f7a72876bc4d578089b5c4cf3fe9b480f1aabb",
  "52c3595b151d93d8b159c257301bfd5aa6f49210de0c55a6cd6df5ebeee44e4206cab950500f5d188f7fa14e6d900b75",
  "91cb67311e910cce68cd5b7d0de77aa40610d87c6681439b44c46c3ff786ae643956ab2c812478a1da8745b259f07a45",
  "859065ac81b81d3735130ba08b8af72a7256b603fefb74faabae25ed28cca6edcaa7c10ea32b5948d675c18a9b0f2b1d",
  "acd82a7d3943e23e95a9dc3ce0b0107ea358d6287f9e3afa245622f7c7e3e0a66142a928b6efcc02f594a95366d3a99d"
];

/**
 * Valid PCR0 values for development environments
 */
const DEFAULT_PCR0_VALUES_DEV = [
  "62c0407056217a4c10764ed9045694c29fa93255d3cc04c2f989cdd9a1f8050c8b169714c71f1118ebce2fcc9951d1a9",
  "cb95519905443f9f66f05f63c548b61ad1561a27fd5717b69285861aaea3c3063fe12a2571773b67fea3c6c11b4d8ec6",
  "deb5895831b5e4286f5a2dcf5e9c27383821446f8df2b465f141d10743599be20ba3bb381ce063bf7139cc89f7f61d4c",
  "70ba26c6af1ec3b57ce80e1adcc0ee96d70224d4c7a078f427895cdf68e1c30f09b5ac4c456588d872f3f21ff77c036b",
  "669404ea71435b8f498b48db7816a5c2ab1d258b1a77685b11d84d15a73189504d79c4dee13a658de9f4a0cbfc39cfe8",
  "a791bf92c25ffdfd372660e460a0e238c6778c090672df6509ae4bc065cf8668b6baac6b6a11d554af53ee0ff0172ad5",
  "c4285443b87b9b12a6cea3bef1064ec060f652b235a297095975af8f134e5ed65f92d70d4616fdec80af9dff48bb9f35"
];

/**
 * Public key used to verify PCR history signatures in SPKI DER format (base64-encoded)
 */
const PCR_VERIFICATION_PUBLIC_KEY_B64 =
  "MHYwEAYHKoZIzj0CAQYFK4EEACIDYgAEHiUY9kFWK1GqBGzczohhwEwElXzgWLDZa9R6wBx3JOBocgSt9+UIzZlJbPDjYeGBfDUXh7Z62BG2vVsh2NgclLB5S7A2ucBBtb1wd8vSQHP8jpdPhZX1slauPgbnROIP";

/**
 * Remote PCR history URLs
 */
const PCR_HISTORY_URLS = {
  prod: "https://raw.githubusercontent.com/OpenSecretCloud/opensecret/master/pcrProdHistory.json",
  dev: "https://raw.githubusercontent.com/OpenSecretCloud/opensecret/master/pcrDevHistory.json"
};

/**
 * PCR history entry type
 */
export type PcrHistoryEntry = {
  PCR0: string;
  PCR1: string;
  PCR2: string;
  timestamp: number;
  signature: string;
};

/**
 * Result of PCR0 validation
 */
export type Pcr0ValidationResult = {
  /** Whether the PCR0 hash matches a known good value */
  isMatch: boolean;
  /** Human-readable description of the validation result */
  text: string;
  /** Timestamp of when the PCR was verified (for remote attestation) */
  verifiedAt?: string;
};

/**
 * Configuration options for PCR validation
 */
export type PcrConfig = {
  /** Additional custom PCR0 values for production environments */
  pcr0Values?: string[];
  /** Additional custom PCR0 values for development environments */
  pcr0DevValues?: string[];
  /** Enable/disable remote attestation (defaults to true) */
  remoteAttestation?: boolean;
  /** Custom URLs for remote attestation */
  remoteAttestationUrls?: {
    /** URL for production PCR history */
    prod?: string;
    /** URL for development PCR history */
    dev?: string;
  };
};

/**
 * Imports the verification public key into the Web Crypto API
 */
async function importVerificationKey(): Promise<CryptoKey> {
  try {
    // Decode the base64 key to binary
    const binaryKey = new Uint8Array(
      atob(PCR_VERIFICATION_PUBLIC_KEY_B64)
        .split("")
        .map((c) => c.charCodeAt(0))
    );

    // Import as SPKI format
    return await crypto.subtle.importKey(
      "spki", // The format: SubjectPublicKeyInfo
      binaryKey, // Pass the Uint8Array directly, not .buffer
      {
        name: "ECDSA", // The algorithm
        namedCurve: "P-384" // The curve (must be P-384 to match our backend)
      },
      false, // Not extractable
      ["verify"] // Only for verification
    );
  } catch (error) {
    console.error("Error importing verification key:", error);
    throw new Error("Failed to import PCR verification key");
  }
}

/**
 * Fetches PCR history from repository
 */
async function fetchPcrHistory(
  env: "prod" | "dev",
  urls?: { prod?: string; dev?: string }
): Promise<PcrHistoryEntry[]> {
  try {
    const baseUrl = urls?.[env] || PCR_HISTORY_URLS[env];
    const response = await fetch(baseUrl);

    if (!response.ok) {
      throw new Error(`Failed to fetch PCR history: ${response.status}`);
    }

    return await response.json();
  } catch (error) {
    console.error("Error fetching PCR history:", error);
    throw new Error("Failed to fetch PCR history");
  }
}

/**
 * Verifies a PCR0 signature
 */
async function verifyPcr0Signature(
  pcr0: string,
  signatureBase64: string,
  publicKey: CryptoKey
): Promise<boolean> {
  try {
    // Convert PCR0 string to binary
    const encoder = new TextEncoder();
    const pcr0Binary = encoder.encode(pcr0);

    // Convert signature from base64 to binary
    const signatureBinary = new Uint8Array(
      atob(signatureBase64)
        .split("")
        .map((c) => c.charCodeAt(0))
    );

    // Verify using Web Crypto API
    return await crypto.subtle.verify(
      {
        name: "ECDSA",
        hash: { name: "SHA-384" } // Must match the hash used for signing
      },
      publicKey,
      signatureBinary,
      pcr0Binary
    );
  } catch (error) {
    console.error("Signature verification error:", error);
    return false;
  }
}

/**
 * Validates a PCR0 against remote history
 */
async function validatePcrAgainstRemoteHistory(
  pcr0: string,
  env: "prod" | "dev",
  urls?: { prod?: string; dev?: string }
): Promise<Pcr0ValidationResult | null> {
  try {
    // Import the verification key
    const publicKey = await importVerificationKey();

    // Fetch the PCR history
    const history = await fetchPcrHistory(env, urls);

    // Find a matching entry in the history
    for (const entry of history) {
      // Only check if PCR0 matches - we don't care about PCR1 or PCR2
      if (entry.PCR0 === pcr0) {
        // Verify the signature (only of PCR0)
        const isValid = await verifyPcr0Signature(entry.PCR0, entry.signature, publicKey);
        if (isValid) {
          return {
            isMatch: true,
            text: "PCR0 matches remotely attested value",
            verifiedAt: new Date(entry.timestamp * 1000).toLocaleString()
          };
        }
      }
    }

    // No valid match found
    return null;
  } catch (error) {
    console.error("PCR remote validation error:", error);
    // We return null for remote validation errors so we can fall back to local validation
    return null;
  }
}

/**
 * Validates a PCR0 hash and returns information about the match
 * @param hash - The PCR0 hash to validate
 * @param config - Optional configuration with custom PCR0 values
 * @returns Object containing match status and descriptive text
 */
export async function validatePcr0Hash(
  hash: string,
  config?: PcrConfig
): Promise<Pcr0ValidationResult> {
  // First check against local values
  const validPcr0Values = [...(config?.pcr0Values || []), ...DEFAULT_PCR0_VALUES];
  const validPcr0DevValues = [...(config?.pcr0DevValues || []), ...DEFAULT_PCR0_VALUES_DEV];

  if (validPcr0Values.includes(hash)) {
    return {
      isMatch: true,
      text: "PCR0 matches a known good value"
    };
  }

  if (validPcr0DevValues.includes(hash)) {
    return {
      isMatch: true,
      text: "PCR0 matches development enclave"
    };
  }

  // If remote attestation is enabled (default is true), check against remote PCR history
  const remoteAttestationEnabled = config?.remoteAttestation !== false;

  if (remoteAttestationEnabled) {
    try {
      // Try prod first
      const prodResult = await validatePcrAgainstRemoteHistory(
        hash,
        "prod",
        config?.remoteAttestationUrls
      );

      if (prodResult) {
        return prodResult;
      }

      // Try dev if prod didn't match
      const devResult = await validatePcrAgainstRemoteHistory(
        hash,
        "dev",
        config?.remoteAttestationUrls
      );

      if (devResult) {
        return devResult;
      }
    } catch (error) {
      console.error("Error during remote PCR validation:", error);
      // We continue with default behavior if remote validation fails
    }
  }

  return {
    isMatch: false,
    text: "PCR0 does not match a known good value"
  };
}
