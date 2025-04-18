---
title: Authentication
sidebar_position: 2
---

# Authentication with OpenSecret

This guide covers the various authentication methods supported by the OpenSecret SDK, including email/password login, guest accounts, and social authentication.

## Authentication Overview

OpenSecret provides a comprehensive authentication system that:

- Securely stores user credentials in hardware-protected enclaves
- Supports various authentication methods
- Manages authentication tokens automatically
- Provides a unified interface through the `useOpenSecret` hook

## Setting Up the Provider

Before using any authentication methods, wrap your application with the `OpenSecretProvider`:

```tsx
import { OpenSecretProvider } from "@opensecret/react";

function App() {
  return (
    <OpenSecretProvider 
      apiUrl="https://api.opensecret.cloud"
      clientId="your-project-uuid"
    >
      <YourApp />
    </OpenSecretProvider>
  );
}
```

## Email and Password Authentication

The most common authentication method is email and password.

### User Registration

To register a new user with email and password:

```tsx
import { useState } from "react";
import { useOpenSecret } from "@opensecret/react";

function SignupForm() {
  const os = useOpenSecret();
  const [name, setName] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [inviteCode, setInviteCode] = useState("");
  const [error, setError] = useState("");

  async function handleSignup(e) {
    e.preventDefault();
    setError("");
    
    try {
      await os.signUp(email, password, inviteCode, name);
      // User is now registered and logged in
    } catch (err) {
      setError(err instanceof Error ? err.message : "Signup failed");
    }
  }

  return (
    <form onSubmit={handleSignup}>
      <div>
        <label htmlFor="name">Name</label>
        <input 
          id="name"
          type="text" 
          value={name} 
          onChange={(e) => setName(e.target.value)} 
          required
        />
      </div>
      <div>
        <label htmlFor="email">Email</label>
        <input 
          id="email"
          type="email" 
          value={email} 
          onChange={(e) => setEmail(e.target.value)} 
          required
        />
      </div>
      <div>
        <label htmlFor="password">Password</label>
        <input 
          id="password"
          type="password" 
          value={password} 
          onChange={(e) => setPassword(e.target.value)} 
          minLength={8}
          required
        />
      </div>
      <div>
        <label htmlFor="inviteCode">Invite Code</label>
        <input 
          id="inviteCode"
          type="text" 
          value={inviteCode} 
          onChange={(e) => setInviteCode(e.target.value)} 
          required
        />
      </div>
      {error && <div className="error">{error}</div>}
      <button type="submit">Sign Up</button>
    </form>
  );
}
```

### User Login

To log in an existing user:

```tsx
import { useState } from "react";
import { useOpenSecret } from "@opensecret/react";

function LoginForm() {
  const os = useOpenSecret();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");

  async function handleLogin(e) {
    e.preventDefault();
    setError("");
    
    try {
      await os.signIn(email, password);
      // User is now logged in
    } catch (err) {
      setError(err instanceof Error ? err.message : "Login failed");
    }
  }

  return (
    <form onSubmit={handleLogin}>
      <div>
        <label htmlFor="email">Email</label>
        <input 
          id="email"
          type="email" 
          value={email} 
          onChange={(e) => setEmail(e.target.value)} 
          required
        />
      </div>
      <div>
        <label htmlFor="password">Password</label>
        <input 
          id="password"
          type="password" 
          value={password} 
          onChange={(e) => setPassword(e.target.value)} 
          required
        />
      </div>
      {error && <div className="error">{error}</div>}
      <button type="submit">Log In</button>
    </form>
  );
}
```

## OAuth Authentication

OpenSecret supports authentication through OAuth providers like GitHub and Google. This allows your users to sign in using their existing accounts on these platforms.

:::tip
Before implementing OAuth authentication, you need to configure the OAuth providers in your project settings at [https://opensecret.cloud](https://opensecret.cloud). Navigate to your project's settings and look for the "Authentication" tab to set up each provider.
:::

### Setting Up OAuth Providers

For each OAuth provider, you'll need to:

1. Register your application with the provider (GitHub, Google, etc.)
2. Obtain Client ID and Client Secret from the provider
3. Configure redirect URLs (usually `https://api.opensecret.cloud/auth/[provider]/callback`)
4. Add these credentials in your OpenSecret project settings

### GitHub Authentication

To implement GitHub login:

```tsx
import { useState } from "react";
import { useOpenSecret } from "@opensecret/react";

function GitHubLoginButton() {
  const os = useOpenSecret();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  
  async function handleGitHubLogin() {
    setLoading(true);
    setError("");
    
    try {
      // Get the invitation code if needed for your application
      const inviteCode = "your-invite-code"; // Or from a state/prop
      
      // Initiate GitHub authentication
      const { auth_url } = await os.initiateGitHubAuth(inviteCode);
      
      // Redirect to GitHub for authentication
      window.location.href = auth_url;
      
      // After user authenticates with GitHub, they will be redirected back to your app
      // The callback handling should be implemented separately
    } catch (err) {
      setError(err instanceof Error ? err.message : "GitHub login failed");
    } finally {
      setLoading(false);
    }
  }
  
  return (
    <button 
      onClick={handleGitHubLogin} 
      disabled={loading}
      className="github-login-button"
    >
      {loading ? "Connecting..." : "Sign in with GitHub"}
    </button>
  );
}
```

### Google Authentication

Similar to GitHub, you can implement Google authentication:

```tsx
import { useState } from "react";
import { useOpenSecret } from "@opensecret/react";

function GoogleLoginButton() {
  const os = useOpenSecret();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  
  async function handleGoogleLogin() {
    setLoading(true);
    setError("");
    
    try {
      const inviteCode = "your-invite-code"; // Or from a state/prop
      
      // Initiate Google authentication
      const { auth_url } = await os.initiateGoogleAuth(inviteCode);
      
      // Redirect to Google for authentication
      window.location.href = auth_url;
    } catch (err) {
      setError(err instanceof Error ? err.message : "Google login failed");
    } finally {
      setLoading(false);
    }
  }
  
  return (
    <button 
      onClick={handleGoogleLogin} 
      disabled={loading}
      className="google-login-button"
    >
      {loading ? "Connecting..." : "Sign in with Google"}
    </button>
  );
}
```

### Handling OAuth Callbacks

After a user authenticates with an OAuth provider, they are redirected back to your application. You need to handle this callback to complete the authentication process:

```tsx
import { useEffect } from "react";
import { useOpenSecret } from "@opensecret/react";
import { useLocation, useNavigate } from "react-router-dom";

function OAuthCallback() {
  const os = useOpenSecret();
  const location = useLocation();
  const navigate = useNavigate();
  
  useEffect(() => {
    async function handleCallback() {
      // Parse URL parameters
      const params = new URLSearchParams(location.search);
      const code = params.get("code");
      const state = params.get("state");
      const inviteCode = params.get("invite_code") || "your-invite-code";
      
      if (!code || !state) {
        console.error("Missing code or state parameter");
        navigate("/login?error=invalid_callback");
        return;
      }
      
      try {
        // Determine which OAuth provider based on your routing
        const isGitHub = location.pathname.includes("github");
        
        if (isGitHub) {
          await os.handleGitHubCallback(code, state, inviteCode);
        } else {
          // Assume Google if not GitHub
          await os.handleGoogleCallback(code, state, inviteCode);
        }
        
        // Authentication successful, redirect to dashboard
        navigate("/dashboard");
      } catch (error) {
        console.error("OAuth callback error:", error);
        navigate(`/login?error=${encodeURIComponent(error.message)}`);
      }
    }
    
    handleCallback();
  }, [location]);
  
  return <div>Completing authentication, please wait...</div>;
}
```

:::note
The above example uses React Router for navigation, but you can adapt it to your preferred routing solution.
:::

### OAuth Configuration in Project Settings

When setting up OAuth in your OpenSecret project settings, you will need to provide:

1. **Client ID**: The identifier for your application issued by the OAuth provider
2. **Client Secret**: The secret key for your application (keep this secure)
3. **Redirect URI**: The callback URL for your application (often automatically configured)

![OAuth Settings](https://placeholder-for-oauth-settings-screenshot.png)

## Guest Accounts

Guest accounts allow users to access your application without providing an email address. This is useful for quick onboarding, demos, or applications where user identity is not initially important.

For full details on implementing guest accounts, see the [Guest Accounts](./guest-accounts) guide.

## Managing the Authentication State

The OpenSecret SDK provides an `auth` object through the `useOpenSecret` hook that contains information about the current authentication state:

```tsx
import { useOpenSecret } from "@opensecret/react";

function AuthenticationStatus() {
  const os = useOpenSecret();
  
  if (os.auth.loading) {
    return <div>Loading authentication state...</div>;
  }
  
  if (!os.auth.user) {
    return <div>Not authenticated</div>;
  }
  
  return (
    <div>
      <h3>Authenticated User</h3>
      <p>ID: {os.auth.user.id}</p>
      <p>Email: {os.auth.user.email || "Guest Account"}</p>
      <p>Name: {os.auth.user.name || "Not set"}</p>
      <button onClick={() => os.signOut()}>Sign Out</button>
    </div>
  );
}
```

## Authentication Tokens

The SDK manages authentication tokens automatically. Access tokens and refresh tokens are stored in localStorage and used to maintain the user's session.

### Refreshing User Information

You can manually refresh the current user's information:

```tsx
import { useOpenSecret } from "@opensecret/react";

function RefreshUserButton() {
  const os = useOpenSecret();
  
  async function handleRefresh() {
    try {
      await os.refetchUser();
      alert("User information refreshed");
    } catch (error) {
      console.error("Failed to refresh user:", error);
    }
  }
  
  return <button onClick={handleRefresh}>Refresh User Info</button>;
}
```

### Signing Out

To sign out the current user:

```tsx
import { useOpenSecret } from "@opensecret/react";

function SignOutButton() {
  const os = useOpenSecret();
  
  async function handleSignOut() {
    try {
      await os.signOut();
      // User is now signed out
    } catch (error) {
      console.error("Sign out failed:", error);
    }
  }
  
  return <button onClick={handleSignOut}>Sign Out</button>;
}
```

## Creating Third-Party Tokens

OpenSecret allows you to generate JWT tokens for third-party services:

```tsx
import { useState } from "react";
import { useOpenSecret } from "@opensecret/react";

function TokenGenerator() {
  const os = useOpenSecret();
  const [audience, setAudience] = useState("https://your-service.com");
  const [token, setToken] = useState("");
  const [error, setError] = useState("");

  async function generateToken() {
    try {
      setError("");
      const response = await os.generateThirdPartyToken(audience);
      setToken(response.token);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to generate token");
    }
  }

  return (
    <div>
      <div>
        <label htmlFor="audience">Audience URL</label>
        <input
          id="audience"
          type="url"
          value={audience}
          onChange={(e) => setAudience(e.target.value)}
          placeholder="Enter audience URL"
        />
        <button onClick={generateToken}>Generate Token</button>
      </div>
      {error && <div className="error">{error}</div>}
      {token && (
        <div>
          <h4>Generated Token:</h4>
          <textarea
            readOnly
            value={token}
            rows={4}
            style={{ width: "100%" }}
          />
          <button
            onClick={() => navigator.clipboard.writeText(token)}
          >
            Copy to Clipboard
          </button>
        </div>
      )}
    </div>
  );
}
```

For more detailed information on third-party tokens, see the [Third-Party Tokens](./third-party-tokens) guide.

## Password Management

### Changing User Password

To allow users to change their password:

```tsx
import { useState } from "react";
import { useOpenSecret } from "@opensecret/react";

function ChangePasswordForm() {
  const os = useOpenSecret();
  const [currentPassword, setCurrentPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [error, setError] = useState("");
  const [success, setSuccess] = useState(false);

  async function handleChangePassword(e) {
    e.preventDefault();
    setError("");
    setSuccess(false);
    
    try {
      await os.changePassword(currentPassword, newPassword);
      setSuccess(true);
      setCurrentPassword("");
      setNewPassword("");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Password change failed");
    }
  }

  return (
    <form onSubmit={handleChangePassword}>
      <div>
        <label htmlFor="currentPassword">Current Password</label>
        <input 
          id="currentPassword"
          type="password" 
          value={currentPassword} 
          onChange={(e) => setCurrentPassword(e.target.value)} 
          required
        />
      </div>
      <div>
        <label htmlFor="newPassword">New Password</label>
        <input 
          id="newPassword"
          type="password" 
          value={newPassword} 
          onChange={(e) => setNewPassword(e.target.value)} 
          minLength={8}
          required
        />
      </div>
      {error && <div className="error">{error}</div>}
      {success && <div className="success">Password changed successfully</div>}
      <button type="submit">Change Password</button>
    </form>
  );
}
```

## Best Practices

1. **Always check authentication state**: Before accessing protected resources or displaying sensitive information, verify that the user is authenticated.

2. **Handle errors gracefully**: Display user-friendly error messages when authentication fails.

3. **Consider guest accounts for easy onboarding**: Guest accounts can reduce friction in your user onboarding process.

4. **Implement proper logout**: Always call `signOut()` when the user wants to log out to ensure all tokens are properly cleared.

5. **Secure your client ID**: While not a secret, your client ID should be handled respectfully as it identifies your application.

## Next Steps

- [Guest Accounts](./guest-accounts) - Learn about anonymous user accounts
- [Key-Value Storage](./key-value-storage) - Learn how to use the secure key-value storage for user data
- [Remote Attestation](./remote-attestation) - Understand how OpenSecret verifies the security of your data
- [Third-Party Tokens](./third-party-tokens) - Explore JWT token generation for third-party services