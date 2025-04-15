---
title: API Reference
sidebar_position: 1
---

# OpenSecret SDK API Reference

This documentation provides a comprehensive reference for the TypeScript SDK.

## Core API

The most important interface in the SDK is **[OpenSecretContextType](./type-aliases/OpenSecretContextType.md)** - this is what developers will use after initializing the SDK. It includes:

- Authentication methods (sign in, sign up)
- Data encryption and decryption
- Key management
- AI integration
- Remote attestation

### How to Set Up and Use the SDK

1. **Set up the provider in your React application:**
   ```tsx
   import { OpenSecretProvider } from '@opensecret/sdk';

   function App() {
     return (
       <OpenSecretProvider clientId="your-project-id">
         <YourApp />
       </OpenSecretProvider>
     );
   }
   ```

2. **Access the SDK in your components using the hook:**
   ```tsx
   import { useOpenSecret } from '@opensecret/sdk';

   function YourComponent() {
     const { 
       auth,                // Current auth state
       signIn,              // Authentication methods 
       signUp,
       signOut,
       encryptData,         // Data encryption
       decryptData,
       getPublicKey,        // Key management
       signMessage,
       // ... and more
     } = useOpenSecret();

     // Use the SDK features...
   }
   ```

## Key Components

- **[OpenSecretProvider](./functions/OpenSecretProvider.md)** - Main provider component to initialize the SDK
- **[useOpenSecret](./functions/useOpenSecret.md)** - React hook to access the SDK functionality
- **[OpenSecretContext](./variables/OpenSecretContext.md)** - React context (useful for custom hooks)

## Developer API

For platform developers, we also offer a set of tools for managing organizations, projects, and settings:

- **[OpenSecretDeveloperContextType](./type-aliases/OpenSecretDeveloperContextType.md)** - Developer context interface
- **[OpenSecretDeveloper](./functions/OpenSecretDeveloper.md)** - Developer provider
- **[useOpenSecretDeveloper](./functions/useOpenSecretDeveloper.md)** - Hook for developer features