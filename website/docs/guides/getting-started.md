---
title: Getting Started
sidebar_position: 1
---

# Getting Started with OpenSecret SDK

Welcome to the OpenSecret SDK documentation. This SDK provides a secure way to interact with the OpenSecret Cloud platform.

## Installation

You can install the OpenSecret SDK using npm, yarn, or bun:

```bash
# Using npm
npm install @opensecret/sdk

# Using yarn
yarn add @opensecret/sdk

# Using bun
bun add @opensecret/sdk
```

## Basic Usage

Here's a simple example of how to use the OpenSecret SDK:

```typescript
import { OpenSecretProvider, useOpenSecret } from '@opensecret/sdk';

// Setup the provider at the root of your application
function App() {
  return (
    <OpenSecretProvider clientId="your-client-id">
      <YourApp />
    </OpenSecretProvider>
  );
}

// Use the SDK in your components
function YourApp() {
  const { auth, signIn } = useOpenSecret();

  async function handleLogin() {
    try {
      await signIn('user@example.com', 'password');
      console.log('Logged in!');
    } catch (error) {
      console.error('Login failed:', error);
    }
  }

  return (
    <div>
      {auth.user ? (
        <p>Welcome, {auth.user.name}!</p>
      ) : (
        <button onClick={handleLogin}>Log In</button>
      )}
    </div>
  );
}
```

## Features

The OpenSecret SDK provides:

- **Secure Authentication** - User authentication with email/password or third-party providers
- **Secure Storage** - Encrypted data storage with client-side encryption
- **Remote Attestation** - Verify the server's security posture before sending sensitive data
- **AI Capabilities** - Integrate with AI features while maintaining privacy guarantees

## Next Steps

Check out the [API Reference](/docs/api/) for detailed information about all the available methods and classes in the SDK.

For more advanced usage examples, explore the other guides in the sidebar.