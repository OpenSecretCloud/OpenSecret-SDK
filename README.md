# OpenSecret React SDK

This is a React SDK for the [OpenSecret](https://opensecret.cloud) platform.

## Installation

```bash
npm install @opensecret/react
```

## Usage

Wrap your application in the `OpenSecretProvider` component and provide the URL of your OpenSecret backend:

```tsx
import { OpenSecretProvider } from "@opensecret/react";

function App() {
  return (
    <OpenSecretProvider apiUrl="https://preview-enclave.opensecret.cloud">
      <App />
    </OpenSecretProvider>
  );
}
```

Now import the `useOpenSecret` hook and use it to access the OpenSecret API:

```tsx
import { useOpenSecret } from "@opensecret/react";

function App() {
  const os = useOpenSecret();

  return (
    <div>
      <button onClick={() => os.signIn("email", "password")}>Sign In</button>
      <button onClick={() => os.signUp("name", "email", "password")}>Sign Up</button>
      <button onClick={() => os.signOut()}>Sign Out</button>
      <button onClick={() => os.get("key")}>Get Value</button>
      <button onClick={() => os.put("key", "value")}>Put Value</button>
      <button onClick={() => os.list()}>List Values</button>
      <button onClick={() => os.del("key")}>Delete Value</button>
    </div>
  );
}
```

## API Reference

### `OpenSecretProvider`

The `OpenSecretProvider` component is the main entry point for the SDK. It requires a single prop, `apiUrl`, which should be set to the URL of your OpenSecret backend.

```tsx
<OpenSecretProvider apiUrl="https://preview-enclave.opensecret.cloud">
  <App />
</OpenSecretProvider>
```

### `useOpenSecret`

The `useOpenSecret` hook provides access to the OpenSecret API. It returns an object with the following methods:

#### Authentication Methods
- `signIn(email: string, password: string): Promise<void>`: Signs in a user with the provided email and password.
- `signUp(name: string, email: string, password: string, inviteCode: string): Promise<void>`: Signs up a new user with the provided name, email, password, and invite code.
- `signOut(): Promise<void>`: Signs out the current user.

#### Key-Value Storage Methods
- `get(key: string): Promise<string | undefined>`: Retrieves the value associated with the provided key.
- `put(key: string, value: string): Promise<string>`: Stores the provided value with the provided key.
- `list(): Promise<KVListItem[]>`: Retrieves all key-value pairs stored by the user.
- `del(key: string): Promise<void>`: Deletes the value associated with the provided key.

#### Account Management Methods
- `refetchUser(): Promise<void>`: Refreshes the user's authentication state.
- `changePassword(currentPassword: string, newPassword: string): Promise<void>`: Changes the user's password.

#### Cryptographic Methods
- `getPrivateKey(): Promise<PrivateKeyResponse>`: Retrieves the user's private key mnemonic phrase. This is used for cryptographic operations and should be kept secure.

- `getPublicKey(algorithm: 'schnorr' | 'ecdsa'): Promise<PublicKeyResponse>`: Retrieves the user's public key for the specified signing algorithm. Supports two algorithms:
  - `'schnorr'`: For Schnorr signatures
  - `'ecdsa'`: For ECDSA signatures

- `signMessage(messageBytes: Uint8Array, algorithm: 'schnorr' | 'ecdsa'): Promise<SignatureResponse>`: Signs a message using the specified algorithm. The message must be provided as a Uint8Array of bytes. Returns a signature that can be verified using the corresponding public key.

### AI Integration

To get encrypted-to-the-gpu AI chat we provide a special version of `fetch` (`os.aiCustomFetch`) that handles all the encryption. Because we require the user to be logged in, and do the encryption client-side, this is safe to call from the client.

The easiest way to use this is through the OpenAI client:

```bash
npm install openai
```

```typescript
import OpenAI from "openai";
import { useOpenSecret } from "@opensecret/react";

//...

// In a component
const os = useOpenSecret();

const openai = new OpenAI({
  baseURL: `${os.apiUrl}/v1/`,
  dangerouslyAllowBrowser: true,
  apiKey: "api-key-doesnt-matter", // The actual API key is handled by OpenSecret
  defaultHeaders: {
    "Accept-Encoding": "identity",
    "Content-Type": "application/json",
  },
  fetch: os.aiCustomFetch, // Use OpenSecret's encrypted fetch
});

//...
```

You can now use the OpenAI client as normal. (Right now only streaming responses are supported.) See the example in `src/AI.tsx` in the SDK source code for a complete example.

For an alternative approach using custom fetch directly, see the implementation in `src/lib/ai.test.ts` in the SDK source code.

### Library development

This library uses [Bun](https://bun.sh/) for development.

To run the demo app, run the following commands:

```bash
bun install
bun run dev
```

To build the library, run the following command:

```bash
bun run build
```

To pack the library, run the following command:

```bash
bun run pack
```

## License

This project is licensed under the MIT License.
