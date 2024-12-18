import { expect, test } from "bun:test";
import {
  fetchLogin,
  fetchSignUp,
  fetchGuestLogin,
  fetchGuestSignUp,
  fetchLogout,
  refreshToken,
  fetchUser,
  convertGuestToEmailAccount,
  generateThirdPartyToken
} from "./api";

const TEST_EMAIL = process.env.VITE_TEST_EMAIL;
const TEST_PASSWORD = process.env.VITE_TEST_PASSWORD;
const TEST_NAME = process.env.VITE_TEST_NAME;
const TEST_INVITE_CODE = process.env.VITE_TEST_INVITE_CODE;

if (!TEST_EMAIL || !TEST_PASSWORD || !TEST_NAME || !TEST_INVITE_CODE) {
  throw new Error("Test credentials must be set in .env.local");
}

async function tryEmailLogin() {
  // Ensure the test user exists
  try {
    return await fetchLogin(TEST_EMAIL!, TEST_PASSWORD!);
  } catch (error) {
    console.warn(error);
    console.log("Login failed, attempting signup");
    await fetchSignUp(TEST_EMAIL!, TEST_PASSWORD!, TEST_INVITE_CODE!, TEST_NAME!);
    return await fetchLogin(TEST_EMAIL!, TEST_PASSWORD!);
  }
}

test("Login with email and fetch user data", async () => {
  const { access_token, refresh_token } = await tryEmailLogin();
  expect(access_token).toBeDefined();
  expect(refresh_token).toBeDefined();
  window.localStorage.setItem("access_token", access_token);
  window.localStorage.setItem("refresh_token", refresh_token);

  const userResponse = await fetchUser();
  expect(userResponse.user.email).toBe(TEST_EMAIL);
});

test("Refresh token works", async () => {
  await tryEmailLogin();

  const refreshResponse = await refreshToken();
  expect(refreshResponse.access_token).toBeDefined();
  expect(refreshResponse.refresh_token).toBeDefined();

  // Verify the new access token works
  const userResponse = await fetchUser();
  expect(userResponse.user.email).toBe(TEST_EMAIL);
});

test("Logout doesn't error", async () => {
  const { refresh_token } = await tryEmailLogin();
  await fetchLogout(refresh_token);
});

test("Guest signup and login flow", async () => {
  // Sign up as guest
  const guestSignup = await fetchGuestSignUp(TEST_PASSWORD!, TEST_INVITE_CODE!);
  expect(guestSignup.id).toBeDefined();
  expect(guestSignup.email).toBeNull();
  expect(guestSignup.access_token).toBeDefined();
  expect(guestSignup.refresh_token).toBeDefined();

  // Login as guest
  const guestLogin = await fetchGuestLogin(guestSignup.id, TEST_PASSWORD!);
  expect(guestLogin.id).toBe(guestSignup.id);
  expect(guestLogin.access_token).toBeDefined();

  // Set tokens for authenticated requests
  window.localStorage.setItem("access_token", guestLogin.access_token);
  window.localStorage.setItem("refresh_token", guestLogin.refresh_token);

  // Verify guest user data
  const userResponse = await fetchUser();
  expect(userResponse.user.id).toBe(guestSignup.id);
  expect(userResponse.user.email).toBeNull();

  // Generate random email and password for conversion
  const newEmail = `tony+test${Math.random().toString(36).substring(2)}@opensecret.cloud`;
  const newPassword = Math.random().toString(36).substring(2);

  // Convert guest to email account
  await convertGuestToEmailAccount(newEmail, newPassword);

  // Verify converted user data
  const convertedUserResponse = await fetchUser();
  expect(convertedUserResponse.user.email).toBe(newEmail);
  expect(convertedUserResponse.user.email_verified).toBe(false);

  // Try converting to an email address already in use
  try {
    await convertGuestToEmailAccount(TEST_EMAIL!, newPassword);
    throw new Error("Should not be able to convert to existing email");
  } catch (error: any) {
    expect(error.message).toBe("Bad Request");
  }

  // Try converting an already converted account
  try {
    await convertGuestToEmailAccount("another@example.com", "newpassword123");
    throw new Error("Should not be able to convert an already converted account");
  } catch (error: any) {
    expect(error.message).toBe("Bad Request");
  }

  // Try login with new email and password (should succeed)
  const emailLogin = await fetchLogin(newEmail, newPassword);
  expect(emailLogin.id).toBe(guestSignup.id);
  expect(emailLogin.email).toBe(newEmail);
  expect(emailLogin.access_token).toBeDefined();

  // Try guest login with old credentials (should fail)
  try {
    await fetchGuestLogin(guestSignup.id, TEST_PASSWORD!);
    throw new Error("Should not be able to login with old guest credentials");
  } catch (error: any) {
    expect(error.message).toBe("Invalid email, password, or login method");
  }
});

test("Guest refresh token works", async () => {
  // Sign up as guest
  const guestSignup = await fetchGuestSignUp(TEST_PASSWORD!, TEST_INVITE_CODE!);
  const guestLogin = await fetchGuestLogin(guestSignup.id, TEST_PASSWORD!);

  // Set tokens for authenticated requests
  window.localStorage.setItem("access_token", guestLogin.access_token);
  window.localStorage.setItem("refresh_token", guestLogin.refresh_token);

  const refreshResponse = await refreshToken();
  expect(refreshResponse.access_token).toBeDefined();
  expect(refreshResponse.refresh_token).toBeDefined();

  // Verify the new access token works
  const userResponse = await fetchUser();
  expect(userResponse.user.id).toBe(guestSignup.id);
  expect(userResponse.user.email).toBeNull();
});

test("Guest logout doesn't error", async () => {
  // Sign up as guest
  const guestSignup = await fetchGuestSignUp(TEST_PASSWORD!, TEST_INVITE_CODE!);
  const { refresh_token } = await fetchGuestLogin(guestSignup.id, TEST_PASSWORD!);
  await fetchLogout(refresh_token);
});

test("Third party token generation", async () => {
  // Login first to get authenticated
  const { access_token, refresh_token } = await tryEmailLogin();
  window.localStorage.setItem("access_token", access_token);
  window.localStorage.setItem("refresh_token", refresh_token);

  // Test successful token generation with valid audience
  const validOpenSecretAudience = "https://billing-dev.opensecret.cloud";
  const opensSecretResponse = await generateThirdPartyToken(validOpenSecretAudience);
  expect(opensSecretResponse.token).toBeDefined();
  expect(typeof opensSecretResponse.token).toBe("string");
  expect(opensSecretResponse.token.length).toBeGreaterThan(0);

  // Test invalid audience URL
  try {
    await generateThirdPartyToken("not-a-url");
    throw new Error("Should not accept invalid URL");
  } catch (error: any) {
    expect(error.message).toBe("Bad Request");
  }

  // Test not authorized URL
  try {
    await generateThirdPartyToken("https://google.com");
    throw new Error("Should not accept any random URL");
  } catch (error: any) {
    expect(error.message).toBe("Bad Request");
  }
});
