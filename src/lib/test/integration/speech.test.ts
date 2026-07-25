import { afterEach, beforeEach, expect, mock, test } from "bun:test";
import { decode, encode } from "@stablelib/base64";
import { ChaCha20Poly1305 } from "@stablelib/chacha20poly1305";
import * as cbor from "cbor2";
import nacl from "tweetnacl";
import { synthesizeSpeech } from "../../ai";
import { getApiUrl, setApiUrl } from "../../api";
import { decryptMessage, encryptMessage } from "../../encryption";

const apiUrl = "https://api.example.com";
const apiKey = "speech-api-key";
const sessionId = "speech-session-id";
const sessionKey = new Uint8Array(32).fill(23);
const originalFetch = globalThis.fetch;
const originalApiUrl = getApiUrl();

const wavBytes = new Uint8Array([
  0x52, 0x49, 0x46, 0x46, 0x24, 0x00, 0x00, 0x00, 0x57, 0x41, 0x56, 0x45, 0x66, 0x6d, 0x74, 0x20,
  0x10, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x80, 0xbb, 0x00, 0x00, 0x00, 0x77, 0x01, 0x00,
  0x02, 0x00, 0x10, 0x00, 0x64, 0x61, 0x74, 0x61, 0x00, 0x00, 0x00, 0x00
]);

beforeEach(() => {
  window.localStorage.clear();
  window.sessionStorage.clear();
  window.sessionStorage.setItem("sessionKey", encode(sessionKey));
  window.sessionStorage.setItem("sessionId", sessionId);
  setApiUrl(apiUrl);
});

afterEach(() => {
  globalThis.fetch = originalFetch;
  setApiUrl(originalApiUrl);
  window.localStorage.clear();
  window.sessionStorage.clear();
});

function encryptedResponse(carrier: unknown, headers?: HeadersInit): Response {
  return new Response(
    JSON.stringify({
      encrypted: encryptMessage(sessionKey, JSON.stringify(carrier))
    }),
    {
      status: 200,
      headers: {
        "Content-Type": "application/json",
        ...headers
      }
    }
  );
}

test("synthesizeSpeech sends the Voxtral request and returns decoded WAV bytes", async () => {
  globalThis.fetch = mock(async (input: string | URL | Request, init?: RequestInit) => {
    expect(input.toString()).toBe(`${apiUrl}/v1/audio/speech`);
    expect(init?.method).toBe("POST");

    const headers = new Headers(init?.headers);
    expect(headers.get("authorization")).toBe(`Bearer ${apiKey}`);
    expect(headers.get("x-session-id")).toBe(sessionId);
    expect(headers.get("accept")).toBe("audio/wav");
    expect(headers.get("content-type")).toBe("application/json");

    const encryptedRequest = JSON.parse(String(init?.body)) as { encrypted: string };
    expect(JSON.parse(decryptMessage(sessionKey, encryptedRequest.encrypted))).toEqual({
      input: "Read this aloud.",
      model: "voxtral-tts",
      voice: "neutral_female"
    });

    return encryptedResponse(
      {
        content_base64: encode(wavBytes),
        content_type: " audio/wav; codecs=1 "
      },
      {
        "Content-Encoding": "identity",
        "Content-Length": "999",
        "Transfer-Encoding": "chunked",
        "X-Preserved": "yes"
      }
    );
  }) as typeof fetch;

  const response = await synthesizeSpeech(
    { input: "Read this aloud." },
    { apiKey, apiUrl: `${apiUrl}/` }
  );

  expect(response.headers.get("content-type")).toBe("audio/wav; codecs=1");
  expect(response.headers.get("content-encoding")).toBeNull();
  expect(response.headers.get("content-length")).toBeNull();
  expect(response.headers.get("transfer-encoding")).toBeNull();
  expect(response.headers.get("x-preserved")).toBe("yes");
  expect(new Uint8Array(await response.arrayBuffer())).toEqual(wavBytes);
});

test("synthesizeSpeech normalizes trailing-slash URLs for a fresh attestation session", async () => {
  window.sessionStorage.clear();

  const localApiUrl = "http://127.0.0.1:31587";
  const serverKeyPair = nacl.box.keyPair();
  const attestationPayload = cbor.encode({ public_key: serverKeyPair.publicKey });
  const fakeAttestationDocument = encode(
    cbor.encode([new Uint8Array(), new Map(), attestationPayload, new Uint8Array()])
  );
  const controller = new AbortController();
  const requestedUrls: string[] = [];
  let attestationNonce: string | undefined;

  globalThis.fetch = mock(async (input: string | URL | Request, init?: RequestInit) => {
    const requestUrl = input.toString();
    requestedUrls.push(requestUrl);

    if (requestUrl.startsWith(`${localApiUrl}/attestation/`)) {
      expect(init?.signal).toBe(controller.signal);
      expect(requestUrl).toMatch(
        /^http:\/\/127\.0\.0\.1:31587\/attestation\/[0-9a-f]{8}-[0-9a-f-]+$/i
      );
      attestationNonce = requestUrl.slice(`${localApiUrl}/attestation/`.length);
      return Response.json({ attestation_document: fakeAttestationDocument });
    }

    if (requestUrl === `${localApiUrl}/key_exchange`) {
      expect(init?.signal).toBe(controller.signal);
      const body = JSON.parse(String(init?.body)) as {
        client_public_key: string;
        nonce: string;
      };
      expect(body.nonce).toBe(attestationNonce);

      const clientPublicKey = decode(body.client_public_key);
      const sharedSecret = nacl.scalarMult(serverKeyPair.secretKey, clientPublicKey);
      const nonce = new Uint8Array(12).fill(31);
      const ciphertext = new ChaCha20Poly1305(sharedSecret).seal(nonce, sessionKey);
      const encryptedSessionKey = new Uint8Array(nonce.length + ciphertext.length);
      encryptedSessionKey.set(nonce);
      encryptedSessionKey.set(ciphertext, nonce.length);

      return Response.json({
        encrypted_session_key: encode(encryptedSessionKey),
        session_id: sessionId
      });
    }

    if (requestUrl === `${localApiUrl}/v1/audio/speech`) {
      expect(init?.signal).toBe(controller.signal);
      const encryptedRequest = JSON.parse(String(init?.body)) as { encrypted: string };
      expect(JSON.parse(decryptMessage(sessionKey, encryptedRequest.encrypted))).toEqual({
        input: "Cold session.",
        model: "voxtral-tts",
        voice: "neutral_female"
      });
      return encryptedResponse({
        content_base64: encode(wavBytes),
        content_type: "audio/wav"
      });
    }

    throw new Error(`Unexpected request URL: ${requestUrl}`);
  }) as typeof fetch;

  const response = await synthesizeSpeech(
    { input: "Cold session." },
    { apiKey, apiUrl: `${localApiUrl}/`, signal: controller.signal }
  );

  expect(requestedUrls).toHaveLength(3);
  expect(requestedUrls[0]).toStartWith(`${localApiUrl}/attestation/`);
  expect(requestedUrls[1]).toBe(`${localApiUrl}/key_exchange`);
  expect(requestedUrls[2]).toBe(`${localApiUrl}/v1/audio/speech`);
  expect(requestedUrls.every((url) => !url.includes(`${localApiUrl}//`))).toBe(true);
  expect(new Uint8Array(await response.arrayBuffer())).toEqual(wavBytes);
});

test("synthesizeSpeech preserves AbortError when cancelled during attestation", async () => {
  window.sessionStorage.clear();

  const localApiUrl = "http://127.0.0.1:31587";
  const controller = new AbortController();
  const abortError = new DOMException("Speech synthesis cancelled", "AbortError");
  const requestedUrls: string[] = [];
  let markAttestationStarted!: () => void;
  const attestationStarted = new Promise<void>((resolve) => {
    markAttestationStarted = resolve;
  });

  globalThis.fetch = mock(
    async (input: string | URL | Request, init?: RequestInit): Promise<Response> => {
      const requestUrl = input.toString();
      requestedUrls.push(requestUrl);

      if (!requestUrl.startsWith(`${localApiUrl}/attestation/`)) {
        throw new Error(`Unexpected request after attestation abort: ${requestUrl}`);
      }

      expect(init?.signal).toBe(controller.signal);
      markAttestationStarted();

      return new Promise<Response>((_resolve, reject) => {
        const signal = init?.signal;
        if (!signal) {
          reject(new Error("Attestation request did not receive the AbortSignal"));
          return;
        }

        const rejectWithAbortReason = () => reject(signal.reason);
        if (signal.aborted) {
          rejectWithAbortReason();
        } else {
          signal.addEventListener("abort", rejectWithAbortReason, { once: true });
        }
      });
    }
  ) as typeof fetch;

  const synthesis = synthesizeSpeech(
    { input: "Cancel before key exchange." },
    {
      apiKey,
      apiUrl: `${localApiUrl}/`,
      signal: controller.signal
    }
  );

  await attestationStarted;
  controller.abort(abortError);

  let rejection: unknown;
  try {
    await synthesis;
  } catch (error) {
    rejection = error;
  }

  expect(rejection).toBe(abortError);
  expect(requestedUrls).toHaveLength(1);
  expect(requestedUrls[0]).toStartWith(`${localApiUrl}/attestation/`);
  expect(requestedUrls.some((url) => url.endsWith("/key_exchange"))).toBe(false);
  expect(requestedUrls.some((url) => url.endsWith("/v1/audio/speech"))).toBe(false);
});

test("synthesizeSpeech rejects malformed base64 audio", async () => {
  globalThis.fetch = mock(async () =>
    encryptedResponse({
      content_base64: "%%%not-base64%%%",
      content_type: "audio/wav"
    })
  ) as typeof fetch;

  await expect(synthesizeSpeech({ input: "Hello" }, { apiKey })).rejects.toThrow(
    "Invalid base64 audio data in response"
  );
});

test("synthesizeSpeech rejects malformed and non-audio carriers", async () => {
  const invalidCarriers = [
    { content_base64: encode(wavBytes) },
    { content_base64: encode(wavBytes), content_type: "audio/" },
    { content_base64: encode(wavBytes), content_type: "application/octet-stream" }
  ];
  let responseIndex = 0;

  globalThis.fetch = mock(async () =>
    encryptedResponse(invalidCarriers[responseIndex++])
  ) as typeof fetch;

  for (const input of ["Missing type", "Empty subtype", "Wrong type"]) {
    await expect(synthesizeSpeech({ input }, { apiKey })).rejects.toThrow(
      "Invalid audio response carrier"
    );
  }
});

test("synthesizeSpeech forwards the caller's AbortSignal", async () => {
  const controller = new AbortController();

  globalThis.fetch = mock(async (_input: string | URL | Request, init?: RequestInit) => {
    expect(init?.signal).toBe(controller.signal);
    return encryptedResponse({
      content_base64: encode(wavBytes),
      content_type: "audio/wav"
    });
  }) as typeof fetch;

  await synthesizeSpeech({ input: "Cancelable speech." }, { apiKey, signal: controller.signal });
});
