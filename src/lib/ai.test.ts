import { expect, test } from "bun:test";
import { fetchLogin } from "./api";
import { createCustomFetch } from "./ai";
import OpenAI from "openai";

const TEST_EMAIL = process.env.VITE_TEST_EMAIL;
const TEST_PASSWORD = process.env.VITE_TEST_PASSWORD;
const API_URL = process.env.VITE_OPEN_SECRET_API_URL;

type ChatMessage = {
    role: "user" | "assistant";
    content: string;
};

async function setupTestUser() {
  const { access_token, refresh_token } = await fetchLogin(TEST_EMAIL!, TEST_PASSWORD!);
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

  const model = "hugging-quants/Meta-Llama-3.1-70B-Instruct-AWQ-INT4";
  const messages = [{ role: "user", content: "please reply with exactly and only the word \"echo\"" } as ChatMessage];

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
  let fullResponse = '';
  
  const response = await customFetch(`${API_URL}/v1/chat/completions`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Accept': 'text/event-stream',
    },
    body: JSON.stringify({
      model: 'hugging-quants/Meta-Llama-3.1-70B-Instruct-AWQ-INT4',
      messages: [{ role: 'user', content: prompt }],
      stream: true
    })
  });

  const text = await response.text();
  console.log('Full response text:', text);
  
  const lines = text.split('\n');
  for (const line of lines) {
    if (line.startsWith('data: ')) {
      const data = line.slice(6);
      if (data === '[DONE]') {
        break;
      }
      try {
        const parsed = JSON.parse(data);
        const content = parsed.choices[0]?.delta?.content;
        if (content) {
          console.log('Content:', content);
          fullResponse += content;
        }
      } catch (error) {
        console.error('Error parsing JSON:', error);
      }
    }
  }

  return fullResponse;
}

test("streams chat completion", async () => {
  await setupTestUser();
  
  const response = await streamCompletion("please reply with exactly and only the word \"echo\"");
  console.log("Final response:", response);
  
  expect(response?.trim()).toBe("echo");
}); 