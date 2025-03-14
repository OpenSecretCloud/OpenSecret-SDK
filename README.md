# OpenSecret React SDK

This is a React SDK for the [OpenSecret](https://opensecret.cloud) platform.

🚧 We're currently in preview mode, please contact us at team@opensecret.cloud for the preview URL and getting started info 🚧

## Installation

```bash
npm install @opensecret/react
```

## Usage

Wrap your application in the `OpenSecretProvider` component and provide:
1. The URL of your OpenSecret backend
2. Your project's client ID (a UUID that identifies your project)

```tsx
import { OpenSecretProvider } from "@opensecret/react";

function App() {
  return (
    <OpenSecretProvider 
      apiUrl="{URL}"
      clientId="{PROJECT_UUID}"
    >
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

The `OpenSecretProvider` component is the main entry point for the SDK. It requires two props:
- `apiUrl`: The URL of your OpenSecret backend
- `clientId`: A UUID that identifies your project/tenant. This is used to scope user accounts and data to your specific project.

```tsx
<OpenSecretProvider 
  apiUrl="{URL}"
  clientId="{PROJECT_UUID}"
>
  <App />
</OpenSecretProvider>
```

### `useOpenSecret`

The `useOpenSecret` hook provides access to the OpenSecret API. It returns an object with the following methods:

#### Authentication Methods
- `signIn(email: string, password: string): Promise<void>`: Signs in a user with the provided email and password.
- `signUp(email: string, password: string, inviteCode: string, name?: string): Promise<void>`: Signs up a new user with the provided email, password, invite code, and optional name.
- `signInGuest(id: string, password: string): Promise<void>`: Signs in a guest user with their ID and password. Guest accounts are scoped to the project specified by `clientId`.
- `signUpGuest(password: string, inviteCode: string): Promise<LoginResponse>`: Creates a new guest account with just a password and invite code. Returns a response containing the guest's ID, access token, and refresh token. The guest account will be associated with the project specified by `clientId`.
- `convertGuestToUserAccount(email: string, password: string, name?: string): Promise<void>`: Converts current guest account to a regular account with email authentication. Optionally sets the user's name. The account remains associated with the same project it was created under.
- `signOut(): Promise<void>`: Signs out the current user.

#### Key-Value Storage Methods
- `get(key: string): Promise<string | undefined>`: Retrieves the value associated with the provided key.
- `put(key: string, value: string): Promise<string>`: Stores the provided value with the provided key.
- `list(): Promise<KVListItem[]>`: Retrieves all key-value pairs stored by the user.
- `del(key: string): Promise<void>`: Deletes the value associated with the provided key.

#### Account Management Methods
- `refetchUser(): Promise<void>`: Refreshes the user's authentication state.
- `changePassword(currentPassword: string, newPassword: string): Promise<void>`: Changes the user's password.
- `generateThirdPartyToken(audience?: string): Promise<{ token: string }>`: Generates a JWT token for use with third-party services. If an audience is provided, it can be any valid URL. If omitted, a token with no audience restriction will be generated.

#### Cryptographic Methods
- `getPrivateKey(): Promise<{ mnemonic: string }>`: Retrieves the user's private key mnemonic phrase. This is used for cryptographic operations and should be kept secure.

- `getPrivateKeyBytes(derivationPath?: string): Promise<{ private_key: string }>`: Retrieves the private key bytes for a given BIP32 derivation path. If no path is provided, returns the master private key bytes.
  - Supports both absolute (starting with "m/") and relative paths
  - Supports hardened derivation using either ' or h notation
    Examples:
    - Absolute path: "m/44'/0'/0'/0/0"
    - Relative path: "0'/0'/0'/0/0"
    - Hardened notation: "44'" or "44h"
  - Common paths:
    - BIP44 (Legacy): `m/44'/0'/0'/0/0`
    - BIP49 (SegWit): `m/49'/0'/0'/0/0`
    - BIP84 (Native SegWit): `m/84'/0'/0'/0/0`
    - BIP86 (Taproot): `m/86'/0'/0'/0/0`

- `getPublicKey(algorithm: 'schnorr' | 'ecdsa', derivationPath?: string): Promise<PublicKeyResponse>`: Retrieves the user's public key for the specified signing algorithm and optional derivation path. The derivation path determines which child key pair is used, allowing different public keys to be generated from the same master key. This is useful for:
  - Separating keys by purpose (e.g., different chains or applications)
  - Generating deterministic addresses
  - Supporting different address formats (Legacy, SegWit, Native SegWit, Taproot)
  
  Supports two algorithms:
  - `'schnorr'`: For Schnorr signatures
  - `'ecdsa'`: For ECDSA signatures

- `signMessage(messageBytes: Uint8Array, algorithm: 'schnorr' | 'ecdsa', derivationPath?: string): Promise<SignatureResponse>`: Signs a message using the specified algorithm and optional derivation path. The message must be provided as a Uint8Array of bytes. Returns a signature that can be verified using the corresponding public key.
  
  Example message preparation:
  ```typescript
  // From string
  const messageBytes = new TextEncoder().encode("Hello, World!");
  
  // From hex
  const messageBytes = new Uint8Array(Buffer.from("deadbeef", "hex"));
  ```

- `encryptData(data: string, derivationPath?: string): Promise<{ encrypted_data: string }>`: Encrypts arbitrary string data using the user's private key.
  - Uses AES-256-GCM encryption with a random nonce for each operation
  - If derivation_path is provided, encryption uses the derived key at that path
  - If derivation_path is omitted, encryption uses the master key
  - Returns base64-encoded encrypted data containing the nonce, ciphertext, and authentication tag
  
  Example:
  ```typescript
  // Encrypt with master key
  const { encrypted_data } = await os.encryptData("Secret message");
  
  // Encrypt with derived key
  const { encrypted_data } = await os.encryptData("Secret message", "m/44'/0'/0'/0/0");
  ```

- `decryptData(encryptedData: string, derivationPath?: string): Promise<string>`: Decrypts data that was previously encrypted with the user's key.
  - Uses AES-256-GCM decryption with authentication tag verification
  - If derivation_path is provided, decryption uses the derived key at that path
  - If derivation_path is omitted, decryption uses the master key
  - The encrypted_data must be in the exact format returned by the encrypt endpoint
  
  Example:
  ```typescript
  // Decrypt with master key
  const decrypted = await os.decryptData(encrypted_data);
  
  // Decrypt with derived key (must use same path as encryption)
  const decrypted = await os.decryptData(encrypted_data, "m/44'/0'/0'/0/0");
  ```

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

To test the library, run the following command:

```bash
bun test --env-file .env.local
```

To test a specific file or test case:

```bash
bun test --test-name-pattern="Developer login and token storage" src/lib/developer.test.ts --env-file .env.local
```

Currently this build step requires `npx` because of [a Bun incompatibility with `vite-plugin-dts`](https://github.com/OpenSecretCloud/OpenSecret-SDK/issues/16).

To pack the library (for publishing) run the following command:

```bash
bun run pack
```

To deploy: 

```bash
NPM_CONFIG_TOKEN=$NPM_CONFIG_TOKEN bun publish --access public
```

## License

This project is licensed under the MIT License.

