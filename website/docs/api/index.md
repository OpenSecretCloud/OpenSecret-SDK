---
title: API Reference
sidebar_position: 1
---

# OpenSecret SDK API Reference

This documentation provides a comprehensive reference for the TypeScript SDK.

## Core API

The most important interface in the SDK is **OpenSecretContextType** - this is what developers will use after initializing the SDK. It includes:

- Authentication methods (sign in, sign up)
- Data encryption and decryption
- Key management
- AI integration
- Remote attestation

### How to Set Up and Use the SDK

**Set up the provider in your React application:**

```js
import { OpenSecretProvider } from '@opensecret/sdk';

function App() {
  return (
    <OpenSecretProvider clientId="your-project-id">
      <YourApp />
    </OpenSecretProvider>
  );
}
```

**Access the SDK in your components using the hook:**

```js
import { useOpenSecret } from '@opensecret/sdk';

function YourComponent() {
  const auth = useOpenSecret();
  // Use the SDK features
}
```

## Key Components

- **OpenSecretProvider** - Main provider component to initialize the SDK
- **useOpenSecret** - React hook to access the SDK functionality
- **OpenSecretContext** - React context (useful for custom hooks)

## Developer API

For platform developers, we also offer a set of tools for managing organizations, projects, and settings:

- **OpenSecretDeveloperContextType** - Developer context interface
- **OpenSecretDeveloper** - Developer provider
- **useOpenSecretDeveloper** - Hook for developer features