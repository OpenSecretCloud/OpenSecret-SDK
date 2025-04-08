import { expect, test, mock, beforeEach, afterEach } from "bun:test";
import { resetTestState } from "../preload";
import { validatePcrAgainstHistory, clearPcrHistoryCache, PcrEntry } from "../../pcrHistory";

// The PCR entry from the provided example
const samplePcrEntry: PcrEntry = {
  PCR0: "cc88f0edbccb5c92a46a2c4ba542c624123a793b002d1150153def94e34f3daa288f70162a8d163c5d36b31269624cb7",
  PCR1: "e45de6f4e9809176f6adc68df999f87f32a602361247d5819d1edf11ac5a403cfbb609943705844251af85713a17c83a",
  PCR2: "7f3c7df92680edd708d19a25784d18883381cc34e16d3fe9079f7f117970ccb2eb4f403875f1340558f86a58edcdcea9",
  timestamp: 1743195585,
  signature: "S6wWKZ8IfWiO5E0Y1dpj+RHbcUhIhyUg44mX1aQRV5VsCjf5MbEeJn/CaKeR8M7NkCixvW0J7LRcYEy4OxfyFIzpc/1IPSL1D4D1+8Lu3IfG6Rr86pNv4IO3qkuDTf4L"
};

// Original fetch to restore later
const originalFetch = global.fetch;

beforeEach(() => {
  resetTestState();
  clearPcrHistoryCache();
  
  // Mock fetch to return our sample entry in the PCR history
  global.fetch = mock(() =>
    Promise.resolve(
      new Response(JSON.stringify([samplePcrEntry]), {
        status: 200,
        headers: { "Content-Type": "application/json" }
      })
    )
  );
});

afterEach(() => {
  // Restore original fetch
  global.fetch = originalFetch;
});

test("validatePcrAgainstHistory should validate matching PCR set", async () => {
  // PCR set matching our sample entry
  const pcrSet = {
    PCR0: samplePcrEntry.PCR0,
    PCR1: samplePcrEntry.PCR1,
    PCR2: samplePcrEntry.PCR2
  };
  
  // Run the validation
  const result = await validatePcrAgainstHistory(pcrSet, {
    remoteValidationUrl: "https://example.com/test-pcr-history.json"
  });
  
  // Should match and include the verification date
  expect(result.isMatch).toBe(true);
  expect(result.text).toContain("Verified PCR with signature from");
});

test("validatePcrAgainstHistory should reject non-matching PCR", async () => {
  // PCR set with different values
  const pcrSet = {
    PCR0: "different-pcr0-value",
    PCR1: "different-pcr1-value",
    PCR2: "different-pcr2-value"
  };
  
  // Run the validation
  const result = await validatePcrAgainstHistory(pcrSet, {
    remoteValidationUrl: "https://example.com/test-pcr-history.json"
  });
  
  // Should not match
  expect(result.isMatch).toBe(false);
  expect(result.text).toBe("PCR not found in verified history");
});

test("validatePcrAgainstHistory should handle fetch errors", async () => {
  // Mock fetch to fail
  global.fetch = mock(() => Promise.reject(new Error("Network error")));
  
  const pcrSet = {
    PCR0: samplePcrEntry.PCR0,
    PCR1: samplePcrEntry.PCR1,
    PCR2: samplePcrEntry.PCR2
  };
  
  // Run the validation
  const result = await validatePcrAgainstHistory(pcrSet, {
    remoteValidationUrl: "https://example.com/test-pcr-history.json"
  });
  
  // Should fail with appropriate message
  expect(result.isMatch).toBe(false);
  expect(result.text).toBe("Couldn't validate against PCR history");
}); 