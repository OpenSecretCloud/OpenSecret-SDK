import { expect, test, beforeEach, afterEach } from "bun:test";
import { resetTestState } from "../preload";
import { verifyPcrSignature, loadPcrPublicKey, clearPcrHistoryCache, PcrEntry } from "../../pcrHistory";

// The PCR entry from the provided example
const samplePcrEntry: PcrEntry = {
  PCR0: "cc88f0edbccb5c92a46a2c4ba542c624123a793b002d1150153def94e34f3daa288f70162a8d163c5d36b31269624cb7",
  PCR1: "e45de6f4e9809176f6adc68df999f87f32a602361247d5819d1edf11ac5a403cfbb609943705844251af85713a17c83a",
  PCR2: "7f3c7df92680edd708d19a25784d18883381cc34e16d3fe9079f7f117970ccb2eb4f403875f1340558f86a58edcdcea9",
  timestamp: 1743195585,
  signature: "S6wWKZ8IfWiO5E0Y1dpj+RHbcUhIhyUg44mX1aQRV5VsCjf5MbEeJn/CaKeR8M7NkCixvW0J7LRcYEy4OxfyFIzpc/1IPSL1D4D1+8Lu3IfG6Rr86pNv4IO3qkuDTf4L"
};

beforeEach(() => {
  resetTestState();
  clearPcrHistoryCache();
});

// The REAL test using actual WebCrypto API - this is what matters most
test("REAL verification - should verify the actual signature with WebCrypto", async () => {
  // Load the actual public key
  const publicKey = await loadPcrPublicKey();
  
  // Verify the signature with actual WebCrypto implementation
  const result = await verifyPcrSignature(samplePcrEntry, publicKey);
  
  // This will fail if the signature doesn't verify with the public key
  expect(result).toBe(true);
}); 