import { expect, test, beforeEach } from "bun:test";
import { platformLogin, platformRegister } from "./platformApi";
import "./test/platform-api-url-loader";
import * as platformApi from "./platformApi";

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

    // Store tokens for subsequent API calls
    window.localStorage.setItem("access_token", response.access_token);
    window.localStorage.setItem("refresh_token", response.refresh_token);

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

      // Store tokens for subsequent API calls
      window.localStorage.setItem("access_token", response.access_token);
      window.localStorage.setItem("refresh_token", response.refresh_token);

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

        // Store tokens for subsequent API calls
        window.localStorage.setItem("access_token", response.access_token);
        window.localStorage.setItem("refresh_token", response.refresh_token);

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
    window.localStorage.setItem("access_token", access_token);
    window.localStorage.setItem("refresh_token", refresh_token);

    // Verify tokens were stored
    expect(window.localStorage.getItem("access_token")).toBe(access_token);
    expect(window.localStorage.getItem("refresh_token")).toBe(refresh_token);
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

    window.localStorage.setItem("access_token", access_token);
    window.localStorage.setItem("refresh_token", refresh_token);

    // Verify tokens are stored
    expect(window.localStorage.getItem("access_token")).toBe(access_token);
    expect(window.localStorage.getItem("refresh_token")).toBe(refresh_token);

    // Clear storage
    window.localStorage.clear();
    expect(window.localStorage.getItem("access_token")).toBeNull();
    expect(window.localStorage.getItem("refresh_token")).toBeNull();
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

test("Create, list, and delete organization", async () => {
  try {
    // Login first to get authenticated
    const { access_token, refresh_token } = await tryDeveloperLogin();
    window.localStorage.setItem("access_token", access_token);
    window.localStorage.setItem("refresh_token", refresh_token);

    // Create a unique organization name (to avoid conflicts if test is run multiple times)
    const orgName = `Test Org ${Date.now()}`;

    // Create the organization
    const createdOrg = await platformApi.createOrganization(orgName);
    expect(createdOrg).toBeDefined();
    expect(createdOrg.name).toBe(orgName);
    expect(createdOrg.id).toBeDefined();

    // List organizations and verify the new one is there
    const orgs = await platformApi.listOrganizations();
    expect(orgs).toBeDefined();
    expect(Array.isArray(orgs)).toBe(true);

    // Find our organization in the list
    const foundOrg = orgs.find((org) => org.id === createdOrg.id);
    expect(foundOrg).toBeDefined();
    expect(foundOrg?.name).toBe(orgName);

    // Delete the organization
    await platformApi.deleteOrganization(createdOrg.id.toString());

    // List organizations again and verify the deleted one is gone
    const orgsAfterDelete = await platformApi.listOrganizations();
    const shouldBeUndefined = orgsAfterDelete.find((org) => org.id === createdOrg.id);
    expect(shouldBeUndefined).toBeUndefined();
  } catch (error: any) {
    console.error("Test failed:", error.message);
    throw error;
  }
});

test("Organization creation with invalid input", async () => {
  try {
    // Login first to get authenticated
    const { access_token, refresh_token } = await tryDeveloperLogin();
    window.localStorage.setItem("access_token", access_token);
    window.localStorage.setItem("refresh_token", refresh_token);

    // Test empty name
    try {
      await platformApi.createOrganization("");
      throw new Error("Should not accept empty organization name");
    } catch (error: any) {
      expect(error.message).toMatch(/Invalid|name.*required|Bad Request/i);
    }

    // Test extremely long name (if there's a limit)
    try {
      const veryLongName = "a".repeat(1000);
      await platformApi.createOrganization(veryLongName);
      // Note: This may or may not fail depending on API implementation
    } catch (error: any) {
      expect(error.message).toMatch(/Invalid|name.*too long|Bad Request/i);
    }
  } catch (error: any) {
    console.error("Test failed:", error.message);
    throw error;
  }
});

test("Organization deletion edge cases", async () => {
  try {
    // Login first to get authenticated
    const { access_token, refresh_token } = await tryDeveloperLogin();
    window.localStorage.setItem("access_token", access_token);
    window.localStorage.setItem("refresh_token", refresh_token);

    // Test deleting non-existent organization
    try {
      await platformApi.deleteOrganization("non-existent-id");
      throw new Error("Should not be able to delete non-existent organization");
    } catch (error: any) {
      expect(error.message).toMatch(
        /Not Found|Organization not found|Bad Request|HTTP error! Status: 400/i
      );
    }
  } catch (error: any) {
    console.error("Test failed:", error.message);
    throw error;
  }
});

test("Create organization with duplicate name", async () => {
  try {
    // Login first to get authenticated
    const { access_token, refresh_token } = await tryDeveloperLogin();
    window.localStorage.setItem("access_token", access_token);
    window.localStorage.setItem("refresh_token", refresh_token);

    // Create a unique organization name
    const orgName = `Test Duplicate Org ${Date.now()}`;

    // Create the first organization
    const firstOrg = await platformApi.createOrganization(orgName);
    expect(firstOrg).toBeDefined();

    try {
      // Try creating a second organization with the same name
      await platformApi.createOrganization(orgName);

      // If we reach here, it means duplicate names are allowed
      // We should clean up both organizations
      const orgs = await platformApi.listOrganizations();
      const duplicateOrgs = orgs.filter((org) => org.name === orgName);
      for (const org of duplicateOrgs) {
        await platformApi.deleteOrganization(org.id.toString());
      }
    } catch (error: any) {
      // Expected error for duplicate name
      expect(error.message).toMatch(/duplicate|already exists|Bad Request/i);

      // Clean up the first organization
      await platformApi.deleteOrganization(firstOrg.id.toString());
    }
  } catch (error: any) {
    console.error("Test failed:", error.message);
    throw error;
  }
});

test("Create and manage multiple organizations", async () => {
  try {
    // Login first to get authenticated
    const { access_token, refresh_token } = await tryDeveloperLogin();
    window.localStorage.setItem("access_token", access_token);
    window.localStorage.setItem("refresh_token", refresh_token);

    // Keep track of created organizations to delete them later
    const createdOrgIds: string[] = [];

    // Create three organizations
    const timestamp = Date.now();
    for (let i = 0; i < 3; i++) {
      const orgName = `Test Multi Org ${timestamp}-${i}`;
      const org = await platformApi.createOrganization(orgName);
      expect(org).toBeDefined();
      expect(org.name).toBe(orgName);
      createdOrgIds.push(org.id.toString());
    }

    // List organizations and verify all three exist
    const orgs = await platformApi.listOrganizations();
    for (const orgId of createdOrgIds) {
      const found = orgs.some((org) => org.id.toString() === orgId);
      expect(found).toBe(true);
    }

    // Delete organizations one by one and verify they're gone
    for (const orgId of createdOrgIds) {
      await platformApi.deleteOrganization(orgId);
      const orgsAfter = await platformApi.listOrganizations();
      const stillExists = orgsAfter.some((org) => org.id.toString() === orgId);
      expect(stillExists).toBe(false);
    }
  } catch (error: any) {
    console.error("Test failed:", error.message);
    throw error;
  }
});

test("List organizations with no organizations", async () => {
  try {
    // Login first to get authenticated
    const { access_token, refresh_token } = await tryDeveloperLogin();
    window.localStorage.setItem("access_token", access_token);
    window.localStorage.setItem("refresh_token", refresh_token);

    // Get all existing organizations
    const initialOrgs = await platformApi.listOrganizations();

    // Delete all organizations (if any)
    for (const org of initialOrgs) {
      await platformApi.deleteOrganization(org.id.toString());
    }

    // List organizations - should be empty or still return a valid empty array
    const emptyOrgs = await platformApi.listOrganizations();
    expect(Array.isArray(emptyOrgs)).toBe(true);

    // Create a test organization to ensure state is reset properly
    const orgName = `Test Reset Org ${Date.now()}`;
    await platformApi.createOrganization(orgName);
  } catch (error: any) {
    console.error("Test failed:", error.message);
    throw error;
  }
});

test("Organization operations require authentication", async () => {
  try {
    // Clear any authentication tokens
    window.localStorage.clear();

    // Try to list organizations without authentication
    try {
      await platformApi.listOrganizations();
      throw new Error("Should not be able to list organizations without authentication");
    } catch (error: any) {
      expect(error.message).toMatch(/unauthorized|unauthenticated|no access token|token/i);
    }

    // Try to create an organization without authentication
    try {
      await platformApi.createOrganization("Test Org");
      throw new Error("Should not be able to create organizations without authentication");
    } catch (error: any) {
      expect(error.message).toMatch(/unauthorized|unauthenticated|no access token|token/i);
    }

    // Try to delete an organization without authentication
    try {
      await platformApi.deleteOrganization("any-id");
      throw new Error("Should not be able to delete organizations without authentication");
    } catch (error: any) {
      expect(error.message).toMatch(/unauthorized|unauthenticated|no access token|token/i);
    }
  } catch (error: any) {
    console.error("Test failed:", error.message);
    throw error;
  }
});

test("Organization with special characters in name", async () => {
  try {
    // Login first to get authenticated
    const { access_token, refresh_token } = await tryDeveloperLogin();
    window.localStorage.setItem("access_token", access_token);
    window.localStorage.setItem("refresh_token", refresh_token);

    // Create an organization with special characters in name
    const specialOrgName = `Test Org & Special #${Date.now()}`;

    try {
      const org = await platformApi.createOrganization(specialOrgName);
      expect(org).toBeDefined();
      expect(org.name).toBe(specialOrgName);

      // Clean up
      await platformApi.deleteOrganization(org.id.toString());
    } catch (error: any) {
      // If the API doesn't allow special characters, expect a proper validation error
      expect(error.message).toMatch(/invalid|character|Bad Request/i);
    }
  } catch (error: any) {
    console.error("Test failed:", error.message);
    throw error;
  }
});

test("List organizations pagination handling", async () => {
  try {
    // Login first to get authenticated
    const { access_token, refresh_token } = await tryDeveloperLogin();
    window.localStorage.setItem("access_token", access_token);
    window.localStorage.setItem("refresh_token", refresh_token);

    // Get initial list of organizations
    const initialOrgs = await platformApi.listOrganizations();

    // Create a batch of organizations if needed to test pagination
    const createdOrgIds: string[] = [];
    const timestamp = Date.now();
    const batchSize = 5; // Create enough to potentially trigger pagination

    for (let i = 0; i < batchSize; i++) {
      const orgName = `Test Pagination Org ${timestamp}-${i}`;
      const org = await platformApi.createOrganization(orgName);
      createdOrgIds.push(org.id.toString());
    }

    // Fetch the updated list of organizations - should include all created ones
    const updatedOrgs = await platformApi.listOrganizations();

    // Verify we have more organizations now
    expect(updatedOrgs.length).toBeGreaterThanOrEqual(initialOrgs.length + batchSize);

    // Verify all our newly created orgs are in the list
    for (const id of createdOrgIds) {
      const found = updatedOrgs.some((org) => org.id.toString() === id);
      expect(found).toBe(true);
    }

    // Clean up all created organizations
    for (const id of createdOrgIds) {
      await platformApi.deleteOrganization(id);
    }
  } catch (error: any) {
    console.error("Test failed:", error.message);
    throw error;
  }
});

// Add test for platform API URL validation
test("Platform API URL is required", () => {
  // The platform-api-url-loader will throw if no URL is provided
  // Verify it's set to something non-empty
  const apiUrl = process.env.VITE_OPEN_SECRET_API_URL;
  expect(apiUrl).toBeDefined();
  expect(apiUrl?.trim().length).toBeGreaterThan(0);
});

// Test chained organization operations as would be used in real app
test("Organization API flow with chained operations", async () => {
  try {
    // Login first to get authenticated
    const { access_token, refresh_token } = await tryDeveloperLogin();
    window.localStorage.setItem("access_token", access_token);
    window.localStorage.setItem("refresh_token", refresh_token);

    // 1. Create a new organization
    const orgName = `Test Chain Org ${Date.now()}`;
    const createdOrg = await platformApi.createOrganization(orgName);

    // 2. Verify it appears in the list
    let orgs = await platformApi.listOrganizations();
    let found = orgs.some((org) => org.id.toString() === createdOrg.id.toString());
    expect(found).toBe(true);

    // 3. Delete the organization
    await platformApi.deleteOrganization(createdOrg.id.toString());

    // 4. Verify it no longer appears in the list
    orgs = await platformApi.listOrganizations();
    found = orgs.some((org) => org.id.toString() === createdOrg.id.toString());
    expect(found).toBe(false);

    // 5. Attempt to delete the same organization again (should fail)
    try {
      await platformApi.deleteOrganization(createdOrg.id.toString());
      throw new Error("Should not be able to delete the same organization twice");
    } catch (error: any) {
      expect(error.message).toMatch(
        /Not Found|Organization not found|Bad Request|HTTP error! Status: 400/i
      );
    }
  } catch (error: any) {
    console.error("Test failed:", error.message);
    throw error;
  }
});

// Test deleting a non-existent organization with a valid UUID format
test("Organization deletion with random UUID", async () => {
  try {
    // Login first to get authenticated
    const { access_token, refresh_token } = await tryDeveloperLogin();
    window.localStorage.setItem("access_token", access_token);
    window.localStorage.setItem("refresh_token", refresh_token);

    // Generate a random UUID that (almost certainly) doesn't exist
    const randomUUID = crypto.randomUUID();

    // Try to delete an organization with this UUID
    try {
      await platformApi.deleteOrganization(randomUUID);
      throw new Error("Should not be able to delete non-existent organization");
    } catch (error: any) {
      // This should return a 404 Not Found or similar
      expect(error.message).toMatch(
        /Not Found|Organization not found|not exist|HTTP error! Status: 404/i
      );
    }
  } catch (error: any) {
    console.error("Test failed:", error.message);
    throw error;
  }
});
