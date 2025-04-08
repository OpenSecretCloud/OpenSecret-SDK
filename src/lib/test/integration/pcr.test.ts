import { expect, test, mock, afterAll } from "bun:test";
import { validatePcr0Hash, type PcrConfig } from "../../pcr";

// Mock localStorage and other browser APIs
const storageMock = () => {
  const storage: Record<string, string> = {};
  return {
    getItem: (key: string) => storage[key] ?? null,
    setItem: (key: string, value: string) => {
      storage[key] = value;
    },
    removeItem: (key: string) => {
      delete storage[key];
    },
    clear: () => {
      Object.keys(storage).forEach((key) => delete storage[key]);
    },
    key: (i: number) => Object.keys(storage)[i] || null,
    length: Object.keys(storage).length
  } as Storage;
};

// Set up global mocks
global.localStorage = storageMock();
global.sessionStorage = storageMock();

// Sample PCR0 values for testing
const VALID_PCR0_PROD =
  "eeddbb58f57c38894d6d5af5e575fbe791c5bf3bbcfb5df8da8cfcf0c2e1da1913108e6a762112444740b88c163d7f4b";
const VALID_PCR0_DEV =
  "62c0407056217a4c10764ed9045694c29fa93255d3cc04c2f989cdd9a1f8050c8b169714c71f1118ebce2fcc9951d1a9";
const CUSTOM_PCR0 = "custom_pcr0_value_for_testing";

// Verified PCR0 with matching signature
const VERIFIED_PCR0 =
  "cc88f0edbccb5c92a46a2c4ba542c624123a793b002d1150153def94e34f3daa288f70162a8d163c5d36b31269624cb7";
const VERIFIED_PCR1 =
  "e45de6f4e9809176f6adc68df999f87f32a602361247d5819d1edf11ac5a403cfbb609943705844251af85713a17c83a";
const VERIFIED_PCR2 =
  "7f3c7df92680edd708d19a25784d18883381cc34e16d3fe9079f7f117970ccb2eb4f403875f1340558f86a58edcdcea9";
const VERIFIED_SIGNATURE =
  "sWjjc3Plp+yYjVW45Cs7MlYjCvlx4JiYB/BKwdCLgFHqY3N+gWsGu4JNWj6PHG9FxsW1i3gGaAikh4KhYYS+ynx3wVts3HrYtsipuFkVwUVFi1BpC8foMhUFgDDPOvRa";

// Mock the fetch API for remote attestation with real values
const originalFetch = global.fetch;

// Create a more complete Response-like object for TypeScript
function createMockResponse(data: any, status = 200, ok = true): Response {
  return {
    ok,
    status,
    statusText: ok ? "OK" : "Not Found",
    headers: new Headers(),
    body: null,
    bodyUsed: false,
    redirected: false,
    type: "basic" as ResponseType,
    url: "",
    json: async () => data,
    text: async () => JSON.stringify(data),
    arrayBuffer: async () => new ArrayBuffer(0),
    blob: async () => new Blob(),
    formData: async () => new FormData(),
    clone: function() { return this; }
  } as Response;
}

// Use more precise typing for the mock function
global.fetch = mock(async (input: RequestInfo | URL, init?: RequestInit): Promise<Response> => {
  const url = typeof input === 'string' ? input : input instanceof URL ? input.toString() : input.url;
  
  if (url.includes("pcr")) {
    return createMockResponse([
      {
        PCR0: VERIFIED_PCR0,
        PCR1: VERIFIED_PCR1,
        PCR2: VERIFIED_PCR2,
        timestamp: 1743710235,
        signature: VERIFIED_SIGNATURE
      }
    ]);
  }
  return createMockResponse(null, 404, false);
});

// Basic tests for PCR validation
test("validates known production PCR0 values", async () => {
  const result = await validatePcr0Hash(VALID_PCR0_PROD);
  expect(result.isMatch).toBe(true);
  expect(result.text).toBe("PCR0 matches a known good value");
});

test("validates known development PCR0 values", async () => {
  const result = await validatePcr0Hash(VALID_PCR0_DEV);
  expect(result.isMatch).toBe(true);
  expect(result.text).toBe("PCR0 matches development enclave");
});

test("validates custom PCR0 values", async () => {
  const config: PcrConfig = {
    pcr0Values: [CUSTOM_PCR0]
  };
  const result = await validatePcr0Hash(CUSTOM_PCR0, config);
  expect(result.isMatch).toBe(true);
  expect(result.text).toBe("PCR0 matches a known good value");
});

test("rejects unknown PCR0 values when remote attestation is disabled", async () => {
  const config: PcrConfig = {
    remoteAttestation: false
  };
  const result = await validatePcr0Hash(VERIFIED_PCR0, config);
  expect(result.isMatch).toBe(false);
});

test("validates PCR0 from remote attestation", async () => {
  // Need to use the exact PCR0 that matches our signature
  const result = await validatePcr0Hash(VERIFIED_PCR0);
  expect(result.isMatch).toBe(true);
  expect(result.text).toBe("PCR0 matches remotely attested value");
  expect(result.verifiedAt).toBeDefined();
});

test("prioritizes local PCR0 values over remote ones", async () => {
  // Add VERIFIED_PCR0 to the local values and ensure it uses local validation
  const config: PcrConfig = {
    pcr0Values: [VERIFIED_PCR0]
  };

  const result = await validatePcr0Hash(VERIFIED_PCR0, config);

  // Should match the local value, not the remote one
  expect(result.isMatch).toBe(true);
  expect(result.text).toBe("PCR0 matches a known good value");
  // No verification timestamp since it used local validation
  expect(result.verifiedAt).toBeUndefined();
});

test("handles fetch errors gracefully", async () => {
  // Mock a failing fetch call
  const failingFetch = mock(async (input: RequestInfo | URL, init?: RequestInit): Promise<Response> => {
    throw new Error("Network error");
  });

  try {
    global.fetch = failingFetch;
    const result = await validatePcr0Hash("unknown-pcr-value");

    // Should fall back to rejecting the PCR0
    expect(result.isMatch).toBe(false);
    expect(result.text).toBe("PCR0 does not match a known good value");
  } finally {
    // Restore the working fetch mock
    global.fetch = originalFetch;
  }
});

test("supports custom remote attestation URLs", async () => {
  const customFetch = mock(async (input: RequestInfo | URL, init?: RequestInit): Promise<Response> => {
    const url = typeof input === 'string' ? input : input instanceof URL ? input.toString() : input.url;
    
    if (url.includes("custom.example.com")) {
      return createMockResponse([
        {
          PCR0: VERIFIED_PCR0,
          PCR1: VERIFIED_PCR1,
          PCR2: VERIFIED_PCR2,
          timestamp: 1743710235,
          signature: VERIFIED_SIGNATURE
        }
      ]);
    }
    return createMockResponse(null, 404, false);
  });

  try {
    global.fetch = customFetch;

    const config: PcrConfig = {
      remoteAttestationUrls: {
        prod: "https://custom.example.com/prod.json",
        dev: "https://custom.example.com/dev.json"
      }
    };

    const result = await validatePcr0Hash(VERIFIED_PCR0, config);
    expect(result.isMatch).toBe(true);
    expect(result.text).toBe("PCR0 matches remotely attested value");
  } finally {
    global.fetch = originalFetch;
  }
});

// Restore the original fetch
afterAll(() => {
  global.fetch = originalFetch;
});
