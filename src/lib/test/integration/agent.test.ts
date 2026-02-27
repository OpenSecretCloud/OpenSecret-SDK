import { expect, test } from "bun:test";
import {
  fetchLogin,
  fetchSignUp,
  getAgentConfig,
  updateAgentConfig,
  listMemoryBlocks,
  getMemoryBlock,
  updateMemoryBlock,
  insertArchivalMemory,
  deleteArchivalMemory,
  searchAgentMemory,
  listAgentConversations,
  listAgentConversationItems
} from "../../api";

const TEST_EMAIL = process.env.VITE_TEST_EMAIL;
const TEST_PASSWORD = process.env.VITE_TEST_PASSWORD;
const TEST_CLIENT_ID = process.env.VITE_TEST_CLIENT_ID;
const API_URL = process.env.VITE_OPEN_SECRET_API_URL;

if (!TEST_EMAIL || !TEST_PASSWORD || !TEST_CLIENT_ID || !API_URL) {
  throw new Error("Test credentials must be set in .env.local");
}

async function setupTestUser() {
  try {
    const { access_token, refresh_token } = await fetchLogin(
      TEST_EMAIL!,
      TEST_PASSWORD!,
      TEST_CLIENT_ID!
    );
    window.localStorage.setItem("access_token", access_token);
    window.localStorage.setItem("refresh_token", refresh_token);
  } catch (error) {
    console.log("Login failed, attempting signup");
    await fetchSignUp(TEST_EMAIL!, TEST_PASSWORD!, "", TEST_CLIENT_ID!, "Test User");
    const { access_token, refresh_token } = await fetchLogin(
      TEST_EMAIL!,
      TEST_PASSWORD!,
      TEST_CLIENT_ID!
    );
    window.localStorage.setItem("access_token", access_token);
    window.localStorage.setItem("refresh_token", refresh_token);
  }
}

test.skip("Get agent config", async () => {
  await setupTestUser();

  const config = await getAgentConfig();

  expect(config).toBeDefined();
  expect(typeof config.enabled).toBe("boolean");
  expect(config.model).toBeDefined();
  expect(config.model.length).toBeGreaterThan(0);
  expect(config.max_context_tokens).toBeGreaterThan(0);
  expect(config.compaction_threshold).toBeGreaterThan(0);
  expect(config.compaction_threshold).toBeLessThanOrEqual(1);

  console.log("Agent config:", JSON.stringify(config));
});

test.skip("Update agent config", async () => {
  await setupTestUser();

  const original = await getAgentConfig();

  const updated = await updateAgentConfig({
    enabled: true,
    max_context_tokens: 80000,
    system_prompt: "You are a test assistant from TypeScript SDK."
  });

  expect(updated.enabled).toBe(true);
  expect(updated.max_context_tokens).toBe(80000);
  expect(updated.system_prompt).toBe("You are a test assistant from TypeScript SDK.");

  // Restore original
  await updateAgentConfig({
    enabled: original.enabled,
    model: original.model,
    max_context_tokens: original.max_context_tokens,
    compaction_threshold: original.compaction_threshold,
    system_prompt: original.system_prompt ?? undefined
  });
});

test.skip("List memory blocks", async () => {
  await setupTestUser();

  // Trigger config init which creates default blocks
  await getAgentConfig();

  const blocks = await listMemoryBlocks();

  expect(blocks).toBeDefined();
  expect(Array.isArray(blocks)).toBe(true);
  expect(blocks.length).toBeGreaterThanOrEqual(2);

  const labels = blocks.map((b) => b.label);
  expect(labels).toContain("persona");
  expect(labels).toContain("human");

  for (const block of blocks) {
    expect(block.label.length).toBeGreaterThan(0);
    expect(block.char_limit).toBeGreaterThan(0);
    expect(typeof block.read_only).toBe("boolean");
    expect(typeof block.version).toBe("number");
  }

  console.log(`Found ${blocks.length} memory blocks`);
});

test.skip("Get memory block by label", async () => {
  await setupTestUser();
  await getAgentConfig();

  const block = await getMemoryBlock("persona");

  expect(block).toBeDefined();
  expect(block.label).toBe("persona");
  expect(block.value.length).toBeGreaterThan(0);
  expect(block.char_limit).toBeGreaterThan(0);

  console.log("Persona block value:", block.value);
});

test.skip("Update memory block", async () => {
  await setupTestUser();
  await getAgentConfig();

  const original = await getMemoryBlock("human");

  const updated = await updateMemoryBlock("human", {
    value: "Test user info from TypeScript SDK integration test."
  });

  expect(updated.label).toBe("human");
  expect(updated.value).toBe("Test user info from TypeScript SDK integration test.");

  // Restore original
  await updateMemoryBlock("human", { value: original.value });
});

test.skip("Insert and delete archival memory", async () => {
  await setupTestUser();

  const inserted = await insertArchivalMemory({
    text: "TypeScript SDK test: The speed of light is approximately 299,792 km/s.",
    metadata: { tags: ["test", "physics"] }
  });

  expect(inserted).toBeDefined();
  expect(inserted.id).toBeDefined();
  expect(inserted.source_type).toBe("archival");
  expect(inserted.token_count).toBeGreaterThan(0);
  expect(inserted.embedding_model).toBeDefined();

  console.log(`Inserted archival memory: id=${inserted.id}, model=${inserted.embedding_model}`);

  const deleted = await deleteArchivalMemory(inserted.id);

  expect(deleted.deleted).toBe(true);
  expect(deleted.id).toBe(inserted.id);
});

test.skip("Search agent memory", async () => {
  await setupTestUser();

  // Insert something searchable
  const inserted = await insertArchivalMemory({
    text: "TypeScript SDK search test: machine learning uses neural networks for pattern recognition."
  });

  const results = await searchAgentMemory({
    query: "neural networks machine learning",
    top_k: 5,
    source_types: ["archival"]
  });

  expect(results).toBeDefined();
  expect(results.results).toBeDefined();
  expect(Array.isArray(results.results)).toBe(true);

  console.log(`Search returned ${results.results.length} results`);

  // Clean up
  await deleteArchivalMemory(inserted.id);
});

test.skip("List agent conversations", async () => {
  await setupTestUser();

  const conversations = await listAgentConversations();

  expect(conversations).toBeDefined();
  expect(conversations.object).toBe("list");
  expect(Array.isArray(conversations.data)).toBe(true);

  console.log(`Agent has ${conversations.data.length} conversations`);

  if (conversations.data.length > 0) {
    const conv = conversations.data[0];
    expect(conv.id).toBeDefined();

    const items = await listAgentConversationItems(conv.id, { limit: 10 });

    expect(items).toBeDefined();
    expect(items.object).toBe("list");
    expect(Array.isArray(items.data)).toBe(true);

    console.log(`First conversation has ${items.data.length} items (page)`);
  }
});

test.skip("Agent chat via SSE (using createCustomFetch)", async () => {
  await setupTestUser();

  // Ensure agent is enabled
  await updateAgentConfig({ enabled: true });

  const { createCustomFetch } = await import("../../ai");

  const customFetch = createCustomFetch();

  const response = await customFetch(`${API_URL}/v1/agent/chat`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Accept: "text/event-stream"
    },
    body: JSON.stringify({
      input: "Hello, please respond with just the word 'pong'."
    })
  });

  expect(response.ok).toBe(true);

  // Read the full response text (createCustomFetch decrypts SSE events inline)
  const text = await response.text();
  expect(text.length).toBeGreaterThan(0);

  // The decrypted SSE stream should contain agent event types and JSON data
  const hasAgentEvents =
    text.includes("agent.message") || text.includes("agent.done") || text.includes("messages");
  expect(hasAgentEvents).toBe(true);

  console.log("Agent chat SSE completed, response length:", text.length);
}, 60000);
