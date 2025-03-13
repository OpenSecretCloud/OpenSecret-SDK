import { expect, test } from "bun:test";
import {
  fetchLogin,
  fetchSignUp,
  fetchGuestLogin,
  fetchGuestSignUp,
  fetchLogout,
  refreshToken,
  fetchUser,
  // convertGuestToEmailAccount,
  generateThirdPartyToken,
  encryptData,
  decryptData
} from "../../api";

const TEST_EMAIL = process.env.VITE_TEST_EMAIL;
const TEST_PASSWORD = process.env.VITE_TEST_PASSWORD;
const TEST_NAME = process.env.VITE_TEST_NAME;
const TEST_CLIENT_ID = process.env.VITE_TEST_CLIENT_ID;

if (!TEST_EMAIL || !TEST_PASSWORD || !TEST_NAME || !TEST_CLIENT_ID) {
  throw new Error("Test credentials must be set in .env.local");
}

async function tryEmailLogin() {
  // Ensure the test user exists
  try {
    return await fetchLogin(TEST_EMAIL!, TEST_PASSWORD!, TEST_CLIENT_ID!);
  } catch (error) {
    console.warn(error);
    console.log("Login failed, attempting signup");
    await fetchSignUp(TEST_EMAIL!, TEST_PASSWORD!, "", TEST_CLIENT_ID!, TEST_NAME!);
    return await fetchLogin(TEST_EMAIL!, TEST_PASSWORD!, TEST_CLIENT_ID!);
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
  const guestSignup = await fetchGuestSignUp(TEST_PASSWORD!, "", TEST_CLIENT_ID!);
  expect(guestSignup.id).toBeDefined();
  expect(guestSignup.email).toBeNull();
  expect(guestSignup.access_token).toBeDefined();
  expect(guestSignup.refresh_token).toBeDefined();

  // Login as guest
  const guestLogin = await fetchGuestLogin(guestSignup.id, TEST_PASSWORD!, TEST_CLIENT_ID!);
  expect(guestLogin.id).toBe(guestSignup.id);
  expect(guestLogin.access_token).toBeDefined();

  // Set tokens for authenticated requests
  window.localStorage.setItem("access_token", guestLogin.access_token);
  window.localStorage.setItem("refresh_token", guestLogin.refresh_token);

  // Verify guest user data
  const userResponse = await fetchUser();
  expect(userResponse.user.id).toBe(guestSignup.id);
  expect(userResponse.user.email).toBeNull();

  /* Commenting out guest conversion tests due to email sending
  // Generate random email and password for conversion
  const newEmail = `tony+test${Math.random().toString(36).substring(2)}@opensecret.cloud`;
  const newPassword = Math.random().toString(36).substring(2);

  // Convert guest to email account
  await convertGuestToEmailAccount(newEmail, newPassword, undefined, TEST_CLIENT_ID!);

  // Verify converted user data
  const convertedUserResponse = await fetchUser();
  expect(convertedUserResponse.user.email).toBe(newEmail);
  expect(convertedUserResponse.user.email_verified).toBe(false);

  // Try converting to an email address already in use
  try {
    await convertGuestToEmailAccount(TEST_EMAIL!, newPassword, undefined, TEST_CLIENT_ID!);
    throw new Error("Should not be able to convert to existing email");
  } catch (error: any) {
    expect(error.message).toBe("Bad Request");
  }

  // Try converting an already converted account
  try {
    await convertGuestToEmailAccount("another@example.com", "newpassword123", undefined, TEST_CLIENT_ID!);
    throw new Error("Should not be able to convert an already converted account");
  } catch (error: any) {
    expect(error.message).toBe("Bad Request");
  }

  // Try login with new email and password (should succeed)
  const emailLogin = await fetchLogin(newEmail, newPassword, TEST_CLIENT_ID!);
  expect(emailLogin.id).toBe(guestSignup.id);
  expect(emailLogin.email).toBe(newEmail);
  expect(emailLogin.access_token).toBeDefined();

  // Try guest login with old credentials (should fail)
  try {
    await fetchGuestLogin(guestSignup.id, TEST_PASSWORD!, TEST_CLIENT_ID!);
    throw new Error("Should not be able to login with old guest credentials");
  } catch (error: any) {
    expect(error.message).toBe("Invalid email, password, or login method");
  }
  */
});

test("Guest refresh token works", async () => {
  // Sign up as guest
  const guestSignup = await fetchGuestSignUp(TEST_PASSWORD!, "", TEST_CLIENT_ID!);
  const guestLogin = await fetchGuestLogin(guestSignup.id, TEST_PASSWORD!, TEST_CLIENT_ID!);

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
  const guestSignup = await fetchGuestSignUp(TEST_PASSWORD!, "", TEST_CLIENT_ID!);
  const { refresh_token } = await fetchGuestLogin(guestSignup.id, TEST_PASSWORD!, TEST_CLIENT_ID!);
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

  // Test with any valid URL (now allowed)
  const anyValidUrl = "https://google.com";
  const googleResponse = await generateThirdPartyToken(anyValidUrl);
  expect(googleResponse.token).toBeDefined();
  expect(typeof googleResponse.token).toBe("string");
  expect(googleResponse.token.length).toBeGreaterThan(0);

  // Test with no audience (should work)
  const noAudienceResponse = await generateThirdPartyToken();
  expect(noAudienceResponse.token).toBeDefined();
  expect(typeof noAudienceResponse.token).toBe("string");
  expect(noAudienceResponse.token.length).toBeGreaterThan(0);

  // Test invalid audience URL (this should still fail)
  try {
    await generateThirdPartyToken("not-a-url");
    throw new Error("Should not accept invalid URL");
  } catch (error: any) {
    expect(error.message).toBe("Bad Request");
  }
});

test("Encrypt and decrypt data", async () => {
  // Login first to get authenticated
  const { access_token, refresh_token } = await tryEmailLogin();
  window.localStorage.setItem("access_token", access_token);
  window.localStorage.setItem("refresh_token", refresh_token);

  // Test data to encrypt
  const testData = "Hello, World!";

  // Encrypt the data
  const encryptResponse = await encryptData(testData);
  expect(encryptResponse.encrypted_data).toBeDefined();
  expect(typeof encryptResponse.encrypted_data).toBe("string");
  expect(encryptResponse.encrypted_data.length).toBeGreaterThan(0);

  // Decrypt the data
  const decryptedData = await decryptData(encryptResponse.encrypted_data);
  expect(decryptedData).toBe(testData);

  // Try with a derivation path
  const derivationPath = "m/44'/0'/0'/0/0";
  const encryptWithPathResponse = await encryptData(testData, derivationPath);
  expect(encryptWithPathResponse.encrypted_data).toBeDefined();

  // Decrypt with the same derivation path
  const decryptedWithPathData = await decryptData(
    encryptWithPathResponse.encrypted_data,
    derivationPath
  );
  expect(decryptedWithPathData).toBe(testData);

  // Try decrypting with a different derivation path (should fail)
  try {
    await decryptData(encryptWithPathResponse.encrypted_data, "m/44'/0'/0'/0/1");
    throw new Error("Should not decrypt with wrong derivation path");
  } catch (error: any) {
    expect(error.message).toBe("Bad Request");
  }

  // Try decrypting with no derivation path when it was encrypted with one (should fail)
  try {
    await decryptData(encryptWithPathResponse.encrypted_data);
    throw new Error("Should not decrypt with missing derivation path");
  } catch (error: any) {
    expect(error.message).toBe("Bad Request");
  }

  // Try with invalid encrypted data
  try {
    await decryptData("invalid-data");
    throw new Error("Should not decrypt invalid data");
  } catch (error: any) {
    expect(error.message).toBe("Bad Request");
  }
});
