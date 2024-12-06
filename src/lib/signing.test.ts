import { expect, test } from "bun:test";
import { fetchLogin, signMessage, fetchPublicKey } from "./api";
import { bytesToHex } from "./test/utils";
import { sha256 } from '@noble/hashes/sha256';
import { schnorr, secp256k1 } from '@noble/curves/secp256k1';

const TEST_EMAIL = process.env.VITE_TEST_EMAIL;
const TEST_PASSWORD = process.env.VITE_TEST_PASSWORD;

async function setupTestUser() {
  const { access_token, refresh_token } = await fetchLogin(TEST_EMAIL!, TEST_PASSWORD!);
  window.localStorage.setItem("access_token", access_token);
  window.localStorage.setItem("refresh_token", refresh_token);
}

test("Sign message with Schnorr returns valid signature", async () => {
  await setupTestUser();

  // First get the public key
  const { public_key } = await fetchPublicKey("schnorr");
  expect(public_key).toBeDefined();
  expect(typeof public_key).toBe("string");
  expect(public_key).toMatch(/^[0-9a-f]{64}$/i); // 32 bytes = 64 hex chars

  // Then sign a message
  const message = new TextEncoder().encode("Hello, World!");
  const response = await signMessage(message, "schnorr");
  
  // Verify we got all fields
  expect(response.signature).toBeDefined();
  expect(response.message_hash).toBeDefined();

  // Verify types and formats
  expect(typeof response.signature).toBe("string");
  expect(typeof response.message_hash).toBe("string");

  // Verify hex format and lengths
  expect(response.signature).toMatch(/^[0-9a-f]{128}$/i); // 64 bytes = 128 hex chars
  expect(response.message_hash).toMatch(/^[0-9a-f]{64}$/i); // 32 bytes = 64 hex chars

  // Verify the hash matches our local hash
  const localHash = sha256(message);
  expect(bytesToHex(localHash)).toBe(response.message_hash.toLowerCase());

  // Verify the signature
  const isValid = schnorr.verify(
    response.signature,
    response.message_hash,
    public_key
  );
  expect(isValid).toBe(true);
});

test("Sign message with ECDSA returns valid signature", async () => {
  await setupTestUser();

  // First get the public key
  const { public_key } = await fetchPublicKey("ecdsa");
  expect(public_key).toBeDefined();
  expect(typeof public_key).toBe("string");
  expect(public_key).toMatch(/^[0-9a-f]{66}$/i); // 33 bytes = 66 hex chars (compressed format)

  // Then sign a message
  const message = new TextEncoder().encode("Hello, World!");
  const response = await signMessage(message, "ecdsa");
  
  // Verify we got all fields
  expect(response.signature).toBeDefined();
  expect(response.message_hash).toBeDefined();

  // Verify types and formats
  expect(typeof response.signature).toBe("string");
  expect(typeof response.message_hash).toBe("string");

  // Verify hex format and lengths
  expect(response.signature).toMatch(/^[0-9a-f]{130,144}$/i); // Adjust range to include shorter valid signatures
  expect(response.message_hash).toMatch(/^[0-9a-f]{64}$/i); // 32 bytes = 64 hex chars

  // Verify the hash matches our local hash
  const localHash = sha256(message);
  expect(bytesToHex(localHash)).toBe(response.message_hash.toLowerCase());

  // Verify the signature using noble/curves
  const isValid = secp256k1.verify(
    response.signature,
    response.message_hash,
    public_key
  );
  expect(isValid).toBe(true);
});

test("Sign message fails without authentication", async () => {
  window.localStorage.removeItem('access_token');
  window.localStorage.removeItem('refresh_token');

  const message = new TextEncoder().encode("Hello, World!");
  await expect(signMessage(message, "schnorr")).rejects.toThrow('No access token available');
});

test("Different messages produce different signatures and hashes", async () => {
  await setupTestUser();

  // Get the public key once
  const { public_key } = await fetchPublicKey("schnorr");
  
  const message1 = new TextEncoder().encode("Hello, World!");
  const message2 = new TextEncoder().encode("Different message");
  
  const response1 = await signMessage(message1, "schnorr");
  const response2 = await signMessage(message2, "schnorr");
  
  // Different signatures and hashes
  expect(response1.signature).not.toBe(response2.signature);
  expect(response1.message_hash).not.toBe(response2.message_hash);

  // Verify both signatures
  const isValid1 = schnorr.verify(
    response1.signature,
    response1.message_hash,
    public_key
  );
  expect(isValid1).toBe(true);

  const isValid2 = schnorr.verify(
    response2.signature,
    response2.message_hash,
    public_key
  );
  expect(isValid2).toBe(true);
});

test("Public key remains constant", async () => {
  await setupTestUser();
  
  const response1 = await fetchPublicKey("schnorr");
  const response2 = await fetchPublicKey("schnorr");
  
  expect(response1.public_key).toBe(response2.public_key);
});

