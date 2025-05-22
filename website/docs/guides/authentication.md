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

OpenSecret supports authentication through OAuth providers like GitHub, Google, and Apple. This allows your users to sign in using their existing accounts on these platforms.

:::tip
Before implementing OAuth authentication, you need to configure the OAuth providers in your project settings at [https://opensecret.cloud](https://opensecret.cloud). Navigate to your project's settings and look for the "Authentication" tab to set up each provider.
:::

### Setting Up OAuth Providers

For each OAuth provider, you'll need to:

1. Register your application with the provider (GitHub, Google, Apple, etc.)
2. Obtain Client ID and Client Secret from the provider
3. Configure redirect URLs (usually `https://api.opensecret.cloud/auth/[provider]/callback`)
4. Add these credentials in your OpenSecret project settings

For Apple authentication, you'll need to create:
- An Apple Developer account
- An App ID with "Sign In with Apple" capability
- A Services ID for web authentication or a Bundle ID for iOS apps
- A private key for creating client secrets
- Your Apple Developer Team ID
- Your Apple Developer Key ID for the private key

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

### Apple Authentication

Apple Sign-In is available in two forms:
1. Web-based OAuth (similar to GitHub and Google)
2. Native iOS integration

#### Web OAuth Authentication

To implement Apple OAuth login for web applications:

```tsx
import { useState } from "react";
import { useOpenSecret } from "@opensecret/react";

function AppleLoginButton() {
  const os = useOpenSecret();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  
  async function handleAppleLogin() {
    setLoading(true);
    setError("");
    
    try {
      const inviteCode = "your-invite-code"; // Or from a state/prop
      
      // Initiate Apple authentication
      const { auth_url } = await os.initiateAppleAuth(inviteCode);
      
      // Redirect to Apple for authentication
      window.location.href = auth_url;
    } catch (err) {
      setError(err instanceof Error ? err.message : "Apple login failed");
    } finally {
      setLoading(false);
    }
  }
  
  return (
    <button 
      onClick={handleAppleLogin} 
      disabled={loading}
      className="apple-login-button"
    >
      {loading ? "Connecting..." : "Sign in with Apple"}
    </button>
  );
}
```

#### Native iOS Authentication

For iOS apps using the native Sign in with Apple:

:::info Security Tip: Using a Nonce
For added security, you should generate a random nonce value when initiating Apple Sign In. This nonce:
- Must be passed to the Apple authentication request as a SHA256 hash of your raw nonce value
- Must be included as the raw (un-hashed) value in your request to the OpenSecret backend
- Will be verified by the OpenSecret backend to ensure the identity token was generated for your specific authentication request
- Helps prevent replay attacks where an attacker might try to reuse a stolen token

Important implementation detail:
- When sending the nonce to Apple, you must hash it using SHA256
- When sending the nonce to OpenSecret's `handleAppleNativeSignIn`, provide the original raw nonce value (not the hash)
- The backend will perform the same SHA256 hash and compare it with what's in the Apple JWT

Example nonce handling:
```typescript
// Generate a random nonce
const rawNonce = generateSecureRandomString();

// When initiating Sign in with Apple, pass the SHA256 hash of the nonce
const hashedNonce = sha256(rawNonce);
appleSignInNative({ nonce: hashedNonce }); // The nonce Apple receives

// When handling the callback, pass the original raw nonce to OpenSecret
const appleUser = {
  // ... other fields ...
  nonce: rawNonce // The raw value, not the hash
};
await os.handleAppleNativeSignIn(appleUser, inviteCode);
```

The OpenSecret backend will validate that the SHA256 hash of your provided nonce matches what's in the Apple JWT.
:::

```tsx
import { useState, useEffect } from "react";
import { useOpenSecret } from "@opensecret/react";

function NativeAppleAuth() {
  const os = useOpenSecret();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  // Store the generated nonce to use in both Apple authentication and backend verification
  const [nonce, setNonce] = useState("");
  
  // Generate a secure random nonce when component mounts
  useEffect(() => {
    // Generate a random nonce - in a real app, use a cryptographically secure method
    const generateNonce = () => {
      const randBytes = new Uint8Array(32);
      window.crypto.getRandomValues(randBytes);
      return Array.from(randBytes)
        .map(b => b.toString(16).padStart(2, "0"))
        .join("");
    };
    
    setNonce(generateNonce());
  }, []);
  
  // Initiate Apple Sign In with the generated nonce
  function initiateAppleSignIn() {
    // Hash the nonce before sending it to Apple
    const sha256 = async (text) => {
      const msgBuffer = new TextEncoder().encode(text);
      const hashBuffer = await crypto.subtle.digest('SHA-256', msgBuffer);
      const hashArray = Array.from(new Uint8Array(hashBuffer));
      return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
    };
    
    // In a real implementation, you would hash the nonce and pass it to your native Apple Sign In
    // This is platform-specific and might use a bridge to native code
    sha256(nonce).then(hashedNonce => {
      appleSignInNative(hashedNonce); // Pass the HASHED nonce to Apple
    });
  }
  
  // This function would be called when you receive the Apple Sign-In response
  async function completeAppleSignIn(appleAuthResponse) {
    setLoading(true);
    setError("");
    
    try {
      // Format the response for the API
      const appleUser = {
        user_identifier: appleAuthResponse.user,
        identity_token: appleAuthResponse.identityToken,
        email: appleAuthResponse.email,
        given_name: appleAuthResponse.fullName?.givenName,
        family_name: appleAuthResponse.fullName?.familyName,
        nonce: nonce // Include the same nonce used during Apple authentication
      };
      
      const inviteCode = "your-invite-code"; // Optional
      
      // Complete the native Apple Sign-In
      await os.handleAppleNativeSignIn(appleUser, inviteCode);
      
      // User is now authenticated
    } catch (err) {
      setError(err instanceof Error ? err.message : "Apple login failed");
    } finally {
      setLoading(false);
    }
  }
  
  return (
    <div>
      <button 
        onClick={initiateAppleSignIn} 
        disabled={loading || !nonce}
        className="apple-login-button"
      >
        {loading ? "Connecting..." : "Sign in with Apple"}
      </button>
      {error && <div className="error">{error}</div>}
    </div>
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
        const provider = location.pathname.split('/').pop();
        
        switch (provider) {
          case 'github':
            await os.handleGitHubCallback(code, state, inviteCode);
            break;
          case 'google':
            await os.handleGoogleCallback(code, state, inviteCode);
            break;
          case 'apple':
            await os.handleAppleCallback(code, state, inviteCode);
            break;
          default:
            throw new Error('Unknown OAuth provider');
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

For Apple Sign-In specifically:

- **Client ID**: For web authentication, use your Services ID (e.g., com.example.web); for iOS apps, use your Bundle ID
- **Client Secret**: The base64-encoded contents of your Apple private key (.p8 file)
- **Redirect URI**: Configure this in your Apple Developer console and the OpenSecret platform

When configuring your OpenSecret project settings:

1. Create project secrets with the following keys:
   - `APPLE_CLIENT_ID` - Your Apple Services ID or Bundle ID
   - `APPLE_CLIENT_SECRET` - Your Apple private key (.p8 file) contents, base64-encoded
   - `APPLE_TEAM_ID` - Your Apple Developer Team ID
   - `APPLE_KEY_ID` - Your Apple Developer Key ID for the private key

2. To base64-encode your private key file, run this command in a terminal:
   ```bash
   base64 -i AuthKey_KEYID.p8 | tr -d '\n'
   ```

3. In your OAuth project settings, enable Apple Sign-In and configure the redirect URL.

:::note
The OpenSecret backend will handle generating the necessary JWT tokens for communicating with Apple. You don't need to pre-generate or manage JWT tokens yourself.
:::

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

### Account Deletion

OpenSecret provides a secure two-step verification process for account deletion, requiring both email verification and a client-side secret to prevent unauthorized deletion requests.

The process works as follows:
1. The user requests account deletion, generating a secure client-side secret
2. OpenSecret sends a verification email with a confirmation code to the user's email address
3. The user confirms deletion by providing both the confirmation code from the email and the original client-side secret
4. The account and all associated data are permanently deleted

To implement account deletion in your application:

```tsx
import { useState } from "react";
import { useOpenSecret } from "@opensecret/react";
import { generateSecureSecret, hashSecret } from "./utils"; // Implement these utility functions

function AccountDeletionFlow() {
  const os = useOpenSecret();
  const [step, setStep] = useState("request"); // 'request' or 'confirm'
  const [secret, setSecret] = useState("");
  const [confirmationCode, setConfirmationCode] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [success, setSuccess] = useState(false);

  // Step 1: Initialize the deletion process
  const handleInitiateDeletion = async () => {
    setLoading(true);
    setError("");

    try {
      // Generate a random secret and store it
      const newSecret = generateSecureSecret(16); // Implement this function to generate a secure random string
      setSecret(newSecret);

      // Hash the secret before sending to server
      const hashedSecret = await hashSecret(newSecret); // Implement this function to securely hash the secret

      // Request account deletion
      await os.requestAccountDeletion(hashedSecret);

      // Move to confirmation step
      setStep("confirm");
      setSuccess(true);
    } catch (error) {
      setError(error instanceof Error ? error.message : "Failed to request account deletion");
      console.error(error);
    } finally {
      setLoading(false);
    }
  };

  // Step 2: Confirm the deletion with confirmation code from email
  const handleConfirmDeletion = async () => {
    setLoading(true);
    setError("");

    try {
      // Use the confirmation code from email and the stored secret
      await os.confirmAccountDeletion(confirmationCode, secret);

      // Account deleted successfully
      setSuccess(true);

      // Redirect to logout or homepage
      setTimeout(() => {
        // Clear local storage, cookies, etc.
        localStorage.clear();
        sessionStorage.clear();
        window.location.href = "/";
      }, 2000);
    } catch (error) {
      setError(error instanceof Error ? error.message : "Failed to confirm account deletion");
      console.error(error);
    } finally {
      setLoading(false);
    }
  };

  // Render UI based on current step
  return (
    <div className="account-deletion-container">
      <h2>Delete Your Account</h2>

      {step === "request" && (
        <div>
          <p>
            Warning: This action will permanently delete your account and all associated data.
            This cannot be undone.
          </p>
          <button 
            onClick={handleInitiateDeletion}
            disabled={loading}
          >
            {loading ? "Processing..." : "Delete My Account"}
          </button>
        </div>
      )}

      {step === "confirm" && success && (
        <div>
          <p>
            A confirmation email has been sent to your email address.
            Please check your email and enter the confirmation code below.
          </p>
          <input
            type="text"
            placeholder="Enter confirmation code from email"
            value={confirmationCode}
            onChange={(e) => setConfirmationCode(e.target.value)}
          />
          <button 
            onClick={handleConfirmDeletion}
            disabled={loading || !confirmationCode}
          >
            {loading ? "Processing..." : "Confirm Deletion"}
          </button>
        </div>
      )}

      {error && <p className="error">{error}</p>}

      {success && step === "confirm" && (
        <p className="success">Your account has been successfully deleted.</p>
      )}
    </div>
  );
}
```

The `generateSecureSecret` and `hashSecret` utility functions might be implemented as follows:

```tsx
// Generate a secure random string of specified length
export function generateSecureSecret(length: number): string {
  const array = new Uint8Array(length);
  window.crypto.getRandomValues(array);
  return Array.from(array)
    .map(b => b.toString(16).padStart(2, "0"))
    .join("");
}

// Hash a string using SHA-256
export async function hashSecret(secret: string): Promise<string> {
  const encoder = new TextEncoder();
  const data = encoder.encode(secret);
  const hashBuffer = await crypto.subtle.digest("SHA-256", data);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  const hashHex = hashArray
    .map(b => b.toString(16).padStart(2, "0"))
    .join("");
  return hashHex;
}
```

:::important
The account deletion process is permanent and cannot be undone. Make sure to clearly communicate this to users before they initiate the process.
:::

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