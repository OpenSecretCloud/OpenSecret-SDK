---
title: Introduction
slug: /
sidebar_position: 0
---

# OpenSecret SDK Documentation

Welcome to the official documentation for the OpenSecret SDK. This SDK enables you to build secure, privacy-preserving applications with end-to-end encryption, secure key management, and AI capabilities.

## What is OpenSecret?

OpenSecret is a platform that provides advanced security and privacy features through hardware-based secure enclaves. It offers:

- **End-to-end encryption** for user data
- **Cryptographic operations** for secure key management and signing
- **Secure key-value storage** for encrypted user data
- **Privacy-preserving AI integration** with encryption to the GPU
- **Remote attestation** to verify server security guarantees
- **Authentication solutions** with flexible options including guest accounts

## Key Features

- **Hardware-based security**: Leveraging AWS Nitro Enclaves for true hardware isolation
- **Client-side encryption**: Encrypt sensitive data before it leaves the user's device
- **Deterministic key derivation**: Derive secure keys from user authentication
- **Guest accounts**: Support anonymous users with full security features
- **AI integration**: Use AI models with end-to-end encryption for prompts and responses
- **Remote attestation**: Cryptographically verify the security of the server
- **Comprehensive React SDK**: Easy integration with React applications

## Getting Started

To start using OpenSecret in your application:

1. [Register for an OpenSecret account](./registration)
2. Set up your project and get your client ID
3. Install the SDK: `npm install @opensecret/react`
4. Wrap your application with the `OpenSecretProvider`
5. Use the `useOpenSecret` hook to access SDK functionality

For more detailed instructions, check out the [Getting Started Guide](./guides/getting-started).

## Documentation Sections

### Guides

Step-by-step guides to help you implement specific features:

- [Getting Started](./guides/getting-started) - Basic setup and usage
- [Authentication](./guides/authentication) - User authentication methods
- [Key-Value Storage](./guides/key-value-storage) - Secure data storage
- [Guest Accounts](./guides/guest-accounts) - Anonymous user support
- [Cryptographic Operations](./guides/cryptographic-operations) - Keys, signing, and verification
- [Data Encryption](./guides/data-encryption) - Client-side encryption
- [AI Integration](./guides/ai-integration) - Encrypted AI capabilities
- [Remote Attestation](./guides/remote-attestation) - Server security verification
- [Third-Party Tokens](./guides/third-party-tokens) - JWT token generation

### API Reference

Detailed reference documentation for all SDK classes, methods, and types:

- [Functions](./api/functions/index) - Core SDK functions
- [Interfaces](./api/interfaces/index) - SDK interfaces
- [Type Aliases](./api/type-aliases/index) - SDK type definitions
- [Variables](./api/variables/index) - SDK constants and exports

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

## Resources

- [GitHub Repository](https://github.com/OpenSecretCloud/OpenSecret-SDK)
- [OpenSecret Website](https://opensecret.cloud)
- [Community Forum](https://community.opensecret.cloud)

## Getting Help

If you encounter any issues or have questions about the SDK:

- Check the [API Reference](./api/index) for detailed documentation
- Search the [Community Forum](https://community.opensecret.cloud) for similar issues
- Contact us at [support@opensecret.cloud](mailto:support@opensecret.cloud)

Ready to get started? [Register for an account](./registration) or jump into the [Getting Started Guide](./guides/getting-started).