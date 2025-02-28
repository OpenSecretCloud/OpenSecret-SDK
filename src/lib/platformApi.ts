import { encryptedApiCall, authenticatedApiCall } from "./encryptedApi";

// Platform Auth Types
export type PlatformLoginResponse = {
  id: string;
  email: string;
  name?: string;
  access_token: string;
  refresh_token: string;
};

export type PlatformRefreshResponse = {
  access_token: string;
  refresh_token: string;
};

// Platform User Types
export type PlatformOrg = {
  id: string;
  name: string;
  role?: string;
  created_at?: string;
  updated_at?: string;
};

export type PlatformUser = {
  id: string;
  email: string;
  name?: string;
  email_verified: boolean;
  created_at: string;
  updated_at: string;
};

export type MeResponse = {
  user: PlatformUser;
  organizations: PlatformOrg[];
};

// Organization Types
export type Organization = {
  id: string;
  name: string;
};

export type Project = {
  id: string;
  client_id: string;
  name: string;
  description?: string;
  status: string;
  created_at: string;
};

export type ProjectSecret = {
  key_name: string;
  created_at: string;
  updated_at: string;
};

export type ProjectSettings = {
  category: string;
  settings: Record<string, unknown>;
  created_at: string;
  updated_at: string;
};

export type EmailSettings = {
  provider: string;
  send_from: string;
  email_verification_url: string;
};

/**
 * Provider-specific OAuth settings
 */
export type OAuthProviderSettings = {
  client_id: string;
  redirect_url: string;
};

export type OAuthSettings = {
  google_oauth_enabled: boolean;
  github_oauth_enabled: boolean;
  google_oauth_settings?: OAuthProviderSettings;
  github_oauth_settings?: OAuthProviderSettings;
};

export type OrganizationMember = {
  user_id: string;
  role: string;
};

let platformApiUrl = "";

export function setPlatformApiUrl(url: string) {
  platformApiUrl = url;
}

export function getPlatformApiUrl(): string {
  return platformApiUrl;
}

// Platform Authentication
export async function platformLogin(
  email: string,
  password: string
): Promise<PlatformLoginResponse> {
  return encryptedApiCall<{ email: string; password: string }, PlatformLoginResponse>(
    `${platformApiUrl}/platform/login`,
    "POST",
    { email, password },
    undefined,
    "Failed to login"
  );
}

export async function platformRegister(
  email: string,
  password: string,
  name?: string
): Promise<PlatformLoginResponse> {
  return encryptedApiCall<
    { email: string; password: string; name?: string },
    PlatformLoginResponse
  >(
    `${platformApiUrl}/platform/register`,
    "POST", 
    { email, password, name },
    undefined,
    "Failed to register"
  );
}

export async function platformLogout(refresh_token: string): Promise<void> {
  return encryptedApiCall<{ refresh_token: string }, void>(
    `${platformApiUrl}/platform/logout`,
    "POST",
    { refresh_token },
    undefined,
    "Failed to logout"
  );
}

/**
 * Refreshes platform access and refresh tokens
 * 
 * This function:
 * 1. Gets the refresh token from localStorage
 * 2. Calls the platform-specific refresh endpoint (/platform/refresh)
 * 3. Updates localStorage with the new tokens
 * 
 * The platform refresh endpoint expects:
 * - A refresh token with audience "platform_refresh" in the request body
 * - The request to be encrypted according to the platform's encryption scheme
 * 
 * It returns new access and refresh tokens if validation succeeds.
 */
export async function platformRefreshToken(): Promise<PlatformRefreshResponse> {
  const refresh_token = window.localStorage.getItem("refresh_token");
  if (!refresh_token) throw new Error("No refresh token available");

  const refreshData = { refresh_token };

  try {
    const response = await encryptedApiCall<typeof refreshData, PlatformRefreshResponse>(
      `${platformApiUrl}/platform/refresh`,
      "POST",
      refreshData,
      undefined,
      "Failed to refresh platform token"
    );

    window.localStorage.setItem("access_token", response.access_token);
    window.localStorage.setItem("refresh_token", response.refresh_token);
    return response;
  } catch (error) {
    console.error("Error refreshing platform token:", error);
    throw error;
  }
}

// Organization Management
export async function createOrganization(name: string): Promise<Organization> {
  return authenticatedApiCall<{ name: string }, Organization>(
    `${platformApiUrl}/platform/orgs`,
    "POST",
    { name }
  );
}

export async function listOrganizations(): Promise<Organization[]> {
  return authenticatedApiCall<void, Organization[]>(
    `${platformApiUrl}/platform/orgs`,
    "GET",
    undefined
  );
}

export async function deleteOrganization(orgId: string): Promise<void> {
  return authenticatedApiCall<void, void>(
    `${platformApiUrl}/platform/orgs/${orgId}`,
    "DELETE",
    undefined
  );
}

// Project Management
export async function createProject(
  orgId: string,
  name: string,
  description?: string
): Promise<Project> {
  return authenticatedApiCall<{ name: string; description?: string }, Project>(
    `${platformApiUrl}/platform/orgs/${orgId}/projects`,
    "POST",
    { name, description }
  );
}

export async function listProjects(orgId: string): Promise<Project[]> {
  return authenticatedApiCall<void, Project[]>(
    `${platformApiUrl}/platform/orgs/${orgId}/projects`,
    "GET",
    undefined
  );
}

export async function getProject(orgId: string, projectId: string): Promise<Project> {
  return authenticatedApiCall<void, Project>(
    `${platformApiUrl}/platform/orgs/${orgId}/projects/${projectId}`,
    "GET",
    undefined
  );
}

export async function updateProject(
  orgId: string,
  projectId: string,
  updates: { name?: string; description?: string; status?: string }
): Promise<Project> {
  return authenticatedApiCall<typeof updates, Project>(
    `${platformApiUrl}/platform/orgs/${orgId}/projects/${projectId}`,
    "PATCH",
    updates
  );
}

export async function deleteProject(orgId: string, projectId: string): Promise<void> {
  return authenticatedApiCall<void, void>(
    `${platformApiUrl}/platform/orgs/${orgId}/projects/${projectId}`,
    "DELETE",
    undefined
  );
}

// Helper function to check if a string is valid base64
function isValidBase64(str: string): boolean {
  // Base64 should have a length that is a multiple of 4
  // It should only contain characters A-Z, a-z, 0-9, +, /, and end with '=' or '=='
  const base64Regex = /^[A-Za-z0-9+/]*[=]{0,2}$/;
  
  // Check if the string length is a multiple of 4
  const validLength = str.length % 4 === 0;
  
  // Check if the string only contains valid base64 characters
  const validChars = base64Regex.test(str);
  
  return validLength && validChars;
}

// Project Secrets
export async function createProjectSecret(
  orgId: string,
  projectId: string,
  keyName: string,
  secret: string
): Promise<ProjectSecret> {
  // Validate that the secret is base64 encoded
  if (!isValidBase64(secret)) {
    throw new Error("Secret must be base64 encoded. Use @stablelib/base64's encode function to encode your data.");
  }
  
  return authenticatedApiCall<{ key_name: string; secret: string }, ProjectSecret>(
    `${platformApiUrl}/platform/orgs/${orgId}/projects/${projectId}/secrets`,
    "POST",
    { key_name: keyName, secret }
  );
}

export async function listProjectSecrets(
  orgId: string,
  projectId: string
): Promise<ProjectSecret[]> {
  return authenticatedApiCall<void, ProjectSecret[]>(
    `${platformApiUrl}/platform/orgs/${orgId}/projects/${projectId}/secrets`,
    "GET",
    undefined
  );
}

export async function deleteProjectSecret(
  orgId: string,
  projectId: string,
  keyName: string
): Promise<void> {
  return authenticatedApiCall<void, void>(
    `${platformApiUrl}/platform/orgs/${orgId}/projects/${projectId}/secrets/${keyName}`,
    "DELETE",
    undefined
  );
}

// Email Settings
export async function getEmailSettings(orgId: string, projectId: string): Promise<EmailSettings> {
  return authenticatedApiCall<void, EmailSettings>(
    `${platformApiUrl}/platform/orgs/${orgId}/projects/${projectId}/settings/email`,
    "GET",
    undefined
  );
}

export async function updateEmailSettings(
  orgId: string,
  projectId: string,
  settings: EmailSettings
): Promise<EmailSettings> {
  return authenticatedApiCall<EmailSettings, EmailSettings>(
    `${platformApiUrl}/platform/orgs/${orgId}/projects/${projectId}/settings/email`,
    "PUT",
    settings
  );
}

// OAuth Settings
export async function getOAuthSettings(orgId: string, projectId: string): Promise<OAuthSettings> {
  return authenticatedApiCall<void, OAuthSettings>(
    `${platformApiUrl}/platform/orgs/${orgId}/projects/${projectId}/settings/oauth`,
    "GET",
    undefined
  );
}

export async function updateOAuthSettings(
  orgId: string,
  projectId: string,
  settings: OAuthSettings
): Promise<OAuthSettings> {
  return authenticatedApiCall<OAuthSettings, OAuthSettings>(
    `${platformApiUrl}/platform/orgs/${orgId}/projects/${projectId}/settings/oauth`,
    "PUT",
    settings
  );
}

// Organization Membership
export async function inviteDeveloper(
  orgId: string,
  email: string,
  role?: string
): Promise<{ code: string }> {
  // Add validation for empty emails
  if (!email || email.trim() === "") {
    throw new Error("Email is required");
  }

  return authenticatedApiCall<{ email: string; role?: string }, { code: string }>(
    `${platformApiUrl}/platform/orgs/${orgId}/invites`,
    "POST",
    { email, role }
  );
}

export async function listOrganizationMembers(orgId: string): Promise<OrganizationMember[]> {
  return authenticatedApiCall<void, OrganizationMember[]>(
    `${platformApiUrl}/platform/orgs/${orgId}/memberships`,
    "GET",
    undefined
  );
}

export async function updateMemberRole(
  orgId: string,
  userId: string,
  role: string
): Promise<OrganizationMember> {
  return authenticatedApiCall<{ role: string }, OrganizationMember>(
    `${platformApiUrl}/platform/orgs/${orgId}/memberships/${userId}`,
    "PATCH",
    { role }
  );
}

export async function removeMember(orgId: string, userId: string): Promise<void> {
  return authenticatedApiCall<void, void>(
    `${platformApiUrl}/platform/orgs/${orgId}/memberships/${userId}`,
    "DELETE",
    undefined
  );
}

export async function acceptInvite(code: string): Promise<void> {
  return authenticatedApiCall<void, void>(
    `${platformApiUrl}/platform/accept_invite/${code}`,
    "POST",
    undefined
  );
}

// Platform User
export async function platformMe(): Promise<MeResponse> {
  return authenticatedApiCall<void, MeResponse>(`${platformApiUrl}/platform/me`, "GET", undefined);
}
