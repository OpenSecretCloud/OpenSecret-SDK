import { encode } from "@stablelib/base64";
import { type AttestationDocument } from "./attestation";
import awsRootCertDer from "../assets/aws_root.der";
import { X509Certificate } from "@peculiar/x509";
import { validatePcr0Hash, type Pcr0ValidationResult, type PcrConfig } from "./pcr";

export const AWS_ROOT_CERT_DER = awsRootCertDer;

export const EXPECTED_ROOT_CERT_HASH =
  "641a0321a3e244efe456463195d606317ed7cdcc3c1756e09893f3c68f79bb5b";

export type ParsedAttestationView = {
  moduleId: string;
  publicKey: string | null;
  timestamp: string;
  digest: string;
  pcrs: Array<{
    id: number;
    value: string;
  }>;
  certificates: Array<{
    subject: string;
    notBefore: string;
    notAfter: string;
    pem: string;
    isRoot: boolean;
  }>;
  userData: string | null;
  nonce: string | null;
  cert0hash: string;
  validatedPcr0Hash: Pcr0ValidationResult | null;
};

function toHexString(data: Uint8Array | number[]): string {
  return Array.from(data)
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

async function calculateCertHash(data: ArrayBuffer): Promise<string> {
  const hashBuffer = await crypto.subtle.digest("SHA-256", data);
  return toHexString(new Uint8Array(hashBuffer));
}

/**
 * Validates PCR0 using standard validation methods
 * Note: When only validating against hardcoded values, only PCR0 is checked.
 * For remote validation, PCR1 and PCR2 are retained for signature verification.
 *
 * @param pcr0 - PCR0 value
 * @param pcr1 - PCR1 value (used for remote signature validation)
 * @param pcr2 - PCR2 value (used for remote signature validation)
 * @param pcrConfig - Optional PCR configuration
 * @returns Promise resolving to PCR validation result
 */
export async function validatePcrSet(
  pcr0: string,
  _pcr1: string, // Unused in local validation but needed for signature verification
  _pcr2: string, // Unused in local validation but needed for signature verification
  pcrConfig?: PcrConfig
): Promise<Pcr0ValidationResult> {
  // For local validation, only PCR0 is checked against known values
  // If remote validation URL is provided, pcr1 and pcr2 are used for signature verification
  return await validatePcr0Hash(pcr0, pcrConfig);
}

export async function parseAttestationForView(
  document: AttestationDocument,
  cabundle: Uint8Array[],
  pcrConfig?: PcrConfig
): Promise<ParsedAttestationView> {
  // Add logging to see what we're getting
  console.log("Raw timestamp:", document.timestamp);
  console.log("Date object:", new Date(document.timestamp));

  // Convert PCRs to hex strings and filter out all-zero values
  const pcrs = Array.from(document.pcrs.entries())
    .map(([key, value]) => ({
      id: key,
      value: toHexString(value)
    }))
    .filter((pcr) => !pcr.value.match(/^0+$/));

  // Find PCR0 for validation
  const pcr0 = pcrs.find((pcr) => pcr.id === 0);
  // PCR1 and PCR2 are available in the PCR array if needed for full validation

  // Use async validation which handles both local and remote checking
  const validatedPcr0Hash = pcr0 ? await validatePcr0Hash(pcr0.value, pcrConfig) : null;

  // Parse certificates - cabundle first, then leaf certificate
  const certificates = [...cabundle, document.certificate].map((certBytes) => {
    const cert = new X509Certificate(certBytes);
    return {
      subject: cert.subject,
      notBefore: cert.notBefore.toLocaleString(),
      notAfter: cert.notAfter.toLocaleString(),
      pem: cert.toString("pem"),
      isRoot: cert.subject === "C=US, O=Amazon, OU=AWS, CN=aws.nitro-enclaves"
    };
  });

  // Parse userData and nonce if they exist
  const decoder = new TextDecoder();

  // Calculate cert0 hash
  const cert0 = new X509Certificate(cabundle[0]);
  const cert0hash = await calculateCertHash(cert0.rawData);

  return {
    moduleId: document.module_id,
    publicKey: document.public_key ? encode(document.public_key) : null,
    timestamp: new Date(document.timestamp).toLocaleDateString("en-US", {
      year: "numeric",
      month: "long",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
      timeZoneName: "short"
    }),
    digest: document.digest,
    pcrs,
    certificates,
    userData: document.user_data ? decoder.decode(document.user_data) : null,
    nonce: document.nonce ? decoder.decode(document.nonce) : null,
    cert0hash,
    validatedPcr0Hash
  };
}
