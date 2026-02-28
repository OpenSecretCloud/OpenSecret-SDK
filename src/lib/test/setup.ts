import { configure } from "../config";

const TEST_API_URL = process.env.VITE_OPEN_SECRET_API_URL;

if (!TEST_API_URL) {
  throw new Error("Test configuration must be set in .env.local (VITE_OPEN_SECRET_API_URL)");
}

// For tests, we'll use a dummy client ID since the SDK now handles this internally
const TEST_CLIENT_ID = "test-client-id-for-testing";

// Configure the SDK for tests
configure({
  apiUrl: TEST_API_URL,
  clientId: TEST_CLIENT_ID
});

export { TEST_API_URL, TEST_CLIENT_ID };
