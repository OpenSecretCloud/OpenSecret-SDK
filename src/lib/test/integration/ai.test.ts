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

  const stream = await openai.chat.completions.create({
    model,
    messages,
    stream: true
  });

  let fullResponse = "";

  for await (const chunk of stream) {
    const content = chunk.choices[0]?.delta?.content || "";
    fullResponse += content;
  }


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

test("OpenAI responses endpoint streams response", async () => {
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

  const stream = await openai.responses.create({
    model: "ibnzterrell/Meta-Llama-3.3-70B-Instruct-AWQ-INT4",
    input: 'please reply with exactly and only the word "echo"',
    stream: true
  });

  let fullResponse = "";
  let hasInProgressEvent = false;
  let hasDeltaEvents = false;
  let hasDoneEvent = false;
  let eventCount = 0;

  try {
    for await (const event of stream) {
      eventCount++;
      
      if (event.type === "response.created" || event.type === "response.in_progress") {
        hasInProgressEvent = true;
      } else if (event.type === "response.output_text.delta") {
        hasDeltaEvents = true;
        // Text delta events have delta field
        if (event.delta) {
          fullResponse += event.delta;
        }
      } else if (event.type === "response.completed" || event.type === "response.done") {
        hasDoneEvent = true;
      }
    }
  } catch (error) {
    console.error("Error processing stream:", error);
    throw error;
  }

  
  // First check if we got any events at all
  expect(eventCount).toBeGreaterThan(0);
  
  // Verify we got all expected event types
  expect(hasInProgressEvent).toBe(true);
  expect(hasDeltaEvents).toBe(true);
  expect(hasDoneEvent).toBe(true);
  
  // Verify the accumulated response
  expect(fullResponse.trim()).toBe("echo");
});
