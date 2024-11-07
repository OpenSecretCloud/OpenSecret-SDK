import { expect, test } from "bun:test";
import { fetchLogin, fetchSignUp, fetchLogout, refreshToken, fetchUser } from "./api";

const TEST_EMAIL = "test@test.com";
const TEST_PASSWORD = "test";
const TEST_NAME = "Test User";
const TEST_INVITE_CODE = "bearclaw24";

async function tryLogin() {
  // Ensure the test user exists
  try {
    return await fetchLogin(TEST_EMAIL, TEST_PASSWORD);
  } catch (error) {
    console.warn(error);
    console.log("Login failed, attempting signup");
    await fetchSignUp(TEST_NAME, TEST_EMAIL, TEST_PASSWORD, TEST_INVITE_CODE);
    return await fetchLogin(TEST_EMAIL, TEST_PASSWORD);
  }
}

test("Login and fetch user data", async () => {
  const { access_token, refresh_token } = await tryLogin();
  expect(access_token).toBeDefined();
  expect(refreshToken).toBeDefined();
  window.localStorage.setItem("access_token", access_token);
  window.localStorage.setItem("refresh_token", refresh_token);

  const userResponse = await fetchUser();
  expect(userResponse.user.email).toBe(TEST_EMAIL);
});

test("Refresh token works", async () => {
  await tryLogin();

  const refreshResponse = await refreshToken();
  expect(refreshResponse.access_token).toBeDefined();
  expect(refreshResponse.refresh_token).toBeDefined();

  // Verify the new access token works
  const userResponse = await fetchUser();
  expect(userResponse.user.email).toBe(TEST_EMAIL);
});

test("Logout doesn't error", async () => {
  const { refresh_token } = await tryLogin();
  await fetchLogout(refresh_token);
});
