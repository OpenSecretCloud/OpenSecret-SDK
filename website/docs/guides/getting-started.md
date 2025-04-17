---
title: Getting Started
sidebar_position: 1
---

# Getting Started with OpenSecret

Welcome to the OpenSecret SDK documentation. This guide will help you get started with integrating OpenSecret into your application for secure, privacy-preserving data handling and AI interactions.

## What is OpenSecret?

OpenSecret is a platform that enables developers to build applications with strong security and privacy guarantees. By leveraging hardware-based security (using enclaves), OpenSecret provides:

- End-to-end encrypted storage
- Secure authentication 
- Privacy-preserving AI integration
- Hardware-verified security guarantees through remote attestation

## Prerequisites

Before you begin, you'll need:

1. An OpenSecret account and project (see [Registration Guide](../registration))
2. Your project's client ID (obtained from the OpenSecret dashboard)
3. A compatible JavaScript/TypeScript development environment (React application)

## Installation

You can install the OpenSecret SDK using npm, yarn, or bun:

```bash
# Using npm
npm install @opensecret/react

# Using yarn
yarn add @opensecret/react

# Using bun
bun add @opensecret/react
```

## Basic Setup

The OpenSecret SDK is built around React and provides a context-based integration for your application.

### 1. Wrap your application with the provider

First, you need to wrap your application with the `OpenSecretProvider` component. This provider requires two props:

- `apiUrl`: URL of the OpenSecret backend
- `clientId`: Your project's unique identifier

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

### 2. Access OpenSecret services with the hook

Now you can use the `useOpenSecret` hook to access all OpenSecret functionality:

```tsx
import { useOpenSecret } from "@opensecret/react";

function YourApp() {
  const os = useOpenSecret();

  // Now you can use os.signIn(), os.put(), etc.

  return (
    <div>
      {/* Your application UI */}
    </div>
  );
}
```

## Basic Authentication Example

Here's a simple login form component using OpenSecret:

```tsx
import React, { useState } from "react";
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

## Authentication Status

You can check the current authentication status through the `auth` property:

```tsx
function AuthStatus() {
  const os = useOpenSecret();
  
  if (os.auth.loading) {
    return <div>Loading...</div>;
  }
  
  if (os.auth.user) {
    return (
      <div>
        <p>Logged in as: {os.auth.user.email}</p>
        <button onClick={() => os.signOut()}>Log Out</button>
      </div>
    );
  }
  
  return <div>Not logged in</div>;
}
```

## Next Steps

Now that you have OpenSecret set up, you can explore more advanced features:

- [Authentication](./authentication) - Learn about different authentication methods
- [Key-Value Storage](./key-value-storage) - Store and retrieve encrypted data
- [Cryptographic Operations](./cryptographic-operations) - Work with keys and signatures
- [AI Integration](./ai-integration) - Use AI with privacy guarantees

For a complete reference of all available methods, check the [API Reference](../api).