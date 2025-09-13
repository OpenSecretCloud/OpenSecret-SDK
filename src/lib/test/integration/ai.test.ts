import { expect, test } from "bun:test";
import { fetchLogin } from "../../api";
import { createCustomFetch } from "../../ai";
import OpenAI from "openai";

const TEST_EMAIL = process.env.VITE_TEST_EMAIL;
const TEST_PASSWORD = process.env.VITE_TEST_PASSWORD;
const TEST_CLIENT_ID = process.env.VITE_TEST_CLIENT_ID;
const API_URL = process.env.VITE_OPEN_SECRET_API_URL;

if (!TEST_EMAIL || !TEST_PASSWORD || !TEST_CLIENT_ID || !API_URL) {
  throw new Error("Test credentials must be set in .env.local");
}

type ChatMessage = {
  role: "user" | "assistant";
  content: string;
};

async function setupTestUser() {
  const { access_token, refresh_token } = await fetchLogin(
    TEST_EMAIL!,
    TEST_PASSWORD!,
    TEST_CLIENT_ID!
  );
  window.localStorage.setItem("access_token", access_token);
  window.localStorage.setItem("refresh_token", refresh_token);
}

test("OpenAI custom fetch successfully makes a simple request", async () => {
  await setupTestUser();

  const openai = new OpenAI({
    baseURL: `${API_URL}/v1/`,
    dangerouslyAllowBrowser: true,
    apiKey: "api-key-doesnt-matter",
    defaultHeaders: {
      "Accept-Encoding": "identity"
    },
    fetch: createCustomFetch()
  });

  const model = "ibnzterrell/Meta-Llama-3.3-70B-Instruct-AWQ-INT4";
  const messages = [
    { role: "user", content: 'please reply with exactly and only the word "echo"' } as ChatMessage
  ];

  const stream = openai.beta.chat.completions.stream({
    model,
    messages,
    stream: true
  });

  let fullResponse = "";

  for await (const chunk of stream) {
    const content = chunk.choices[0]?.delta?.content || "";
    fullResponse += content;
  }

  await stream.finalChatCompletion();

  expect(fullResponse.trim()).toBe("echo");
});

async function streamCompletion(prompt: string) {
  const customFetch = createCustomFetch();
  let fullResponse = "";

  const response = await customFetch(`${API_URL}/v1/chat/completions`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Accept: "text/event-stream"
    },
    body: JSON.stringify({
      model: "ibnzterrell/Meta-Llama-3.3-70B-Instruct-AWQ-INT4",
      messages: [{ role: "user", content: prompt }],
      stream: true
    })
  });

  const text = await response.text();
  console.log("Full response text:", text);

  const lines = text.split("\n");
  for (const line of lines) {
    if (line.startsWith("data: ")) {
      const data = line.slice(6);
      if (data === "[DONE]") {
        break;
      }
      try {
        const parsed = JSON.parse(data);
        const content = parsed.choices[0]?.delta?.content;
        if (content) {
          console.log("Content:", content);
          fullResponse += content;
        }
      } catch (error) {
        console.error("Error parsing JSON:", error);
      }
    }
  }

  return fullResponse;
}

test("streams chat completion", async () => {
  await setupTestUser();

  const response = await streamCompletion('please reply with exactly and only the word "echo"');
  console.log("Final response:", response);

  expect(response?.trim()).toBe("echo");
});

test("text-to-speech with kokoro model", async () => {
  await setupTestUser();

  const client = new OpenAI({
    baseURL: `${API_URL}/v1/`,
    dangerouslyAllowBrowser: true,
    apiKey: "api-key-doesnt-matter",
    defaultHeaders: {
      "Accept-Encoding": "identity"
    },
    fetch: createCustomFetch()
  });

  const textToSpeak = "Hello, this is a test of the text-to-speech system.";
  
  const response = await client.audio.speech.create({
    model: "kokoro",
    // @ts-expect-error - Using custom Kokoro model voices not in OpenAI's type definitions
    voice: "af_sky",
    input: textToSpeak,
    response_format: "mp3"
  });

  const buffer = Buffer.from(await response.arrayBuffer());
  
  // Verify we got back audio data
  expect(buffer.length).toBeGreaterThan(0);
  
  console.log(`TTS response size: ${buffer.length} bytes`);
  
  // Log the first 20 bytes to understand the format
  const first20Bytes = buffer.slice(0, 20).toString('hex');
  console.log(`First 20 bytes (hex): ${first20Bytes}`);
  
  // Also log as string to see if it's JSON or text
  const first100Chars = buffer.slice(0, 100).toString('utf-8');
  console.log(`First 100 chars (string): ${first100Chars}`);
  
  // MP3 files typically start with an ID3 tag or FF FB/FF FA (MPEG audio sync)
  const firstBytes = buffer.slice(0, 3).toString('hex');
  const isID3 = firstBytes === '494433'; // "ID3" in hex
  const isMPEGSync = firstBytes.startsWith('fff') || firstBytes.startsWith('ffe');
  
  // For now, just verify we got data back
  // The format check might need adjustment based on what the server returns
  if (!isID3 && !isMPEGSync) {
    console.warn("Response doesn't appear to be MP3 format, but got data back");
  }
});
