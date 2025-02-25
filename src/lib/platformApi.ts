import { encryptedApiCall, authenticatedApiCall } from "./encryptedApi";
import { encode } from "@stablelib/base64";

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
  id: number;
  uuid: string;
  client_id: string;
  name: string;
  description?: string;
  status: string;
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
    { email, password }
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
  >(`${platformApiUrl}/platform/register`, "POST", { email, password, name });
}

export async function platformRefreshToken(
  refresh_token: string
): Promise<PlatformRefreshResponse> {
  return encryptedApiCall<{ refresh_token: string }, PlatformRefreshResponse>(
    `${platformApiUrl}/platform/refresh`,
    "POST",
    { refresh_token }
  );
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

// Project Secrets
export async function createProjectSecret(
  orgId: string,
  projectId: string,
  keyName: string,
  secret: string
): Promise<ProjectSecret> {
  // The secret parameter should already be base64 encoded by the caller
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
