# Developer Platform API

The Developer Platform API allows developers to manage organizations, projects, secrets, and user access within the OpenSecret platform. This API is specifically designed for developers integrating OpenSecret into their applications and platforms.

### `OpenSecretDeveloper`

The `OpenSecretDeveloper` component is the provider for developer-specific platform operations. It requires the URL of the OpenSecret developer API.

```tsx
import { OpenSecretDeveloper } from "@opensecret/react";

function App() {
  return (
    <OpenSecretDeveloper 
      apiUrl="https://developer.opensecret.cloud"
      pcrConfig={{}} // Optional PCR configuration for attestation validation
    >
      <YourApp />
    </OpenSecretDeveloper>
  );
}
```

### Developer Authentication

Before using the developer platform APIs, you need to authenticate. The SDK provides authentication methods through the `useOpenSecretDeveloper` hook:

```tsx
import { useOpenSecretDeveloper } from "@opensecret/react";

function DeveloperLogin() {
  const dev = useOpenSecretDeveloper();

  // Login with existing developer account
  async function handleLogin() {
    try {
      const response = await dev.signIn("developer@example.com", "yourpassword");
      console.log("Login successful", response);
      // Now you can use the developer context APIs
      // Authentication state is automatically updated
    } catch (error) {
      console.error("Login failed:", error);
    }
  }

  // Register a new developer account
  async function handleRegister() {
    try {
      const response = await dev.signUp(
        "developer@example.com", 
        "yourpassword", 
        "Developer Name" // Optional
      );
      console.log("Registration successful", response);
      // Now you can use the developer context APIs
      // Authentication state is automatically updated
    } catch (error) {
      console.error("Registration failed:", error);
    }
  }

  // Sign out
  async function handleLogout() {
    try {
      await dev.signOut();
      // The developer context will update automatically
    } catch (error) {
      console.error("Logout failed:", error);
    }
  }

  return (
    <div>
      {/* Login/Register UI */}
    </div>
  );
}
```

When a developer successfully logs in or registers, the authentication tokens are stored in localStorage and managed by the SDK. The `OpenSecretDeveloper` provider automatically detects these tokens and loads the developer profile. You can check the authentication state using the `developer` property:

```tsx
const dev = useOpenSecretDeveloper();

// Check if developer is loaded and authenticated
if (!dev.developer.loading && dev.developer.developer) {
  console.log("Developer is authenticated:", dev.developer.developer.email);
} else if (!dev.developer.loading) {
  console.log("Developer is not authenticated");
}
```

### `useOpenSecretDeveloper`

The `useOpenSecretDeveloper` hook provides access to all developer platform management APIs. It returns an object with the following properties and methods:

```tsx
import { useOpenSecretDeveloper } from "@opensecret/react";

function PlatformManagement() {
  const dev = useOpenSecretDeveloper();
  
  // Access developer information
  const { loading, developer } = dev.developer;
  
  // Now you can use any of the platform management methods
  // ...
}
```

#### Developer State

- `developer`: An object containing the current developer's information
  - `loading`: Boolean indicating whether developer information is being loaded
  - `developer`: Developer data (undefined if not logged in) including:
    - `id`: Developer's unique ID
    - `email`: Developer's email address
    - `name`: Developer's name (optional)
    - `organizations`: Array of organizations the developer belongs to
- `apiUrl`: The current OpenSecret developer API URL being used

#### Developer Authentication

- `signIn(email: string, password: string): Promise<PlatformLoginResponse>`: Signs in a developer with the provided email and password. Returns a response containing access and refresh tokens. The authentication state is automatically updated.
- `signUp(email: string, password: string, name?: string): Promise<PlatformLoginResponse>`: Registers a new developer account with the provided email, password, and optional name. Returns a response containing access and refresh tokens. The authentication state is automatically updated.
- `signOut(): Promise<void>`: Signs out the current developer by removing authentication tokens and making a server logout call.
- `refetchDeveloper(): Promise<void>`: Refreshes the developer's authentication state. Useful after making changes that affect developer profile or organization membership.

#### Attestation Verification

- `pcrConfig`: An object containing additional PCR0 hashes to validate against.
- `getAttestation`: Gets attestation from the enclave.
- `authenticate`: Authenticates an attestation document.
- `parseAttestationForView`: Parses an attestation document for viewing.
- `awsRootCertDer`: AWS root certificate in DER format.
- `expectedRootCertHash`: Expected hash of the AWS root certificate.
- `getAttestationDocument()`: Gets and verifies an attestation document from the enclave. This is a convenience function that:
  1. Fetches the attestation document with a random nonce
  2. Authenticates the document
  3. Parses it for viewing

#### Organization Management

- `createOrganization(name: string): Promise<Organization>`: Creates a new organization with the given name.
- `listOrganizations(): Promise<Organization[]>`: Lists all organizations the developer has access to.
- `deleteOrganization(orgId: string): Promise<void>`: Deletes an organization (requires owner role).

Example:
```tsx
const handleCreateOrg = async () => {
  try {
    const org = await dev.createOrganization("My New Organization");
    console.log("Created organization:", org);
  } catch (error) {
    console.error("Failed to create organization:", error);
  }
};
```

#### Project Management

- `createProject(orgId: string, name: string, description?: string): Promise<Project>`: Creates a new project within an organization.
- `listProjects(orgId: string): Promise<Project[]>`: Lists all projects within an organization.
- `updateProject(orgId: string, projectId: string, updates: { name?: string; description?: string; status?: string }): Promise<Project>`: Updates project details.
- `deleteProject(orgId: string, projectId: string): Promise<void>`: Deletes a project.

Example:
```tsx
const handleCreateProject = async (orgId) => {
  try {
    const project = await dev.createProject(
      orgId,
      "My New Project",
      "A description of my project"
    );
    console.log("Created project:", project);
    // project.client_id can be used as the clientId for OpenSecretProvider
  } catch (error) {
    console.error("Failed to create project:", error);
  }
};
```

#### Project Secrets Management

- `createProjectSecret(orgId: string, projectId: string, keyName: string, secret: string): Promise<ProjectSecret>`: Creates a new secret for a project. The secret must be base64 encoded.
- `listProjectSecrets(orgId: string, projectId: string): Promise<ProjectSecret[]>`: Lists all secrets for a project.
- `deleteProjectSecret(orgId: string, projectId: string, keyName: string): Promise<void>`: Deletes a project secret.

Example:
```tsx
import { encode } from "@stablelib/base64";

const handleCreateSecret = async (orgId, projectId) => {
  // Encode the secret
  const secretValue = "my-secret-value";
  const encodedSecret = encode(new TextEncoder().encode(secretValue));
  
  try {
    await dev.createProjectSecret(
      orgId,
      projectId,
      "API_KEY",
      encodedSecret
    );
    console.log("Secret created successfully");
  } catch (error) {
    console.error("Failed to create secret:", error);
  }
};
```

#### Email Configuration

- `getEmailSettings(orgId: string, projectId: string): Promise<EmailSettings>`: Gets email configuration for a project.
- `updateEmailSettings(orgId: string, projectId: string, settings: EmailSettings): Promise<EmailSettings>`: Updates email configuration.

Example:
```tsx
const handleUpdateEmailSettings = async (orgId, projectId) => {
  try {
    await dev.updateEmailSettings(orgId, projectId, {
      provider: "smtp",
      send_from: "noreply@yourdomain.com",
      email_verification_url: "https://yourdomain.com/verify-email"
    });
    console.log("Email settings updated");
  } catch (error) {
    console.error("Failed to update email settings:", error);
  }
};
```

#### OAuth Configuration

- `getOAuthSettings(orgId: string, projectId: string): Promise<OAuthSettings>`: Gets OAuth settings for a project.
- `updateOAuthSettings(orgId: string, projectId: string, settings: OAuthSettings): Promise<OAuthSettings>`: Updates OAuth configuration.

Example:
```tsx
const handleUpdateOAuthSettings = async (orgId, projectId) => {
  try {
    await dev.updateOAuthSettings(orgId, projectId, {
      google_oauth_enabled: true,
      github_oauth_enabled: false,
      google_oauth_settings: {
        client_id: "your-google-client-id",
        redirect_url: "https://yourdomain.com/auth/google/callback"
      }
    });
    console.log("OAuth settings updated");
  } catch (error) {
    console.error("Failed to update OAuth settings:", error);
  }
};
```

#### Developer Membership Management

- `inviteDeveloper(orgId: string, email: string, role?: string): Promise<{ code: string }>`: Creates an invitation to join an organization.
- `listOrganizationMembers(orgId: string): Promise<OrganizationMember[]>`: Lists all members of an organization.
- `updateMemberRole(orgId: string, userId: string, role: string): Promise<OrganizationMember>`: Updates a member's role.
- `removeMember(orgId: string, userId: string): Promise<void>`: Removes a member from the organization.
- `acceptInvite(code: string): Promise<void>`: Accepts an organization invitation.

Example:
```tsx
const handleInviteDeveloper = async (orgId) => {
  try {
    const result = await dev.inviteDeveloper(
      orgId,
      "developer@example.com",
      "admin" // Possible roles: "owner", "admin", "developer", "viewer"
    );
    console.log("Invitation sent with code:", result.code);
  } catch (error) {
    console.error("Failed to invite developer:", error);
  }
};
```

### Complete Example

Here's a complete example of how to use the developer platform API:

```tsx
import React, { useEffect, useState } from 'react';
import { OpenSecretDeveloper, useOpenSecretDeveloper } from '@opensecret/react';

function DeveloperPortal() {
  const dev = useOpenSecretDeveloper();
  const [orgs, setOrgs] = useState([]);
  const [selectedOrg, setSelectedOrg] = useState(null);
  const [projects, setProjects] = useState([]);
  
  useEffect(() => {
    // Load organizations when developer is available
    if (!dev.developer.loading && dev.developer.developer) {
      loadOrganizations();
    }
  }, [dev.developer.loading, dev.developer.developer]);
  
  async function loadOrganizations() {
    try {
      const orgs = await dev.listOrganizations();
      setOrgs(orgs);
    } catch (error) {
      console.error("Failed to load organizations:", error);
    }
  }
  
  async function loadProjects(orgId) {
    try {
      const projects = await dev.listProjects(orgId);
      setProjects(projects);
    } catch (error) {
      console.error("Failed to load projects:", error);
    }
  }
  
  async function handleCreateProject() {
    if (!selectedOrg) return;
    
    try {
      await dev.createProject(
        selectedOrg.id,
        "New Project " + Date.now(),
        "Created from developer portal"
      );
      // Reload projects
      loadProjects(selectedOrg.id);
    } catch (error) {
      console.error("Failed to create project:", error);
    }
  }
  
  return (
    <div>
      <h1>Developer Portal</h1>
      {dev.developer.loading ? (
        <p>Loading...</p>
      ) : dev.developer.developer ? (
        <div>
          <p>Welcome, {dev.developer.developer.name || dev.developer.developer.email}</p>
          
          <h2>Your Organizations</h2>
          <ul>
            {orgs.map(org => (
              <li key={org.id}>
                {org.name}
                <button onClick={() => {
                  setSelectedOrg(org);
                  loadProjects(org.id);
                }}>
                  Select
                </button>
              </li>
            ))}
          </ul>
          <button onClick={() => {
            const name = prompt("Organization name:");
            if (name) dev.createOrganization(name).then(loadOrganizations);
          }}>
            Create Organization
          </button>
          
          {selectedOrg && (
            <div>
              <h2>Projects in {selectedOrg.name}</h2>
              <ul>
                {projects.map(project => (
                  <li key={project.id}>
                    {project.name} - Client ID: {project.client_id}
                  </li>
                ))}
              </ul>
              <button onClick={handleCreateProject}>Create Project</button>
            </div>
          )}
        </div>
      ) : (
        <p>Please log in to access developer features</p>
      )}
    </div>
  );
}

function App() {
  return (
    <OpenSecretDeveloper apiUrl="https://developer.opensecret.cloud">
      <DeveloperPortal />
    </OpenSecretDeveloper>
  );
}

export default App;
```
