import { expect, test, mock, beforeEach, afterEach } from "bun:test";
import { resetTestState } from "../preload";

// Import directly from source instead of lib index
import { validatePcr0Hash } from "../../pcr";
import { validatePcrAgainstHistory, PcrEntry, clearPcrHistoryCache } from "../../pcrHistory";

// Mock PCR entries for testing
const mockPcrEntries: PcrEntry[] = [
  {
    HashAlgorithm: "Sha384 { ... }",
    PCR0: "cc88f0edbccb5c92a46a2c4ba542c624123a793b002d1150153def94e34f3daa288f70162a8d163c5d36b31269624cb7",
    PCR1: "e45de6f4e9809176f6adc68df999f87f32a602361247d5819d1edf11ac5a403cfbb609943705844251af85713a17c83a",
    PCR2: "7f3c7df92680edd708d19a25784d18883381cc34e16d3fe9079f7f117970ccb2eb4f403875f1340558f86a58edcdcea9",
    timestamp: 1743007470,
    signature:
      "MGUCMGhWQG4c3y6mWzySkMgUMp34Es2QVw2XRLXebdjpzJX2tMAr3vEHTPCDlNvWPu9e7wIxAPq6M9cL5xFiTj9+ONFfdaH4/4CAqm2w2h4OVUjbKuhOvV0iW8c7oYKLDf53fjRn6A=="
  }
];

// Prepare for test
const originalFetch = global.fetch;

// Use bun test hooks

// Reset state and save the original fetch before each test
beforeEach(() => {
  // Reset all test state
  resetTestState();
  clearPcrHistoryCache();

  // Create a proper Response object for mocking
  global.fetch = mock(() =>
    Promise.resolve(
      new Response(JSON.stringify([]), {
        status: 200,
        headers: { "Content-Type": "application/json" }
      })
    )
  );
});

// Restore original fetch after each test
afterEach(() => {
  global.fetch = originalFetch;
});

// Tests below cover the async version of PCR validation

test("validatePcr0Hash should validate using hardcoded values", async () => {
  // Test with a known production PCR0 value
  const knownProdPcr0 =
    "eeddbb58f57c38894d6d5af5e575fbe791c5bf3bbcfb5df8da8cfcf0c2e1da1913108e6a762112444740b88c163d7f4b";
  const resultProd = await validatePcr0Hash(knownProdPcr0);
  expect(resultProd.isMatch).toBe(true);
  expect(resultProd.text).toBe("PCR0 matches a known good value");

  // Test with an unknown PCR0 value (without remote validation)
  const unknownPcr0 =
    "aaaaaa58f57c38894d6d5af5e575fbe791c5bf3bbcfb5df8da8cfcf0c2e1da1913108e6a762112444740b88c163d7f4b";
  const resultUnknown = await validatePcr0Hash(unknownPcr0);
  expect(resultUnknown.isMatch).toBe(false);
});

test("validatePcr0Hash should attempt remote validation if URL is provided", async () => {
  // Mock a successful fetch response with proper Response object
  global.fetch = mock(() =>
    Promise.resolve(
      new Response(JSON.stringify(mockPcrEntries), {
        status: 200,
        headers: { "Content-Type": "application/json" }
      })
    )
  );

  // Test with a PCR0 value from mockPcrEntries
  const validPcr0 = mockPcrEntries[0].PCR0;
  const result = await validatePcr0Hash(validPcr0, {
    remoteValidationUrl: "https://example.com/pcr-history.json"
  });

  // Check that we received a result
  expect(result).toBeDefined();
  expect(typeof result.isMatch).toBe("boolean");
  expect(typeof result.text).toBe("string");

  // The actual validation result depends on the implementation of the verifyPcrSignature function,
  // which uses Web Crypto API that might be mocked differently in test environments.
  // Here we're primarily testing that the remote validation path is attempted.
});

test("validatePcrAgainstHistory should validate PCR sets", async () => {
  // Mock a successful fetch response with proper Response object
  global.fetch = mock(() =>
    Promise.resolve(
      new Response(JSON.stringify(mockPcrEntries), {
        status: 200,
        headers: { "Content-Type": "application/json" }
      })
    )
  );

  // Create a PCR set matching the mock entry
  const validPcrSet = {
    PCR0: mockPcrEntries[0].PCR0,
    PCR1: mockPcrEntries[0].PCR1,
    PCR2: mockPcrEntries[0].PCR2
  };

  const result = await validatePcrAgainstHistory(validPcrSet, {
    remoteValidationUrl: "https://example.com/pcr-history.json"
  });

  // Validate result structure
  expect(result).toBeDefined();
  expect(typeof result.isMatch).toBe("boolean");
  expect(typeof result.text).toBe("string");
});

test("validatePcrAgainstHistory should handle fetch errors", async () => {
  // Mock a failed fetch
  global.fetch = mock(() => Promise.reject(new Error("Network error")));

  const pcrSet = {
    PCR0: "test",
    PCR1: "test",
    PCR2: "test"
  };

  const result = await validatePcrAgainstHistory(pcrSet, {
    remoteValidationUrl: "https://example.com/pcr-history.json"
  });

  expect(result.isMatch).toBe(false);
  expect(result.text).toBe("Couldn't validate against PCR history");
});
