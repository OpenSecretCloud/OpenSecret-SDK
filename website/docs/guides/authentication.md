---
title: Authentication
sidebar_position: 2
---

# Authentication with OpenSecret SDK

This guide explains how to authenticate users with the OpenSecret SDK.

## Setting Up Authentication

The OpenSecret SDK provides a React context provider that handles authentication state and token management. To set up authentication, wrap your application with the `OpenSecretProvider` component:

```typescript
import { OpenSecretProvider } from '@opensecret/sdk';

function App() {
  return (
    <OpenSecretProvider clientId="your-client-id">
      {/* Your application components */}
    </OpenSecretProvider>
  );
}
```

## Authentication Methods

The SDK supports multiple authentication methods:

### Email/Password Authentication

```typescript
import { useOpenSecret } from '@opensecret/sdk';

function LoginForm() {
  const { signIn } = useOpenSecret();
  
  async function handleSubmit(email, password) {
    try {
      await signIn(email, password);
      // User is now logged in
    } catch (error) {
      // Handle error
    }
  }
  
  // Your form JSX
}
```

### Third-Party Authentication

The SDK supports authentication with third-party providers like GitHub and Google:

```typescript
import { useOpenSecret } from '@opensecret/sdk';

function LoginButtons() {
  const { signInWithGithub, signInWithGoogle } = useOpenSecret();
  
  async function handleGithubLogin() {
    try {
      await signInWithGithub();
      // User is now logged in
    } catch (error) {
      // Handle error
    }
  }
  
  // Similar function for Google login
  
  return (
    <div>
      <button onClick={handleGithubLogin}>Login with GitHub</button>
      {/* Google login button */}
    </div>
  );
}
```

## Managing Authentication State

The SDK provides access to the current authentication state through the `useOpenSecret` hook:

```typescript
import { useOpenSecret } from '@opensecret/sdk';

function UserProfile() {
  const { auth } = useOpenSecret();
  
  if (auth.loading) {
    return <p>Loading...</p>;
  }
  
  if (!auth.user) {
    return <p>Please log in</p>;
  }
  
  return (
    <div>
      <h2>Welcome, {auth.user.name}</h2>
      <p>Email: {auth.user.email}</p>
      {/* Other user profile information */}
    </div>
  );
}
```

## Logging Out

To log a user out, use the `signOut` method:

```typescript
import { useOpenSecret } from '@opensecret/sdk';

function LogoutButton() {
  const { signOut } = useOpenSecret();
  
  async function handleLogout() {
    await signOut();
    // User is now logged out
  }
  
  return <button onClick={handleLogout}>Logout</button>;
}
```

## Security Considerations

The OpenSecret SDK uses secure token storage and management:

- Access tokens are stored in memory
- Refresh tokens are stored in localStorage with proper encryption
- Tokens are automatically refreshed when needed
- Remote attestation verifies the server's security posture

For enhanced security in production applications, consider implementing additional security measures like token rotation and secure storage strategies.