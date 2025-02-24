import { expect, test, beforeEach } from "bun:test";
import { platformLogin, platformRegister } from "./platformApi";
import "./test/platform-api-url-loader";

const TEST_DEVELOPER_EMAIL = process.env.VITE_TEST_DEVELOPER_EMAIL;
const TEST_DEVELOPER_PASSWORD = process.env.VITE_TEST_DEVELOPER_PASSWORD;
const TEST_DEVELOPER_NAME = process.env.VITE_TEST_DEVELOPER_NAME;

if (!TEST_DEVELOPER_EMAIL || !TEST_DEVELOPER_PASSWORD || !TEST_DEVELOPER_NAME) {
  throw new Error("Test developer credentials must be set in .env.local");
}

// Cache login response to avoid multiple logins
let cachedLoginResponse: { access_token: string; refresh_token: string } | null = null;

async function tryDeveloperLogin() {
  // If we have a successful login cached, reuse it
  if (cachedLoginResponse) {
    return cachedLoginResponse;
  }

  try {
    // First, try to login directly
    console.log(`Attempting login with email: ${TEST_DEVELOPER_EMAIL}`);
    const response = await platformLogin(TEST_DEVELOPER_EMAIL!, TEST_DEVELOPER_PASSWORD!);
    console.log("Login successful");
    cachedLoginResponse = response;
    return response;
  } catch (loginError) {
    console.warn("Login failed, attempting to register the user");

    try {
      // Try to register the user with the credentials from environment variables
      const response = await platformRegister(
        TEST_DEVELOPER_EMAIL!,
        TEST_DEVELOPER_PASSWORD!,
        TEST_DEVELOPER_NAME!
      );
      console.log("Successfully registered test user");
      cachedLoginResponse = response;
      return response;
    } catch (registerError: any) {
      if (
        registerError.message.includes("Email already registered") ||
        registerError.message.includes("User already exists")
      ) {
        console.log("User already registered, retrying login");
        // If user already exists, try login again (maybe there was a temporary issue)
        const response = await platformLogin(TEST_DEVELOPER_EMAIL!, TEST_DEVELOPER_PASSWORD!);
        cachedLoginResponse = response;
        return response;
      } else {
        console.error("Registration failed with unexpected error:", registerError.message);
        throw registerError;
      }
    }
  }
}

// Clean up before each test
beforeEach(async () => {
  window.localStorage.clear();
});

test("Developer login and token storage", async () => {
  try {
    const { access_token, refresh_token } = await tryDeveloperLogin();
    expect(access_token).toBeDefined();
    expect(refresh_token).toBeDefined();

    // Store tokens in localStorage
    window.localStorage.setItem("platform_access_token", access_token);
    window.localStorage.setItem("platform_refresh_token", refresh_token);

    // Verify tokens were stored
    expect(window.localStorage.getItem("platform_access_token")).toBe(access_token);
    expect(window.localStorage.getItem("platform_refresh_token")).toBe(refresh_token);
  } catch (error: any) {
    console.error("Test failed:", error.message);
    throw error;
  }
});

test("Developer login fails with invalid credentials", async () => {
  try {
    await platformLogin("invalid@email.com", "wrongpassword");
    throw new Error("Should not succeed with invalid credentials");
  } catch (error: any) {
    // Match a wider range of error messages about invalid credentials
    expect(error.message).toMatch(/Invalid email|Invalid password|Invalid.*login|login.*Invalid/i);
  }
});

test("Developer registration fails with existing email", async () => {
  try {
    // This should fail since we've already registered this email in tryDeveloperLogin
    await platformRegister(TEST_DEVELOPER_EMAIL!, TEST_DEVELOPER_PASSWORD!, TEST_DEVELOPER_NAME!);
    // If we reach here without an error being thrown, the test should fail
    expect(true).toBe(false); // This line should never be reached
  } catch (error: any) {
    // We expect an "Email already registered" error
    expect(error.message).toMatch(/Email already registered|User already exists/);
  }
});

test("Developer login persists tokens correctly", async () => {
  try {
    const { access_token, refresh_token } = await tryDeveloperLogin();

    window.localStorage.setItem("platform_access_token", access_token);
    window.localStorage.setItem("platform_refresh_token", refresh_token);

    // Verify tokens are stored
    expect(window.localStorage.getItem("platform_access_token")).toBe(access_token);
    expect(window.localStorage.getItem("platform_refresh_token")).toBe(refresh_token);

    // Clear storage
    window.localStorage.clear();
    expect(window.localStorage.getItem("platform_access_token")).toBeNull();
    expect(window.localStorage.getItem("platform_refresh_token")).toBeNull();
  } catch (error: any) {
    console.error("Test failed:", error.message);
    throw error;
  }
});

test("Developer registration validates input", async () => {
  // Test empty email
  try {
    await platformRegister("", TEST_DEVELOPER_PASSWORD!, TEST_DEVELOPER_NAME!);
    throw new Error("Should not accept empty email");
  } catch (error: any) {
    // Allow for different error message formats
    expect(error.message).toMatch(/Invalid email|Email.*invalid|Bad Request/i);
  }

  // Test invalid email format
  try {
    await platformRegister("notanemail", TEST_DEVELOPER_PASSWORD!, TEST_DEVELOPER_NAME!);
    throw new Error("Should not accept invalid email format");
  } catch (error: any) {
    // Allow for different error message formats
    expect(error.message).toMatch(/Invalid email|Email.*invalid|Bad Request/i);
  }

  // Test empty password
  try {
    await platformRegister(TEST_DEVELOPER_EMAIL!, "", TEST_DEVELOPER_NAME!);
    throw new Error("Should not accept empty password");
  } catch (error: any) {
    // Allow for different error message formats
    expect(error.message).toMatch(/Invalid password|Password.*invalid|Bad Request/i);
  }
});
