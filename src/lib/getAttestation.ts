import { verifyAttestation } from "./attestation";
import { keyExchange } from "./api";
import nacl from "tweetnacl";
import { ChaCha20Poly1305 } from "@stablelib/chacha20poly1305";
import { encode, decode } from "@stablelib/base64";

export interface Attestation {
  sessionKey: Uint8Array | null;
  sessionId: string | null;
}

export async function getAttestation(forceRefresh?: boolean): Promise<Attestation> {
  // Check if we already have a sessionKey and sessionId in sessionstorage
  const sessionKey = sessionStorage.getItem("sessionKey");
  const sessionId = sessionStorage.getItem("sessionId");

  console.groupCollapsed("Attestation");

  try {
    // Attestation already set up
    if (sessionKey && sessionId && !forceRefresh) {
      const key = decode(sessionKey);
      console.log("Using existing attestation from session storage.");
      return { sessionKey: key, sessionId };
    }

    // Need to get a new attestation
    const attestationNonce = window.crypto.randomUUID();
    console.log("Generated attestation nonce:", attestationNonce);
    const document = await verifyAttestation(attestationNonce);

    if (document && document.public_key) {
      console.log("Attestation document verification succeeded");
      const clientKeyPair = nacl.box.keyPair();
      console.log("Generated client key pair");
      const serverPublicKey = new Uint8Array(document.public_key);

      const { encrypted_session_key, session_id } = await keyExchange(
        encode(clientKeyPair.publicKey),
        attestationNonce
      );
      console.log("Key exchange completed.");

      const sharedSecret = nacl.scalarMult(clientKeyPair.secretKey, serverPublicKey);

      const encryptedData = decode(encrypted_session_key);

      const nonceLength = 12;
      const decryptionNonce = encryptedData.slice(0, nonceLength);
      const ciphertext = encryptedData.slice(nonceLength);

      const chacha = new ChaCha20Poly1305(sharedSecret);
      const decryptedSessionKey = chacha.open(decryptionNonce, ciphertext);

      if (decryptedSessionKey) {
        console.log("Session key decrypted successfully");
        window.sessionStorage.setItem("sessionKey", encode(decryptedSessionKey));
        window.sessionStorage.setItem("sessionId", session_id);
        return { sessionKey: decryptedSessionKey, sessionId: session_id };
      } else {
        throw new Error("Failed to decrypt session key");
      }
    } else {
      throw new Error("Invalid attestation document");
    }
  } catch (error) {
    console.error("Error verifying attestation:", error);
    throw error;
  } finally {
    console.groupEnd();
  }
}
