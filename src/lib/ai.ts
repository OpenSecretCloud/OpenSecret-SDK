import { decryptMessage, encryptMessage } from "./encryption";
import { getAttestation } from "./getAttestation";
import * as api from "./api";

export interface CustomFetchOptions {
  apiKey?: string; // Optional API key to use instead of JWT token
  apiUrl?: string; // Optional API URL for attestation (required when not using OpenSecretProvider)
}

export const VOXTRAL_TTS_VOICES = [
  "neutral_female",
  "neutral_male",
  "casual_female",
  "casual_male",
  "cheerful_female",
  "ar_male",
  "de_female",
  "de_male",
  "es_female",
  "es_male",
  "fr_female",
  "fr_male",
  "hi_female",
  "hi_male",
  "it_female",
  "it_male",
  "nl_female",
  "nl_male",
  "pt_female",
  "pt_male"
] as const;

export type VoxtralTtsVoice = (typeof VOXTRAL_TTS_VOICES)[number];
export type SpeechSynthesisVoice = VoxtralTtsVoice;

export type SpeechSynthesisRequest = {
  input: string;
  model?: "voxtral-tts";
  voice?: SpeechSynthesisVoice;
};

export interface SpeechSynthesisOptions extends CustomFetchOptions {
  signal?: AbortSignal;
}

type AudioResponseCarrier = {
  content_base64: string;
  content_type: string;
};

const DEFAULT_SPEECH_MODEL = "voxtral-tts";
const DEFAULT_SPEECH_VOICE: SpeechSynthesisVoice = "neutral_female";

export async function synthesizeSpeech(
  request: SpeechSynthesisRequest,
  options?: SpeechSynthesisOptions
): Promise<Response> {
  const requestApiUrl = options?.apiUrl || api.getApiUrl();
  if (!requestApiUrl) {
    throw new Error(
      "No API URL configured. Pass apiUrl or call synthesizeSpeech within OpenSecretProvider."
    );
  }

  const apiUrl = requestApiUrl.replace(/\/+$/, "");
  const customFetch = createCustomFetch({
    apiKey: options?.apiKey,
    apiUrl
  });

  return customFetch(`${apiUrl}/v1/audio/speech`, {
    method: "POST",
    headers: {
      Accept: "audio/wav",
      "Content-Type": "application/json"
    },
    body: JSON.stringify({
      input: request.input,
      model: request.model ?? DEFAULT_SPEECH_MODEL,
      voice: request.voice ?? DEFAULT_SPEECH_VOICE
    }),
    signal: options?.signal
  });
}

export function createCustomFetch(
  options?: CustomFetchOptions
): (input: string | URL | Request, init?: RequestInit) => Promise<Response> {
  return async (requestUrl: string | URL | Request, init?: RequestInit): Promise<Response> => {
    const getAuthHeader = () => {
      // If an API key is provided, use it instead of JWT token
      if (options?.apiKey) {
        return `Bearer ${options.apiKey}`;
      }

      // Otherwise, use the standard JWT token
      const currentAccessToken = window.localStorage.getItem("access_token");
      if (!currentAccessToken) {
        throw new Error("No access token or API key available");
      }
      return `Bearer ${currentAccessToken}`;
    };

    try {
      const headers = new Headers(init?.headers);
      headers.set("Authorization", getAuthHeader());

      const { sessionKey, sessionId } = await getAttestation(
        false,
        options?.apiUrl,
        init?.signal ?? undefined
      );
      if (!sessionKey || !sessionId) {
        throw new Error("No session key or ID available");
      }
      headers.set("x-session-id", sessionId);

      const requestOptions: RequestInit = { ...init, headers };

      // Encrypt the request body if it exists
      if (init?.body) {
        const encryptedBody = encryptMessage(sessionKey, init.body as string);
        requestOptions.body = JSON.stringify({ encrypted: encryptedBody });
        headers.set("Content-Type", "application/json");
      }

      let response = await fetch(requestUrl, requestOptions);

      if (response.status === 401 && !options?.apiKey) {
        // Only attempt token refresh if we're using JWT auth (not API key)
        console.warn("Unauthorized, refreshing access token");
        await api.refreshToken();
        headers.set("Authorization", getAuthHeader());
        requestOptions.headers = headers;
        response = await fetch(requestUrl, requestOptions);
      }

      if (!response.ok) {
        const errorText = await response.text();
        console.error(
          "Request failed with response status:",
          response.status,
          " and message:",
          errorText
        );
        throw new Error(`Request failed with status ${response.status}: ${errorText}`);
      }

      // Decrypt SSE events
      if (response.headers.get("content-type")?.includes("text/event-stream")) {
        const reader = response.body?.getReader();
        const decoder = new TextDecoder();

        let buffer = "";
        const stream = new ReadableStream({
          async start(controller) {
            while (true) {
              const { done, value } = await reader!.read();
              if (done) break;

              const chunk = decoder.decode(value);
              buffer += chunk;

              let event;
              while ((event = extractEvent(buffer))) {
                buffer = buffer.slice(event.length);

                // Split the event into individual lines
                const lines = event.split("\n");

                for (const line of lines) {
                  // Handle event: lines - pass them through as-is
                  if (line.trim().startsWith("event: ")) {
                    controller.enqueue(line + "\n");
                  }
                  // Handle data: lines - decrypt them
                  else if (line.trim().startsWith("data: ")) {
                    const data = line.slice(6).trim();
                    if (data === "[DONE]") {
                      controller.enqueue(`data: [DONE]\n\n`);
                    } else {
                      try {
                        const decrypted = decryptMessage(sessionKey, data);

                        // Always enqueue the decrypted data
                        // Note: We don't add \n\n here because the empty line will be added separately
                        controller.enqueue(`data: ${decrypted}\n`);
                      } catch (error) {
                        console.error("Decryption error:", error, "Data:", data);
                        // Instead of sending the encrypted data, we'll skip this chunk
                        console.log("Skipping corrupted chunk");
                      }
                    }
                  }
                  // Pass through empty lines
                  else if (line === "") {
                    controller.enqueue("\n");
                  }
                }
              }
            }
            controller.close();
          }
        });

        return new Response(stream, {
          headers: response.headers,
          status: response.status,
          statusText: response.statusText
        });
      }

      // Decrypt regular JSON responses
      const responseText = await response.text();
      let responseData: unknown;
      try {
        responseData = JSON.parse(responseText);
      } catch {
        // If it's not JSON or doesn't have encrypted field, return original response
        console.log("Response is not encrypted JSON, returning as-is");
      }

      if (isRecord(responseData) && typeof responseData.encrypted === "string") {
        const decrypted = decryptMessage(sessionKey, responseData.encrypted);
        const audioCarrier = parseAudioResponseCarrier(decrypted);

        if (audioCarrier) {
          const bytes = decodeAudioResponseCarrier(audioCarrier);
          const headersOut = new Headers(response.headers);
          headersOut.set("content-type", audioCarrier.content_type);
          // These describe the encrypted carrier rather than the decoded audio.
          headersOut.delete("content-encoding");
          headersOut.delete("content-length");
          headersOut.delete("transfer-encoding");

          return new Response(bytes, {
            headers: headersOut,
            status: response.status,
            statusText: response.statusText
          });
        }

        return new Response(decrypted, {
          headers: response.headers,
          status: response.status,
          statusText: response.statusText
        });
      }

      // Return the original response text as a new Response
      return new Response(responseText, {
        headers: response.headers,
        status: response.status,
        statusText: response.statusText
      });
    } catch (error) {
      console.error("Error during fetch process:", error);
      throw error;
    }
  };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function parseAudioResponseCarrier(decrypted: string): AudioResponseCarrier | null {
  let value: unknown;
  try {
    value = JSON.parse(decrypted);
  } catch {
    return null;
  }

  if (!isRecord(value)) {
    return null;
  }

  const hasContent = Object.prototype.hasOwnProperty.call(value, "content_base64");
  const hasContentType = Object.prototype.hasOwnProperty.call(value, "content_type");
  if (!hasContent && !hasContentType) {
    return null;
  }

  if (typeof value.content_base64 !== "string" || value.content_base64.length === 0) {
    throw new Error("Invalid audio response carrier");
  }

  if (typeof value.content_type !== "string") {
    throw new Error("Invalid audio response carrier");
  }

  const contentType = value.content_type.trim();
  const mediaType = contentType.split(";", 1)[0].trim().toLowerCase();
  if (!/^audio\/[^\s/;]+$/.test(mediaType)) {
    throw new Error("Invalid audio response carrier");
  }

  return {
    content_base64: value.content_base64,
    content_type: contentType
  };
}

function decodeAudioResponseCarrier(carrier: AudioResponseCarrier): Uint8Array {
  let binaryString: string;
  try {
    binaryString = atob(carrier.content_base64);
  } catch (error) {
    console.error("Failed to decode base64 audio data:", error);
    throw new Error("Invalid base64 audio data in response");
  }

  if (binaryString.length === 0) {
    throw new Error("Audio response carrier contained no audio data");
  }

  const bytes = new Uint8Array(binaryString.length);
  for (let i = 0; i < binaryString.length; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }
  return bytes;
}

function extractEvent(buffer: string): string | null {
  const eventEnd = buffer.indexOf("\n\n");
  if (eventEnd === -1) return null;
  return buffer.slice(0, eventEnd + 2);
}
