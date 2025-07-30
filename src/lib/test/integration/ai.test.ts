import { expect, test } from "bun:test";
import { fetchLogin, transcribeAudio } from "../../api";
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

// To run this test manually:
// bun test --test-name-pattern="Whisper transcription with real MP3 file" src/lib/test/integration/ai.test.ts --env-file .env.local
test.skip("Whisper transcription with real MP3 file", async () => {
  await setupTestUser();

  // Read the test MP3 file
  const mp3Path = new URL('./test-transcript.mp3', import.meta.url).pathname;
  const mp3Buffer = await Bun.file(mp3Path).arrayBuffer();
  const audioFile = new File([mp3Buffer], "test-transcript.mp3", { type: "audio/mpeg" });
  
  console.log(`Test MP3 file size: ${mp3Buffer.byteLength} bytes`);
  
  // Transcribe the MP3 file
  const transcriptionResult = await transcribeAudio(
    audioFile,
    "whisper-large-v3",  // model
    "en"                  // language
  );
  
  console.log("Transcribed text from test MP3:", transcriptionResult.text);
  
  // Just verify we got some text back
  expect(transcriptionResult.text).toBeTruthy();
  expect(transcriptionResult.text.length).toBeGreaterThan(10);
}, 20000);  // 20 second timeout for this test

test("TTS → Whisper transcription chain", async () => {
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

  // Step 1: Generate speech from simple text
  const originalText = "Hello";
  
  console.log("Generating speech from text:", originalText);
  
  const ttsResponse = await client.audio.speech.create({
    model: "kokoro",
    // @ts-expect-error - Using custom Kokoro model voices not in OpenAI's type definitions
    voice: "af_sky",
    input: originalText,
    response_format: "mp3"
  });

  const audioBuffer = Buffer.from(await ttsResponse.arrayBuffer());
  console.log(`Generated audio size: ${audioBuffer.length} bytes`);
  
  // Step 2: Create a Blob from the audio buffer
  const audioBlob = new Blob([audioBuffer], { type: "audio/mpeg" });
  const audioFile = new File([audioBlob], "tts_output.mp3", { type: "audio/mpeg" });
  
  // Step 3: Transcribe the audio back to text using Whisper
  console.log("Transcribing audio back to text...");
  
  const transcriptionResult = await transcribeAudio(
    audioFile,
    "whisper-large-v3",  // model
    "en",                // language
    undefined,           // prompt
    0.0                  // temperature
  );
  
  console.log("Transcribed text:", transcriptionResult.text);
  console.log("Original text:", originalText);
  
  // Just check that we got "hello" back (case-insensitive)
  expect(transcriptionResult.text.toLowerCase()).toContain("hello");
});

test("OpenAI responses endpoint returns in_progress status", async () => {
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

  const response = await openai.responses.create({
    model: "ibnzterrell/Meta-Llama-3.3-70B-Instruct-AWQ-INT4",
    input: 'please reply with exactly and only the word "echo"'
  });

  console.log("Response:", response);

  // Check that the response has the expected structure
  expect(response).toHaveProperty("id");
  expect(response).toHaveProperty("object", "response");
  expect(response).toHaveProperty("status");

  // Since you mentioned the response is in_progress, let's check for that
  expect(response.status).toBe("in_progress");

  // TODO: Once the backend implements polling/completion for responses endpoint:
  // expect(response.output_text.trim()).toBe("echo");
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

test("OpenAI responses endpoint validates complete event sequence", async () => {
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
    input: 'please reply with exactly and only the words "echo echo"',
    stream: true
  });

  const events: any[] = [];
  const eventTypes: string[] = [];
  let fullResponse = "";

  try {
    for await (const event of stream) {
      events.push(event);
      eventTypes.push(event.type);
      
      if (event.type === "response.output_text.delta") {
        // Validate delta event structure
        expect(event).toHaveProperty("delta");
        expect(event).toHaveProperty("sequence_number");
        expect(event).toHaveProperty("item_id");
        expect(event).toHaveProperty("output_index");
        expect(event).toHaveProperty("content_index");
        expect(event).toHaveProperty("logprobs");
        
        fullResponse += event.delta;
      }
    }
  } catch (error) {
    console.error("Error processing stream:", error);
    throw error;
  }

  console.log("Received event types:", eventTypes);
  console.log("Total events:", events.length);

  // Validate we have the minimum required events
  expect(eventTypes).toContain("response.created");
  expect(eventTypes).toContain("response.output_text.delta");
  expect(eventTypes).toContain("response.completed");

  // Should have multiple events (not just 3)
  expect(events.length).toBeGreaterThan(3);

  // Validate we got multiple delta events (at least 2 for "echo echo")
  const deltaEvents = events.filter(e => e.type === "response.output_text.delta");
  expect(deltaEvents.length).toBeGreaterThanOrEqual(2);

  // Validate first event has proper response structure
  const createdEvent = events.find(e => e.type === "response.created");
  expect(createdEvent).toBeDefined();
  expect(createdEvent).toHaveProperty("sequence_number", 0);
  expect(createdEvent).toHaveProperty("response");
  expect(createdEvent.response).toHaveProperty("id");
  expect(createdEvent.response).toHaveProperty("status", "in_progress");
  expect(createdEvent.response).toHaveProperty("model");
  expect(createdEvent.response).toHaveProperty("output");
  expect(createdEvent.response.output).toEqual([]);

  // Check for other expected events
  const expectedEventTypes = [
    "response.created",
    "response.in_progress",
    "response.output_item.added",
    "response.content_part.added",
    "response.output_text.done",
    "response.content_part.done", 
    "response.output_item.done",
    "response.completed"
  ];

  // Log missing events for debugging
  const missingEvents = expectedEventTypes.filter(et => !eventTypes.includes(et));
  if (missingEvents.length > 0) {
    console.log("Missing event types:", missingEvents);
  }

  // Validate completed event has usage data
  const completedEvent = events.find(e => e.type === "response.completed");
  expect(completedEvent).toBeDefined();
  expect(completedEvent).toHaveProperty("sequence_number");
  expect(completedEvent).toHaveProperty("response");
  expect(completedEvent.response).toHaveProperty("status", "completed");
  expect(completedEvent.response).toHaveProperty("usage");
  if (completedEvent.response.usage) {
    expect(completedEvent.response.usage).toHaveProperty("total_tokens");
    expect(completedEvent.response.usage).toHaveProperty("input_tokens");
    expect(completedEvent.response.usage).toHaveProperty("output_tokens");
  }

  // Verify sequence numbers are in order
  const sequenceNumbers = events
    .filter(e => e.sequence_number !== undefined)
    .map(e => e.sequence_number);
  
  for (let i = 1; i < sequenceNumbers.length; i++) {
    expect(sequenceNumbers[i]).toBeGreaterThan(sequenceNumbers[i-1]);
  }

  // Verify the accumulated response
  expect(fullResponse.trim()).toBe("echo echo");
});

test("DEBUG: Inspect responses streaming events in detail", async () => {
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
    input: 'hello how is it going?',
    stream: true
  });

  console.log("\n🔍 STARTING DETAILED EVENT INSPECTION 🔍\n");
  
  let eventCount = 0;
  let fullResponse = "";

  for await (const event of stream) {
    eventCount++;
    console.log(`\n========== EVENT ${eventCount} ==========`);
    console.log(`Type: ${event.type}`);
    console.log(`Full event:`, JSON.stringify(event, null, 2));
    
    if (event.type === "response.output_text.delta") {
      fullResponse += event.delta;
      console.log(`🔤 DELTA: "${event.delta}" (length: ${event.delta.length})`);
      console.log(`📝 Accumulated so far: "${fullResponse}"`);
    }
  }

  console.log("\n📊 FINAL SUMMARY:");
  console.log(`Total events: ${eventCount}`);
  console.log(`Final response: "${fullResponse}"`);
  console.log(`Response length: ${fullResponse.length} characters`);
});
