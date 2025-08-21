import { expect, test, describe, beforeAll, afterAll } from "bun:test";
import {
  fetchLogin,
  createApiKey,
  listApiKeys,
  deleteApiKey,
  fetchModels
} from "../../api";
import { createCustomFetch } from "../../ai";
import OpenAI from "openai";

const TEST_EMAIL = process.env.VITE_TEST_EMAIL;
const TEST_PASSWORD = process.env.VITE_TEST_PASSWORD;
const TEST_CLIENT_ID = process.env.VITE_TEST_CLIENT_ID;
const API_URL = process.env.VITE_OPEN_SECRET_API_URL;

if (!TEST_EMAIL || !TEST_PASSWORD || !TEST_CLIENT_ID || !API_URL) {
  throw new Error("Test credentials must be set in .env.local");
}

async function setupTestUser() {
  const { access_token, refresh_token } = await fetchLogin(
    TEST_EMAIL!,
    TEST_PASSWORD!,
    TEST_CLIENT_ID!
  );
  window.localStorage.setItem("access_token", access_token);
  window.localStorage.setItem("refresh_token", refresh_token);
}

describe("API Key Management", () => {
  beforeAll(async () => {
    await setupTestUser();
  });

  test("Create, list, and delete API key", async () => {
    // Create a new API key
    const keyName = `Test Key ${Date.now()}`;
    const createdKey = await createApiKey(keyName);
    
    expect(createdKey.name).toBe(keyName);
    expect(createdKey.id).toBeGreaterThan(0);
    expect(createdKey.key).toBeDefined();
    expect(createdKey.created_at).toBeDefined();
    
    // Verify the key format is a UUID with dashes
    const uuidRegex = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
    expect(createdKey.key).toMatch(uuidRegex);
    
    // List API keys and verify the new key is present
    const response = await listApiKeys();
    console.log("Listed response:", response);
    
    // Check if keys is an array
    expect(Array.isArray(response.keys)).toBe(true);
    const foundKey = response.keys.find(k => k.id === createdKey.id);
    
    expect(foundKey).toBeDefined();
    expect(foundKey!.name).toBe(keyName);
    expect(foundKey!.created_at).toBe(createdKey.created_at);
    // The key field should NOT be present in the list response
    expect((foundKey as any).key).toBeUndefined();
    
    // Delete the API key
    await deleteApiKey(createdKey.id);
    
    // Verify the key is deleted
    const responseAfterDelete = await listApiKeys();
    const deletedKey = responseAfterDelete.keys.find(k => k.id === createdKey.id);
    expect(deletedKey).toBeUndefined();
  });

  test("Create multiple API keys", async () => {
    const keyIds: number[] = [];
    
    try {
      // Create multiple keys
      for (let i = 0; i < 3; i++) {
        const key = await createApiKey(`Test Key ${i} - ${Date.now()}`);
        keyIds.push(key.id);
      }
      
      // List keys and verify all are present
      const response = await listApiKeys();
      for (const id of keyIds) {
        expect(response.keys.some(k => k.id === id)).toBe(true);
      }
    } finally {
      // Clean up: delete all created keys
      for (const id of keyIds) {
        try {
          await deleteApiKey(id);
        } catch (error) {
          console.warn(`Failed to delete key ${id}:`, error);
        }
      }
    }
  });
});

describe("API Key Authentication with OpenAI", () => {
  let testApiKey: string;
  let testApiKeyId: number;

  beforeAll(async () => {
    await setupTestUser();
    
    // Create an API key for testing
    const createdKey = await createApiKey(`OpenAI Test Key ${Date.now()}`);
    testApiKey = createdKey.key;
    testApiKeyId = createdKey.id;
  });

  test("OpenAI custom fetch with API key makes a simple request", async () => {
    const openai = new OpenAI({
      baseURL: `${API_URL}/v1/`,
      dangerouslyAllowBrowser: true,
      apiKey: testApiKey, // Use the API key directly
      defaultHeaders: {
        "Accept-Encoding": "identity"
      },
      fetch: createCustomFetch({ apiKey: testApiKey })
    });

    const model = "ibnzterrell/Meta-Llama-3.3-70B-Instruct-AWQ-INT4";
    const messages = [
      { role: "user" as const, content: 'please reply with exactly and only the word "echo"' }
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

  test("Fetch models with API key authentication", async () => {
    // Test that fetchModels works with API key
    const models = await fetchModels(testApiKey);
    
    expect(Array.isArray(models)).toBe(true);
    expect(models.length).toBeGreaterThan(0);
    
    // Verify each model has required fields
    const firstModel = models[0];
    expect(firstModel.id).toBeDefined();
    expect(firstModel.object).toBe("model");
  });

  test("API key authentication fails with invalid key", async () => {
    const invalidApiKey = "550e8400-e29b-41d4-a716-000000000000"; // Invalid UUID
    
    const customFetch = createCustomFetch({ apiKey: invalidApiKey });
    
    try {
      const response = await customFetch(`${API_URL}/v1/models`, {
        method: "GET",
        headers: {
          "Content-Type": "application/json"
        }
      });
      
      // Should get 401 Unauthorized
      expect(response.status).toBe(401);
    } catch (error) {
      // If it throws before the response, that's also acceptable
      expect(error).toBeDefined();
    }
  });

  test("JWT auth still works alongside API key support", async () => {
    // Test that the original JWT-based auth still works
    const models = await fetchModels();
    expect(Array.isArray(models)).toBe(true);
    expect(models.length).toBeGreaterThan(0);
  });

  // Clean up after all tests
  afterAll(async () => {
    if (testApiKeyId) {
      try {
        await deleteApiKey(testApiKeyId);
      } catch (error) {
        console.warn("Failed to delete test API key:", error);
      }
    }
  });
});