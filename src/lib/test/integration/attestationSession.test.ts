import { afterEach, expect, mock, test } from "bun:test";
import { encode } from "@stablelib/base64";
import { getAttestation, getAttestationSessionStorageKey } from "../../getAttestation";
import { getTrustedReleaseSnapshotId } from "../../pcr";

const originalFetch = globalThis.fetch;

afterEach(() => {
  window.sessionStorage.clear();
  globalThis.fetch = originalFetch;
});

test("session cache keys include the full normalized API base path and environment", () => {
  const prodPath = getAttestationSessionStorageKey("https://custom.example/prod/", "prod");
  const devPath = getAttestationSessionStorageKey("https://custom.example/dev", "prod");
  const devEnvironment = getAttestationSessionStorageKey("https://custom.example/prod", "dev");

  expect(prodPath).not.toBe(devPath);
  expect(prodPath).not.toBe(devEnvironment);
  expect(prodPath).toBe(getAttestationSessionStorageKey("https://custom.example/prod", "prod"));
});

test("reads only a valid policy-scoped unexpired cached session", async () => {
  const apiUrl = "https://custom.example/prod";
  const cacheKey = getAttestationSessionStorageKey(apiUrl, "prod");
  const sessionKey = new Uint8Array(32).fill(7);
  window.sessionStorage.setItem(
    cacheKey,
    JSON.stringify({
      sessionKey: encode(sessionKey),
      sessionId: "session-id",
      apiBaseUrl: apiUrl,
      environment: "prod",
      snapshotId: getTrustedReleaseSnapshotId(),
      expiresAt: Date.now() + 60_000
    })
  );
  globalThis.fetch = mock(async () => {
    throw new Error("cached sessions must not fetch");
  }) as typeof fetch;

  const result = await getAttestation(false, apiUrl, "prod");
  expect(result.sessionKey).toEqual(sessionKey);
  expect(result.sessionId).toBe("session-id");
  expect(globalThis.fetch).not.toHaveBeenCalled();
});

test("never reads legacy unversioned session keys", async () => {
  window.sessionStorage.setItem("sessionKey", encode(new Uint8Array(32).fill(8)));
  window.sessionStorage.setItem("sessionId", "legacy-session");
  globalThis.fetch = mock(async () => {
    throw new Error("fresh attestation required");
  }) as typeof fetch;

  await expect(getAttestation(false, "http://localhost:31110")).rejects.toThrow(
    "fresh attestation required"
  );
  expect(globalThis.fetch).toHaveBeenCalled();
  expect(window.sessionStorage.getItem("sessionKey")).toBeNull();
  expect(window.sessionStorage.getItem("sessionId")).toBeNull();
});

test("rejects expired or cross-policy cached sessions", async () => {
  const apiUrl = "https://custom.example/prod";
  const cacheKey = getAttestationSessionStorageKey(apiUrl, "prod");
  window.sessionStorage.setItem(
    cacheKey,
    JSON.stringify({
      sessionKey: encode(new Uint8Array(32).fill(7)),
      sessionId: "expired",
      apiBaseUrl: apiUrl,
      environment: "prod",
      snapshotId: getTrustedReleaseSnapshotId(),
      expiresAt: Date.now() - 1
    })
  );
  globalThis.fetch = mock(async () => {
    throw new Error("fresh attestation required");
  }) as typeof fetch;

  await expect(getAttestation(false, apiUrl, "prod")).rejects.toThrow("fresh attestation required");
  expect(window.sessionStorage.getItem(cacheKey)).toBeNull();
});
