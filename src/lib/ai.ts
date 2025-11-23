import { decryptMessage, encryptMessage } from "./encryption";
import { getAttestation } from "./getAttestation";
import * as api from "./api";

export interface CustomFetchOptions {
  apiKey?: string; // Optional API key to use instead of JWT token
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

      const { sessionKey, sessionId } = await getAttestation();
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
      try {
        const responseData = JSON.parse(responseText);

        // Check if the response has an encrypted field
        if (responseData.encrypted) {
          const decrypted = decryptMessage(sessionKey, responseData.encrypted);

          // Try to parse as JSON to check for TTS response format
          try {
            const decryptedData = JSON.parse(decrypted);

            // Check if this is a TTS response with content_base64 and content_type
            if (decryptedData.content_base64 && decryptedData.content_type) {
              console.log("TTS response detected with content_type:", decryptedData.content_type);

              // Decode base64 audio data to binary
              let bytes: Uint8Array;
              try {
                const binaryString = atob(decryptedData.content_base64);
                bytes = new Uint8Array(binaryString.length);
                for (let i = 0; i < binaryString.length; i++) {
                  bytes[i] = binaryString.charCodeAt(i);
                }
              } catch (e) {
                console.error("Failed to decode base64 audio data:", e);
                throw new Error("Invalid base64 audio data in TTS response");
              }

              console.log("Decoded audio bytes length:", bytes.length);

              // Return as a binary response with the proper content type
              const headersOut = new Headers(response.headers);
              headersOut.set("content-type", decryptedData.content_type);
              // Remove headers that are no longer valid for the decoded response
              headersOut.delete("content-encoding");
              headersOut.delete("content-length");
              headersOut.delete("transfer-encoding");

              return new Response(bytes, {
                headers: headersOut,
                status: response.status,
                statusText: response.statusText
              });
            }
          } catch (jsonError) {
            // Not JSON, continue with regular text response
          }
          // Return a new Response with the decrypted data
          return new Response(decrypted, {
            headers: response.headers,
            status: response.status,
            statusText: response.statusText
          });
        }
      } catch (e) {
        // If it's not JSON or doesn't have encrypted field, return original response
        console.log("Response is not encrypted JSON, returning as-is");
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

function extractEvent(buffer: string): string | null {
  const eventEnd = buffer.indexOf("\n\n");
  if (eventEnd === -1) return null;
  return buffer.slice(0, eventEnd + 2);
}
