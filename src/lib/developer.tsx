import React, { createContext, useState, useEffect } from "react";
import * as platformApi from "./platformApi";
import { setPlatformApiUrl } from "./platformApi";
import type {
  Organization,
  Project,
  ProjectSecret,
  ProjectSettings,
  EmailSettings,
  OAuthSettings,
  OrganizationMember,
  PlatformOrg,
  PlatformUser
} from "./platformApi";

export type DeveloperRole = "owner" | "admin" | "developer" | "viewer";

export type OrganizationDetails = Organization;

export type ProjectDetails = Project;

export { type ProjectSettings };

export type DeveloperResponse = PlatformUser & { organizations: PlatformOrg[] };

export type OpenSecretDeveloperState = {
  loading: boolean;
  developer?: DeveloperResponse;
};

export type OpenSecretDeveloperContextType = {
  developer: OpenSecretDeveloperState;

  /**
   * Signs in a developer with email and password
   * @param email - Developer's email address
   * @param password - Developer's password
   * @returns A promise that resolves to the login response with access and refresh tokens
   */
  signIn: (email: string, password: string) => Promise<platformApi.PlatformLoginResponse>;

  /**
   * Registers a new developer account
   * @param email - Developer's email address
   * @param password - Developer's password
   * @param name - Optional developer name
   * @returns A promise that resolves to the login response with access and refresh tokens
   */
  signUp: (
    email: string,
    password: string,
    name?: string
  ) => Promise<platformApi.PlatformLoginResponse>;

  /**
   * Signs out the current developer by removing authentication tokens
   */
  signOut: () => void;

  /**
   * Creates a new organization
   * @param name - Organization name
   * @returns A promise that resolves to the created organization
   */
  createOrganization: (name: string) => Promise<Organization>;

  /**
   * Lists all organizations the developer has access to
   * @returns A promise resolving to array of organization details
   */
  listOrganizations: () => Promise<Organization[]>;

  /**
   * Deletes an organization (requires owner role)
   * @param orgId - Organization ID
   */
  deleteOrganization: (orgId: string) => Promise<void>;

  /**
   * Creates a new project within an organization
   * @param orgId - Organization ID
   * @param name - Project name
   * @param description - Optional project description
   * @returns A promise that resolves to the project details including client ID
   */
  createProject: (orgId: string, name: string, description?: string) => Promise<Project>;

  /**
   * Lists all projects within an organization
   * @param orgId - Organization ID
   * @returns A promise resolving to array of project details
   */
  listProjects: (orgId: string) => Promise<Project[]>;

  /**
   * Updates project details
   * @param orgId - Organization ID
   * @param projectId - Project ID
   * @param updates - Object containing fields to update
   */
  updateProject: (
    orgId: string,
    projectId: string,
    updates: { name?: string; description?: string; status?: string }
  ) => Promise<Project>;

  /**
   * Deletes a project
   * @param orgId - Organization ID
   * @param projectId - Project ID
   */
  deleteProject: (orgId: string, projectId: string) => Promise<void>;

  /**
   * Creates a new secret for a project
   * @param orgId - Organization ID
   * @param projectId - Project ID
   * @param keyName - Secret key name (must be alphanumeric)
   * @param secret - Secret value (must be base64 encoded by the caller)
   *
   * Example:
   * ```typescript
   * // To encode a string secret
   * import { encode } from "@stablelib/base64";
   * const encodedSecret = encode(new TextEncoder().encode("my-secret-value"));
   *
   * // Now pass the encoded secret to the function
   * createProjectSecret(orgId, projectId, "mySecretKey", encodedSecret);
   * ```
   */
  createProjectSecret: (
    orgId: string,
    projectId: string,
    keyName: string,
    secret: string
  ) => Promise<ProjectSecret>;

  /**
   * Lists all secrets for a project
   * @param orgId - Organization ID
   * @param projectId - Project ID
   */
  listProjectSecrets: (orgId: string, projectId: string) => Promise<ProjectSecret[]>;

  /**
   * Deletes a project secret
   * @param orgId - Organization ID
   * @param projectId - Project ID
   * @param keyName - Secret key name
   */
  deleteProjectSecret: (orgId: string, projectId: string, keyName: string) => Promise<void>;

  /**
   * Gets email configuration for a project
   * @param orgId - Organization ID
   * @param projectId - Project ID
   */
  getEmailSettings: (orgId: string, projectId: string) => Promise<EmailSettings>;

  /**
   * Updates email configuration
   * @param orgId - Organization ID
   * @param projectId - Project ID
   * @param settings - Email settings
   */
  updateEmailSettings: (
    orgId: string,
    projectId: string,
    settings: EmailSettings
  ) => Promise<EmailSettings>;

  /**
   * Gets OAuth settings for a project
   * @param orgId - Organization ID
   * @param projectId - Project ID
   */
  getOAuthSettings: (orgId: string, projectId: string) => Promise<OAuthSettings>;

  /**
   * Updates OAuth configuration
   * @param orgId - Organization ID
   * @param projectId - Project ID
   * @param settings - OAuth settings
   */
  updateOAuthSettings: (
    orgId: string,
    projectId: string,
    settings: OAuthSettings
  ) => Promise<OAuthSettings>;

  /**
   * Creates an invitation to join an organization
   * @param orgId - Organization ID
   * @param email - Developer's email address
   * @param role - Role to assign (defaults to "admin")
   */
  inviteDeveloper: (orgId: string, email: string, role?: string) => Promise<{ code: string }>;

  /**
   * Lists all members of an organization
   * @param orgId - Organization ID
   */
  listOrganizationMembers: (orgId: string) => Promise<OrganizationMember[]>;

  /**
   * Updates a member's role
   * @param orgId - Organization ID
   * @param userId - User ID to update
   * @param role - New role to assign
   */
  updateMemberRole: (orgId: string, userId: string, role: string) => Promise<OrganizationMember>;

  /**
   * Removes a member from the organization
   * @param orgId - Organization ID
   * @param userId - User ID to remove
   */
  removeMember: (orgId: string, userId: string) => Promise<void>;

  /**
   * Accepts an organization invitation
   * @param code - Invitation code
   */
  acceptInvite: (code: string) => Promise<void>;

  /**
   * Returns the current OpenSecret developer API URL being used
   */
  apiUrl: string;
};

export const OpenSecretDeveloperContext = createContext<OpenSecretDeveloperContextType>({
  developer: {
    loading: true,
    developer: undefined
  },
  signIn: (email, password) => platformApi.platformLogin(email, password),
  signUp: (email, password, name) => platformApi.platformRegister(email, password, name),
  signOut: () => {
    localStorage.removeItem("access_token");
    localStorage.removeItem("refresh_token");
  },
  createOrganization: platformApi.createOrganization,
  listOrganizations: platformApi.listOrganizations,
  deleteOrganization: platformApi.deleteOrganization,
  createProject: platformApi.createProject,
  listProjects: platformApi.listProjects,
  updateProject: platformApi.updateProject,
  deleteProject: platformApi.deleteProject,
  createProjectSecret: platformApi.createProjectSecret,
  listProjectSecrets: platformApi.listProjectSecrets,
  deleteProjectSecret: platformApi.deleteProjectSecret,
  getEmailSettings: platformApi.getEmailSettings,
  updateEmailSettings: platformApi.updateEmailSettings,
  getOAuthSettings: platformApi.getOAuthSettings,
  updateOAuthSettings: platformApi.updateOAuthSettings,
  inviteDeveloper: platformApi.inviteDeveloper,
  listOrganizationMembers: platformApi.listOrganizationMembers,
  updateMemberRole: platformApi.updateMemberRole,
  removeMember: platformApi.removeMember,
  acceptInvite: platformApi.acceptInvite,
  apiUrl: ""
});

/**
 * Provider component for OpenSecret developer operations.
 * This provider is used for managing organizations, projects, and developer access.
 *
 * @param props - Configuration properties for the OpenSecret developer provider
 * @param props.children - React child components to be wrapped by the provider
 * @param props.apiUrl - URL of OpenSecret developer API
 *
 * @example
 * ```tsx
 * <OpenSecretDeveloper
 *   apiUrl='https://developer.opensecret.cloud'
 * >
 *   <App />
 * </OpenSecretDeveloper>
 * ```
 */
export function OpenSecretDeveloper({
  children,
  apiUrl
}: {
  children: React.ReactNode;
  apiUrl: string;
}) {
  const [developer, setDeveloper] = useState<OpenSecretDeveloperState>({
    loading: true,
    developer: undefined
  });

  useEffect(() => {
    if (!apiUrl || apiUrl.trim() === "") {
      throw new Error(
        "OpenSecretDeveloper requires a non-empty apiUrl. Please provide a valid API endpoint URL."
      );
    }
    setPlatformApiUrl(apiUrl);
  }, [apiUrl]);

  async function fetchDeveloper() {
    const access_token = window.localStorage.getItem("access_token");
    const refresh_token = window.localStorage.getItem("refresh_token");
    if (!access_token || !refresh_token) {
      setDeveloper({
        loading: false,
        developer: undefined
      });
      return;
    }

    try {
      const response = await platformApi.platformMe();
      setDeveloper({
        loading: false,
        developer: {
          ...response.user,
          organizations: response.organizations
        }
      });
    } catch (error) {
      console.error("Failed to fetch developer:", error);
      setDeveloper({
        loading: false,
        developer: undefined
      });
    }
  }

  useEffect(() => {
    fetchDeveloper();
  }, []);

  const value: OpenSecretDeveloperContextType = {
    developer,
    signIn: (email, password) => platformApi.platformLogin(email, password),
    signUp: (email, password, name) => platformApi.platformRegister(email, password, name),
    signOut: () => {
      localStorage.removeItem("access_token");
      localStorage.removeItem("refresh_token");
      setDeveloper({
        loading: false,
        developer: undefined
      });
    },
    createOrganization: platformApi.createOrganization,
    listOrganizations: platformApi.listOrganizations,
    deleteOrganization: platformApi.deleteOrganization,
    createProject: platformApi.createProject,
    listProjects: platformApi.listProjects,
    updateProject: platformApi.updateProject,
    deleteProject: platformApi.deleteProject,
    createProjectSecret: platformApi.createProjectSecret,
    listProjectSecrets: platformApi.listProjectSecrets,
    deleteProjectSecret: platformApi.deleteProjectSecret,
    getEmailSettings: platformApi.getEmailSettings,
    updateEmailSettings: platformApi.updateEmailSettings,
    getOAuthSettings: platformApi.getOAuthSettings,
    updateOAuthSettings: platformApi.updateOAuthSettings,
    inviteDeveloper: platformApi.inviteDeveloper,
    listOrganizationMembers: platformApi.listOrganizationMembers,
    updateMemberRole: platformApi.updateMemberRole,
    removeMember: platformApi.removeMember,
    acceptInvite: platformApi.acceptInvite,
    apiUrl
  };

  return (
    <OpenSecretDeveloperContext.Provider value={value}>
      {children}
    </OpenSecretDeveloperContext.Provider>
  );
}
