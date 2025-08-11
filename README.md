# OpenSecret React SDK

This is a React SDK for the [OpenSecret](https://opensecret.cloud) platform.

🚧 We're currently in preview mode, please contact us at team@opensecret.cloud for the preview URL and getting started info 🚧

## Installation

```bash
npm install @opensecret/react
```

## Usage

### Direct API Usage (Recommended)

Configure the SDK once in your application and then use the API functions directly. This approach is compatible with React Suspense and gives you full control over state management.

```tsx
import { configure, signIn, signUp, signOut, get, put, list, del } from "@opensecret/react";

// Configure once at app initialization
configure({
  apiUrl: "{URL}",
  clientId: "{PROJECT_UUID}"
});

// Use functions directly in your components
function App() {
  const handleSignIn = async () => {
    try {
      await signIn("email", "password");
      // Handle successful sign in
      // Tokens are automatically stored in localStorage
    } catch (error) {
      // Handle error
    }
  };

  return (
    <div>
      <button onClick={handleSignIn}>Sign In</button>
      <button onClick={() => signUp("email", "password", "inviteCode")}>Sign Up</button>
      <button onClick={() => signOut()}>Sign Out</button>
      <button onClick={async () => console.log(await get("key"))}>Get Value</button>
      <button onClick={() => put("key", "value")}>Put Value</button>
      <button onClick={async () => console.log(await list())}>List Values</button>
      <button onClick={() => del("key")}>Delete Value</button>
    </div>
  );
}
```

### With React Suspense and TanStack Query

The direct API approach works seamlessly with modern data fetching libraries:

```tsx
import { useSuspenseQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { configure, fetchUser, signIn, signOut } from '@opensecret/react';

// Configure once
configure({
  apiUrl: "{URL}",
  clientId: "{PROJECT_UUID}"
});

function UserProfile() {
  const queryClient = useQueryClient();
  
  // This will suspend while loading
  const { data: user } = useSuspenseQuery({
    queryKey: ['user'],
    queryFn: fetchUser
  });

  const signOutMutation = useMutation({
    mutationFn: signOut,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['user'] });
    }
  });

  return (
    <div>
      <p>Welcome {user.email}</p>
      <button onClick={() => signOutMutation.mutate()}>Sign Out</button>
    </div>
  );
}

// Wrap with Suspense boundary
function App() {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <UserProfile />
    </Suspense>
  );
}
```

### Legacy Provider Usage (Deprecated)

The provider-based approach is still available but deprecated:

```tsx
import { OpenSecretProvider, useOpenSecret } from "@opensecret/react";

function App() {
  return (
    <OpenSecretProvider 
      apiUrl="{URL}"
      clientId="{PROJECT_UUID}"
    >
      <MyApp />
    </OpenSecretProvider>
  );
}

function MyApp() {
  const os = useOpenSecret();
  
  return (
    <div>
      <button onClick={() => os.signIn("email", "password")}>Sign In</button>
    </div>
  );
}
```

## API Reference

### Configuration

#### `configure(options)`

Configures the OpenSecret SDK with your API URL and client ID. Must be called before using any other SDK functions.

```typescript
configure({
  apiUrl: string,    // The URL of your OpenSecret backend
  clientId: string   // A UUID that identifies your project/tenant
})
```

Example:
```typescript
import { configure } from '@opensecret/react';

configure({
  apiUrl: 'https://api.opensecret.cloud',
  clientId: '550e8400-e29b-41d4-a716-446655440000'
});
```

### Direct API Functions

All functions can be imported directly from the package:

#### Authentication Methods
```typescript
import { signIn, signUp, signInGuest, signUpGuest, convertGuestToUserAccount, signOut } from '@opensecret/react';

// Sign in with email/password
await signIn(email: string, password: string);

// Sign up new user
await signUp(email: string, password: string, inviteCode: string, name?: string);

// Guest authentication
await signInGuest(id: string, password: string);
const { id, access_token, refresh_token } = await signUpGuest(password: string, inviteCode: string);

// Convert guest to full account
await convertGuestToUserAccount(email: string, password: string, name?: string);

// Sign out
await signOut();
```

#### Key-Value Storage Methods
```typescript
import { get, put, list, del } from '@opensecret/react';

// Get a value
const value = await get(key: string);

// Store a value  
await put(key: string, value: string);

// List all key-value pairs
const items = await list();

// Delete a value
await del(key: string);
```

#### User Management
```typescript
import { fetchUser, changePassword, generateThirdPartyToken } from '@opensecret/react';

// Get current user
const user = await fetchUser();

// Change password
await changePassword(currentPassword: string, newPassword: string);

// Generate third-party JWT token
const { token } = await generateThirdPartyToken(audience?: string);
```

#### Cryptographic Methods

##### Key Derivation Options

For cryptographic operations, the SDK supports a `KeyOptions` object with the following structure:

```typescript
type KeyOptions = {
  /** 
   * BIP-85 derivation path to derive a child mnemonic
   * Example: "m/83696968'/39'/0'/12'/0'"
   */
  seed_phrase_derivation_path?: string;
  
  /**
   * BIP-32 derivation path to derive a child key from the master (or BIP-85 derived) seed
   * Example: "m/44'/0'/0'/0/0"
   */
  private_key_derivation_path?: string;
};
```

All cryptographic methods accept this `KeyOptions` object as a parameter to specify derivation options.

##### Methods

- `getPrivateKey(key_options?: KeyOptions): Promise<{ mnemonic: string }>`: Retrieves the user's private key mnemonic phrase.
  - If no key_options are provided, returns the master mnemonic
  - If `seed_phrase_derivation_path` is provided, returns a BIP-85 derived child mnemonic
  - For BIP-85, the path format is typically `m/83696968'/39'/0'/12'/0'` where:
    - `83696968'` is the hardened BIP-85 application number (ASCII for "BIPS")
    - `39'` is the hardened BIP-39 application (for mnemonic derivation)
    - `0'` is the hardened coin type (0' for Bitcoin)
    - `12'` is the hardened entropy in words (12-word mnemonic)
    - `0'` is the hardened index (can be incremented to generate different phrases)

- `getPrivateKeyBytes(key_options?: KeyOptions): Promise<{ private_key: string }>`: Retrieves the private key bytes with flexible derivation options.
  - Supports multiple derivation approaches:
  
  1. Master key only (no parameters)
     - Returns the master private key bytes
  
  2. BIP-32 derivation only
     - Uses path format like `m/44'/0'/0'/0/0`
     - Supports both absolute (starting with "m/") and relative paths
     - Supports hardened derivation using either ' or h notation
  
  3. BIP-85 derivation only
     - Derives a child mnemonic from the master seed using BIP-85
     - Then returns the master private key of that derived seed
  
  4. Combined BIP-85 and BIP-32 derivation
     - First derives a child mnemonic via BIP-85
     - Then applies BIP-32 derivation to that derived seed

  Common BIP-32 paths:
  - BIP44 (Legacy): `m/44'/0'/0'/0/0`
  - BIP49 (SegWit): `m/49'/0'/0'/0/0`
  - BIP84 (Native SegWit): `m/84'/0'/0'/0/0`
  - BIP86 (Taproot): `m/86'/0'/0'/0/0`

- `getPublicKey(algorithm: 'schnorr' | 'ecdsa', key_options?: KeyOptions): Promise<PublicKeyResponse>`: Retrieves the user's public key for the specified signing algorithm and derivation options.
  
  The derivation paths determine which key is used to generate the public key:
  - Master key (no parameters)
  - BIP-32 derived key
  - BIP-85 derived key
  - Combined BIP-85 + BIP-32 derived key
  
  Supports two algorithms:
  - `'schnorr'`: For Schnorr signatures
  - `'ecdsa'`: For ECDSA signatures

- `signMessage(messageBytes: Uint8Array, algorithm: 'schnorr' | 'ecdsa', key_options?: KeyOptions): Promise<SignatureResponse>`: Signs a message using the specified algorithm and derivation options.
  
  Example message preparation:
  ```typescript
  // From string
  const messageBytes = new TextEncoder().encode("Hello, World!");
  
  // From hex
  const messageBytes = new Uint8Array(Buffer.from("deadbeef", "hex"));
  ```

- `encryptData(data: string, key_options?: KeyOptions): Promise<{ encrypted_data: string }>`: Encrypts arbitrary string data using the user's private key with flexible derivation options.
  
  Examples:
  ```typescript
  // Encrypt with master key
  const { encrypted_data } = await os.encryptData("Secret message");
  
  // Encrypt with BIP-32 derived key
  const { encrypted_data } = await os.encryptData("Secret message", {
    private_key_derivation_path: "m/44'/0'/0'/0/0"
  });
  
  // Encrypt with BIP-85 derived key
  const { encrypted_data } = await os.encryptData("Secret message", {
    seed_phrase_derivation_path: "m/83696968'/39'/0'/12'/0'"
  });
  
  // Encrypt with combined BIP-85 and BIP-32 derivation
  const { encrypted_data } = await os.encryptData("Secret message", {
    seed_phrase_derivation_path: "m/83696968'/39'/0'/12'/0'",
    private_key_derivation_path: "m/44'/0'/0'/0/0"
  });
  ```

- `decryptData(encryptedData: string, key_options?: KeyOptions): Promise<string>`: Decrypts data that was previously encrypted with the user's key.
  
  IMPORTANT: You must use the exact same derivation options for decryption that were used for encryption.
  
  Examples:
  ```typescript
  // Decrypt with master key
  const decrypted = await os.decryptData(encrypted_data);
  
  // Decrypt with BIP-32 derived key
  const decrypted = await os.decryptData(encrypted_data, {
    private_key_derivation_path: "m/44'/0'/0'/0/0"
  });
  
  // Decrypt with BIP-85 derived key
  const decrypted = await os.decryptData(encrypted_data, {
    seed_phrase_derivation_path: "m/83696968'/39'/0'/12'/0'"
  });
  
  // Decrypt with combined BIP-85 and BIP-32 derivation
  const decrypted = await os.decryptData(encrypted_data, {
    seed_phrase_derivation_path: "m/83696968'/39'/0'/12'/0'",
    private_key_derivation_path: "m/44'/0'/0'/0/0"
  });
  ```

##### Implementation Examples

1. Basic Usage with Default Master Key

```typescript
// Get the master mnemonic
const { mnemonic } = await os.getPrivateKey();

// Get the master private key bytes
const { private_key } = await os.getPrivateKeyBytes();

// Sign with the master key
const signature = await os.signMessage(messageBytes, 'ecdsa');
```

2. Using BIP-32 Derivation Only

```typescript
// Get private key bytes using BIP-32 derivation
const { private_key } = await os.getPrivateKeyBytes({
  private_key_derivation_path: "m/44'/0'/0'/0/0"
});

// Sign with a derived key
const signature = await os.signMessage(messageBytes, 'ecdsa', {
  private_key_derivation_path: "m/44'/0'/0'/0/0"
});
```

3. Using BIP-85 Derivation Only

```typescript
// Get a child mnemonic phrase derived via BIP-85
const { mnemonic } = await os.getPrivateKey({
  seed_phrase_derivation_path: "m/83696968'/39'/0'/12'/0'"
});

// Get master private key of a BIP-85 derived seed
const { private_key } = await os.getPrivateKeyBytes({
  seed_phrase_derivation_path: "m/83696968'/39'/0'/12'/0'"
});
```

4. Using Both BIP-85 and BIP-32 Derivation

```typescript
// Get private key bytes derived through BIP-85 and then BIP-32
const { private_key } = await os.getPrivateKeyBytes({
  seed_phrase_derivation_path: "m/83696968'/39'/0'/12'/0'",
  private_key_derivation_path: "m/44'/0'/0'/0/0"
});

// Sign a message with a key derived through both methods
const signature = await os.signMessage(messageBytes, 'schnorr', {
  seed_phrase_derivation_path: "m/83696968'/39'/0'/12'/0'",
  private_key_derivation_path: "m/44'/0'/0'/0/0"
});
```

5. Encryption/Decryption with Derived Keys

```typescript
// Encrypt with a BIP-85 derived key
const { encrypted_data } = await os.encryptData("Secret message", {
  seed_phrase_derivation_path: "m/83696968'/39'/0'/12'/0'"
});

// Decrypt using the same derivation path
const decrypted = await os.decryptData(encrypted_data, {
  seed_phrase_derivation_path: "m/83696968'/39'/0'/12'/0'"
});
```

### AI Integration

To get encrypted-to-the-gpu AI chat, use the `createAiCustomFetch` function to create a special version of `fetch` that handles all the encryption. Because we require the user to be logged in, and do the encryption client-side, this is safe to call from the client.

The easiest way to use this is through the OpenAI client:

```bash
npm install openai
```

```typescript
import OpenAI from "openai";
import { configure, createAiCustomFetch, getConfig } from "@opensecret/react";

// Configure the SDK
configure({
  apiUrl: "https://api.opensecret.cloud",
  clientId: "your-project-uuid"
});

// Create the OpenAI client with encrypted fetch
const openai = new OpenAI({
  baseURL: `${getConfig().apiUrl}/v1/`,
  dangerouslyAllowBrowser: true,
  apiKey: "api-key-doesnt-matter", // The actual API key is handled by OpenSecret
  defaultHeaders: {
    "Accept-Encoding": "identity",
    "Content-Type": "application/json",
  },
  fetch: createAiCustomFetch(), // Use OpenSecret's encrypted fetch
});

// Use the OpenAI client as normal
const completion = await openai.chat.completions.create({
  model: "gpt-4",
  messages: [{ role: "user", content: "Hello!" }],
  stream: true
});
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
bun test --test-name-pattern="Developer login and token storage" src/lib/test/integration/developer.test.ts --env-file .env.local
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

### Documentation Development

The SDK documentation is built using [Docusaurus](https://docusaurus.io/), a modern documentation framework. The documentation is automatically generated from TypeScript code comments and supplemented with manually written guides.

#### Getting Started with Documentation

To start the documentation development server:

```bash
bun run docs:dev
```

This will start the Docusaurus development server and open the documentation in your browser at http://localhost:3000/. The server supports hot-reloading, so any changes you make to the documentation will be immediately reflected in the browser.

#### Building Documentation

To build the documentation for production:

```bash
bun run docs:build
```

This will generate static HTML, JavaScript, and CSS files in the `website/build` directory.

To serve the built documentation locally:

```bash
bun run docs:serve
```

#### Documentation Structure

The documentation is organized into the following directories:

- `/website/docs/` - Contains all manual documentation files
  - `index.md` - The documentation landing page
  - `/guides/` - Step-by-step guides for using the SDK
  - `/api/` - API reference documentation (mostly auto-generated)

#### API Reference Documentation

The API reference documentation is automatically generated from TypeScript code comments using [TypeDoc](https://typedoc.org/). To update the API documentation:

1. Write proper JSDoc comments in the TypeScript source code
2. Run `bun run docs:build` to regenerate the documentation

Important notes for API documentation:

- Use standard JSDoc syntax for documenting parameters, return types, and descriptions
- For Markdown in JSDoc comments, be aware that backticks (`) must be properly escaped
- For code examples with apostrophes (e.g., BIP paths like `m/44'/0'/0'/0/0`), use backslash escaping: `m/44\'/0\'/0\'/0/0`

#### Adding New Guides

To add a new guide:

1. Create a new Markdown file in the `/website/docs/guides/` directory
2. Add frontmatter at the top of the file:
   ```md
   ---
   title: Your Guide Title
   sidebar_position: X  # Controls the order in the sidebar
   ---
   ```
3. Update the sidebar configuration in `/website/sidebars.ts` if needed

#### Customizing the Documentation

The main configuration files for Docusaurus are:

- `/website/docusaurus.config.ts` - Main Docusaurus configuration
- `/website/sidebars.ts` - Sidebar configuration
- `/website/typedoc.json` - TypeDoc configuration for API docs

To customize the appearance:

- Edit `/website/src/css/custom.css` for global styles
- Create or modify components in `/website/src/components/`

#### Deployment

The documentation can be deployed to various platforms like GitHub Pages, Netlify, or Vercel. For CloudFlare Pages deployment, as mentioned in our guideline:

1. In CloudFlare Pages, create a new project connected to your GitHub repo
2. Use these build settings:
   - Build command: `cd website && bun run build`
   - Build output directory: `website/build`
3. Set up a custom domain through CloudFlare's dashboard

#### Troubleshooting

Common issues:

- If TypeDoc fails to generate documentation, check the JSDoc comments for syntax errors
- If you see "Could not parse expression with acorn" errors, there are likely unescaped characters in code examples
- If links are broken, check that the referenced pages exist and paths are correct
- For sidebar issues, verify that the sidebar configuration in `sidebars.ts` is correct

## License

This project is licensed under the MIT License.

