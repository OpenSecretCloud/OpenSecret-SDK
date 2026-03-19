import { afterEach, beforeEach, expect, mock, test } from "bun:test";
import { encode } from "@stablelib/base64";
import { decryptMessage, encryptMessage } from "../../encryption";
import {
  getPushSettings,
  setPlatformApiUrl,
  updatePushSettings,
  type PushSettings
} from "../../platformApi";

const sessionKey = new Uint8Array(32).fill(7);
const sessionId = "push-settings-session-id";
const accessToken = "push-settings-access-token";
const platformApiUrl = "https://platform.example.com";

const originalFetch = globalThis.fetch;

beforeEach(() => {
  window.localStorage.clear();
  window.sessionStorage.clear();
  window.localStorage.setItem("access_token", accessToken);
  window.sessionStorage.setItem("sessionKey", encode(sessionKey));
  window.sessionStorage.setItem("sessionId", sessionId);
  setPlatformApiUrl(platformApiUrl);
});

afterEach(() => {
  globalThis.fetch = originalFetch;
});

test("getPushSettings calls the project push settings endpoint", async () => {
  const responseSettings: PushSettings = {
    encrypted_preview_enabled: true,
    ios: {
      enabled: true,
      bundle_id: "ai.trymaple.ios",
      apns_environment: "prod",
      team_id: "TEAM123",
      key_id: "KEY123"
    },
    android: {
      enabled: true,
      firebase_project_id: "firebase-project",
      package_name: "ai.trymaple.android"
    }
  };

  globalThis.fetch = mock(async (input: string | URL | Request, init?: RequestInit) => {
    expect(input.toString()).toBe(
      `${platformApiUrl}/platform/orgs/org-123/projects/project-456/settings/push`
    );
    expect(init?.method).toBe("GET");
    expect(init?.headers).toMatchObject({
      Authorization: `Bearer ${accessToken}`,
      "x-session-id": sessionId
    });

    return new Response(
      JSON.stringify({
        encrypted: encryptMessage(sessionKey, JSON.stringify(responseSettings))
      }),
      {
        status: 200,
        headers: { "Content-Type": "application/json" }
      }
    );
  }) as typeof fetch;

  const settings = await getPushSettings("org-123", "project-456");

  expect(settings).toEqual(responseSettings);
});

test("updatePushSettings sends encrypted push settings to the project endpoint", async () => {
  const requestSettings: PushSettings = {
    encrypted_preview_enabled: true,
    ios: {
      enabled: true,
      bundle_id: "ai.trymaple.ios",
      apns_environment: "dev",
      team_id: "TEAM456",
      key_id: "KEY456"
    },
    android: {
      enabled: false,
      firebase_project_id: "firebase-project",
      package_name: "ai.trymaple.android"
    }
  };

  globalThis.fetch = mock(async (input: string | URL | Request, init?: RequestInit) => {
    expect(input.toString()).toBe(
      `${platformApiUrl}/platform/orgs/org-123/projects/project-456/settings/push`
    );
    expect(init?.method).toBe("PUT");
    expect(init?.headers).toMatchObject({
      Authorization: `Bearer ${accessToken}`,
      "x-session-id": sessionId
    });

    const requestBody = JSON.parse(String(init?.body)) as { encrypted: string };
    const decryptedRequest = JSON.parse(
      decryptMessage(sessionKey, requestBody.encrypted)
    ) as PushSettings;
    expect(decryptedRequest).toEqual(requestSettings);

    return new Response(
      JSON.stringify({
        encrypted: encryptMessage(sessionKey, JSON.stringify(requestSettings))
      }),
      {
        status: 200,
        headers: { "Content-Type": "application/json" }
      }
    );
  }) as typeof fetch;

  const settings = await updatePushSettings("org-123", "project-456", requestSettings);

  expect(settings).toEqual(requestSettings);
});
