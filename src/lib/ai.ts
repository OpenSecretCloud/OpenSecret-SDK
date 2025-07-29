import { decryptMessage, encryptMessage } from "./encryption";
import { getAttestation } from "./getAttestation";
import * as api from "./api";

export function createCustomFetch(): (input: string | URL | Request, init?: RequestInit) => Promise<Response> {
  return async (requestUrl: string | URL | Request, init?: RequestInit): Promise<Response> => {
    const getAuthHeader = () => {
      const currentAccessToken = window.localStorage.getItem("access_token");
      if (!currentAccessToken) {
        throw new Error("No access token available");
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

      const options: RequestInit = { ...init, headers };

      // Encrypt the request body if it exists
      if (init?.body) {
        const encryptedBody = encryptMessage(sessionKey, init.body as string);
        options.body = JSON.stringify({ encrypted: encryptedBody });
        headers.set("Content-Type", "application/json");
      }

      let response = await fetch(requestUrl, options);

      if (response.status === 401) {
        console.warn("Unauthorized, refreshing access token");
        await api.refreshToken();
        headers.set("Authorization", getAuthHeader());
        options.headers = headers;
        response = await fetch(requestUrl, options);
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
                if (event.trim().startsWith("data: ")) {
                  const data = event.slice(6).trim();
                  if (data === "[DONE]") {
                    controller.enqueue(`data: [DONE]\n\n`);
                  } else {
                    try {
                      console.groupCollapsed("Decrypting chunk");
                      console.log("Attempting to decrypt, data length:", data.length);
                      const decrypted = decryptMessage(sessionKey, data);
                      console.log("Decrypted data length:", decrypted.length);
                      console.log("Decrypted data:", decrypted);

                      try {
                        const parsedJson = JSON.parse(decrypted);
                        console.log("Parsed JSON:", parsedJson);
                        controller.enqueue(`data: ${JSON.stringify(parsedJson)}\n\n`);
                      } catch (jsonError) {
                        if (jsonError instanceof SyntaxError) {
                          console.log("Failed to parse JSON:", decrypted);
                          controller.enqueue(`data: ${decrypted}\n\n`);
                        }
                      }
                    } catch (error) {
                      console.error("Decryption error:", error, "Data:", data);
                      // Instead of sending the encrypted data, we'll skip this chunk
                      console.log("Skipping corrupted chunk");
                    } finally {
                      console.groupEnd();
                    }
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

      return response;
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
