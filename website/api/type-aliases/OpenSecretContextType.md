# Type Alias: OpenSecretContextType

> **OpenSecretContextType** = `object`

## Properties

### aiCustomFetch()

> **aiCustomFetch**: (`url`, `init?`) => `Promise`\<`Response`\>

Custom fetch function for AI requests that handles encryption
and token refreshing.

Meant to be used with the OpenAI JS library

Example:
```tsx
const openai = new OpenAI({
  baseURL: `${os.apiUrl}/v1/`,
  dangerouslyAllowBrowser: true,
  apiKey: "the-api-key-doesnt-matter",
  defaultHeaders: {
    "Accept-Encoding": "identity"
  },
  fetch: os.aiCustomFetch
});
```

#### Parameters

##### url

`RequestInfo`

##### init?

`RequestInit`

#### Returns

`Promise`\<`Response`\>

***

### apiUrl

> **apiUrl**: `string`

Returns the current OpenSecret enclave API URL being used

#### Returns

The current API URL

***

### auth

> **auth**: [`OpenSecretAuthState`](OpenSecretAuthState.md)

***

### authenticate

> **authenticate**: *typeof* `authenticate`

Authenticates an attestation document

***

### awsRootCertDer

> **awsRootCertDer**: *typeof* `AWS_ROOT_CERT_DER`

AWS root certificate in DER format

***

### changePassword

> **changePassword**: *typeof* `api.changePassword`

***

### clientId

> **clientId**: `string`

The client ID for this project/tenant.
A UUID that identifies which project/tenant this instance belongs to.

***

### confirmAccountDeletion()

> **confirmAccountDeletion**: (`uuid`, `plaintextSecret`) => `Promise`\<`void`\>

Confirms and completes the account deletion process

#### Parameters

##### uuid

`string`

The UUID from the verification email

##### plaintextSecret

`string`

The plaintext secret that was hashed in the request step

#### Returns

`Promise`\<`void`\>

A promise resolving to void

#### Throws

If confirmation fails

This function:
1. Requires the user to be logged in (uses authenticatedApiCall)
2. Verifies both the UUID from email and the secret known only to the client
3. Permanently deletes the user account and all associated data
4. After successful deletion, the client should clear all local storage and tokens

***

### confirmPasswordReset()

> **confirmPasswordReset**: (`email`, `alphanumericCode`, `plaintextSecret`, `newPassword`) => `Promise`\<`void`\>

#### Parameters

##### email

`string`

##### alphanumericCode

`string`

##### plaintextSecret

`string`

##### newPassword

`string`

#### Returns

`Promise`\<`void`\>

***

### convertGuestToUserAccount()

> **convertGuestToUserAccount**: (`email`, `password`, `name?`) => `Promise`\<`void`\>

Upgrades a guest account to a user account with email and password authentication.

#### Parameters

##### email

`string`

User's email address

##### password

`string`

User's chosen password

##### name?

Optional user's full name

`string` | `null`

#### Returns

`Promise`\<`void`\>

A promise that resolves when account creation is complete

#### Throws

If:
- The current user is not a guest account
- The email address is already in use
- The user is not authenticated

- Upgrades the currently signed-in guest account (identified by their UUID) to a full email account
- Requires the user to be currently authenticated as a guest
- Updates the auth state with new user information
- Preserves all existing data associated with the guest account

***

### decryptData

> **decryptData**: *typeof* `api.decryptData`

Decrypts data that was previously encrypted with the user's key

#### Param

Base64-encoded encrypted data string

#### Param

Optional key derivation options or legacy BIP32 derivation path string

#### Returns

A promise resolving to the decrypted string

#### Throws

If:
- The encrypted data is malformed
- The derivation paths are invalid
- Authentication fails
- Server-side decryption error occurs

This function supports multiple decryption approaches:

1. Decrypt with master key (no derivation parameters)

2. Decrypt with BIP-32 derived key
   - Derives a child key from the master seed using BIP-32

3. Decrypt with BIP-85 derived key
   - Derives a child mnemonic using BIP-85, then uses its master key

4. Decrypt with combined BIP-85 and BIP-32 derivation
   - First derives a child mnemonic via BIP-85
   - Then applies BIP-32 derivation to derive a key from that seed

IMPORTANT: You must use the exact same derivation options for decryption
that were used for encryption.

***

### del

> **del**: *typeof* `api.fetchDelete`

Deletes a key-value pair from the user's storage

#### Param

The unique identifier for the value to be deleted

#### Returns

A promise resolving when the deletion is complete

#### Throws

If the key cannot be deleted

- Calls the authenticated API endpoint to remove a specific key
- Requires an active authentication session
- Throws an error if the deletion fails (including for non-existent keys)
- Propagates any server-side errors directly

***

### encryptData

> **encryptData**: *typeof* `api.encryptData`

Encrypts arbitrary string data using the user's private key

#### Param

String content to be encrypted

#### Param

Optional key derivation options or legacy BIP32 derivation path string

#### Returns

A promise resolving to the encrypted data response

#### Throws

If:
- The derivation paths are invalid
- Authentication fails
- Server-side encryption error occurs

This function supports multiple encryption approaches:

1. Encrypt with master key (no derivation parameters)

2. Encrypt with BIP-32 derived key
   - Derives a child key from the master seed using BIP-32
   - Example: "m/44\'/0\'/0\'/0/0"

3. Encrypt with BIP-85 derived key
   - Derives a child mnemonic using BIP-85, then uses its master key
   - Example: { seed_phrase_derivation_path: "m/83696968\'/39\'/0\'/12\'/0\'" }

4. Encrypt with combined BIP-85 and BIP-32 derivation
   - First derives a child mnemonic via BIP-85
   - Then applies BIP-32 derivation to derive a key from that seed
   - Example: {
       seed_phrase_derivation_path: "m/83696968\'/39\'/0\'/12\'/0\'",
       private_key_derivation_path: "m/44\'/0\'/0\'/0/0"
     }

Technical details:
- Encrypts data with AES-256-GCM
- A random nonce is generated for each encryption operation (included in the result)
- The encrypted_data format includes the nonce and is base64-encoded

***

### expectedRootCertHash

> **expectedRootCertHash**: *typeof* `EXPECTED_ROOT_CERT_HASH`

Expected hash of the AWS root certificate

***

### generateThirdPartyToken()

> **generateThirdPartyToken**: (`audience?`) => `Promise`\<`ThirdPartyTokenResponse`\>

Generates a JWT token for use with third-party services

#### Parameters

##### audience?

`string`

Optional URL of the service (e.g. "https://billing.opensecret.cloud")

#### Returns

`Promise`\<`ThirdPartyTokenResponse`\>

A promise resolving to the token response

#### Throws

If:
- The user is not authenticated
- The audience URL is invalid (if provided)

- Generates a signed JWT token for use with third-party services
- If audience is provided, it can be any valid URL
- If audience is omitted, a token with no audience restriction will be generated
- Requires an active authentication session
- Token can be used to authenticate with the specified service

***

### get

> **get**: *typeof* `api.fetchGet`

Retrieves a value from key-value storage

#### Param

The unique identifier for the stored value

#### Returns

A promise resolving to the stored value

#### Throws

If the key cannot be retrieved

- Calls the authenticated API endpoint to fetch a value
- Returns undefined if the key does not exist
- Requires an active authentication session
- Logs any retrieval errors

***

### getAttestation

> **getAttestation**: *typeof* `getAttestation`

Gets attestation from the enclave

***

### getAttestationDocument()

> **getAttestationDocument**: () => `Promise`\<[`ParsedAttestationView`](ParsedAttestationView.md)\>

Gets and verifies an attestation document from the enclave

#### Returns

`Promise`\<[`ParsedAttestationView`](ParsedAttestationView.md)\>

A promise resolving to the parsed attestation document

#### Throws

If attestation fails or is invalid

This is a convenience function that:
1. Fetches the attestation document with a random nonce
2. Authenticates the document
3. Parses it for viewing

***

### getPrivateKey

> **getPrivateKey**: *typeof* `api.fetchPrivateKey`

Retrieves the user's private key mnemonic phrase

#### Param

Optional key derivation options

#### Returns

A promise resolving to the private key response

#### Throws

If the private key cannot be retrieved

This function supports two modes:

1. Master mnemonic (no parameters)
   - Returns the user's master 12-word BIP39 mnemonic

2. BIP-85 derived mnemonic
   - Derives a child mnemonic using BIP-85
   - Requires seed_phrase_derivation_path in options
   - Example: "m/83696968'/39'/0'/12'/0'"

***

### getPrivateKeyBytes

> **getPrivateKeyBytes**: *typeof* `api.fetchPrivateKeyBytes`

Retrieves the private key bytes for the given derivation options

#### Param

Optional key derivation options or legacy BIP32 derivation path string

#### Returns

A promise resolving to the private key bytes response

#### Throws

If:
- The private key bytes cannot be retrieved
- The derivation paths are invalid

This function supports multiple derivation approaches:

1. Master key only (no parameters)
   - Returns the master private key bytes

2. BIP-32 derivation only
   - Uses a single derivation path to derive a child key from the master seed
   - Supports both absolute and relative paths with hardened derivation:
     - Absolute path: "m/44'/0'/0'/0/0"
     - Relative path: "0'/0'/0'/0/0"
     - Hardened notation: "44'" or "44h"
   - Common paths:
     - BIP44 (Legacy): m/44'/0'/0'/0/0
     - BIP49 (SegWit): m/49'/0'/0'/0/0
     - BIP84 (Native SegWit): m/84'/0'/0'/0/0
     - BIP86 (Taproot): m/86'/0'/0'/0/0

3. BIP-85 derivation only
   - Derives a child mnemonic from the master seed using BIP-85
   - Then returns the master private key of that derived seed
   - Example path: "m/83696968'/39'/0'/12'/0'"

4. Combined BIP-85 and BIP-32 derivation
   - First derives a child mnemonic via BIP-85
   - Then applies BIP-32 derivation to that derived seed

***

### getPublicKey

> **getPublicKey**: *typeof* `api.fetchPublicKey`

Retrieves the user's public key for the specified algorithm

#### Param

The signing algorithm ('schnorr' or 'ecdsa')

#### Param

Optional key derivation options or legacy BIP32 derivation path string

#### Returns

A promise resolving to the public key response

#### Throws

If the public key cannot be retrieved

The derivation paths determine which key is used to generate the public key:

1. Master key (no derivation parameters)
   - Returns the public key corresponding to the master private key

2. BIP-32 derived key
   - Returns the public key for a derived child key

3. BIP-85 derived key
   - Returns the public key for the master key of a BIP-85 derived seed

4. Combined BIP-85 and BIP-32 derivation
   - First derives a child mnemonic via BIP-85
   - Then applies BIP-32 derivation to get the corresponding public key

***

### handleAppleCallback()

> **handleAppleCallback**: (`code`, `state`, `inviteCode`) => `Promise`\<`void`\>

#### Parameters

##### code

`string`

##### state

`string`

##### inviteCode

`string`

#### Returns

`Promise`\<`void`\>

***

### handleAppleNativeSignIn()

> **handleAppleNativeSignIn**: (`appleUser`, `inviteCode?`) => `Promise`\<`void`\>

#### Parameters

##### appleUser

`api.AppleUser`

##### inviteCode?

`string`

#### Returns

`Promise`\<`void`\>

***

### handleGitHubCallback()

> **handleGitHubCallback**: (`code`, `state`, `inviteCode`) => `Promise`\<`void`\>

#### Parameters

##### code

`string`

##### state

`string`

##### inviteCode

`string`

#### Returns

`Promise`\<`void`\>

***

### handleGoogleCallback()

> **handleGoogleCallback**: (`code`, `state`, `inviteCode`) => `Promise`\<`void`\>

#### Parameters

##### code

`string`

##### state

`string`

##### inviteCode

`string`

#### Returns

`Promise`\<`void`\>

***

### initiateAppleAuth()

> **initiateAppleAuth**: (`inviteCode`) => `Promise`\<`api.AppleAuthResponse`\>

#### Parameters

##### inviteCode

`string`

#### Returns

`Promise`\<`api.AppleAuthResponse`\>

***

### initiateGitHubAuth()

> **initiateGitHubAuth**: (`inviteCode`) => `Promise`\<[`GithubAuthResponse`](GithubAuthResponse.md)\>

#### Parameters

##### inviteCode

`string`

#### Returns

`Promise`\<[`GithubAuthResponse`](GithubAuthResponse.md)\>

***

### initiateGoogleAuth()

> **initiateGoogleAuth**: (`inviteCode`) => `Promise`\<[`GoogleAuthResponse`](GoogleAuthResponse.md)\>

#### Parameters

##### inviteCode

`string`

#### Returns

`Promise`\<[`GoogleAuthResponse`](GoogleAuthResponse.md)\>

***

### list

> **list**: *typeof* `api.fetchList`

Retrieves all key-value pairs stored by the user

#### Returns

A promise resolving to an array of stored items

#### Throws

If the list cannot be retrieved

- Calls the authenticated API endpoint to fetch all stored items
- Returns an array of key-value pairs with metadata
- Requires an active authentication session
- Each item includes key, value, creation, and update timestamps
- Logs any listing errors

***

### parseAttestationForView()

> **parseAttestationForView**: (`document`, `cabundle`, `pcrConfig?`) => `Promise`\<[`ParsedAttestationView`](ParsedAttestationView.md)\>

Parses an attestation document for viewing

#### Parameters

##### document

[`AttestationDocument`](AttestationDocument.md)

##### cabundle

`Uint8Array`[]

##### pcrConfig?

[`PcrConfig`](PcrConfig.md)

#### Returns

`Promise`\<[`ParsedAttestationView`](ParsedAttestationView.md)\>

***

### pcrConfig

> **pcrConfig**: [`PcrConfig`](PcrConfig.md)

Additional PCR0 hashes to validate against

***

### put

> **put**: *typeof* `api.fetchPut`

Stores a key-value pair in the user's storage

#### Param

The unique identifier for the value

#### Param

The string value to be stored

#### Returns

A promise resolving to the server's response

#### Throws

If the value cannot be stored

- Calls the authenticated API endpoint to store a value
- Requires an active authentication session
- Overwrites any existing value for the given key
- Logs any storage errors

***

### refetchUser()

> **refetchUser**: () => `Promise`\<`void`\>

#### Returns

`Promise`\<`void`\>

***

### refreshAccessToken

> **refreshAccessToken**: *typeof* `api.refreshToken`

***

### requestAccountDeletion()

> **requestAccountDeletion**: (`hashedSecret`) => `Promise`\<`void`\>

Initiates the account deletion process for logged-in users

#### Parameters

##### hashedSecret

`string`

Client-side hashed secret for verification

#### Returns

`Promise`\<`void`\>

A promise resolving to void

#### Throws

If request fails

This function:
1. Requires the user to be logged in (uses authenticatedApiCall)
2. Sends a verification email to the user's email address
3. The email contains a UUID that will be needed for confirmation
4. The client must store the plaintext secret for confirmation

***

### requestNewVerificationCode

> **requestNewVerificationCode**: *typeof* `api.requestNewVerificationCode`

***

### requestNewVerificationEmail

> **requestNewVerificationEmail**: *typeof* `api.requestNewVerificationCode`

***

### requestPasswordReset()

> **requestPasswordReset**: (`email`, `hashedSecret`) => `Promise`\<`void`\>

#### Parameters

##### email

`string`

##### hashedSecret

`string`

#### Returns

`Promise`\<`void`\>

***

### signIn()

> **signIn**: (`email`, `password`) => `Promise`\<`void`\>

Authenticates a user with email and password.

- Calls the login API endpoint with the configured clientId
- Stores access_token and refresh_token in localStorage
- Updates the auth state with user information
- Throws an error if authentication fails

#### Parameters

##### email

`string`

User's email address

##### password

`string`

User's password

#### Returns

`Promise`\<`void`\>

A promise that resolves when authentication is complete

#### Throws

If login fails

***

### signInGuest()

> **signInGuest**: (`id`, `password`) => `Promise`\<`void`\>

Authenticates a guest user with user id and password

#### Parameters

##### id

`string`

User's unique id

##### password

`string`

User's password

#### Returns

`Promise`\<`void`\>

A promise that resolves when authentication is complete

#### Throws

If login fails

- Calls the login API endpoint
- Stores access_token and refresh_token in localStorage
- Updates the auth state with user information
- Throws an error if authentication fails

***

### signMessage

> **signMessage**: *typeof* `api.signMessage`

Signs a message using the specified algorithm.
This function supports multiple signing approaches: master key (no derivation),
BIP-32 derived key, BIP-85 derived key, or combined BIP-85 and BIP-32 derivation.

#### Param

The message to sign as a Uint8Array

#### Param

The signing algorithm ('schnorr' or 'ecdsa')

#### Param

Optional key derivation options or legacy BIP32 derivation path string

#### Returns

A promise resolving to the signature response

#### Throws

If the message signing fails

***

### signOut()

> **signOut**: () => `Promise`\<`void`\>

Logs out the current user

#### Returns

`Promise`\<`void`\>

A promise that resolves when logout is complete

#### Throws

If logout fails

- Calls the logout API endpoint with the current refresh_token
- Removes access_token, refresh_token from localStorage
- Removes session-related items from sessionStorage
- Resets the auth state to show no user is authenticated

***

### signUp()

> **signUp**: (`email`, `password`, `inviteCode`, `name?`) => `Promise`\<`void`\>

Creates a new user account

#### Parameters

##### email

`string`

User's email address

##### password

`string`

User's chosen password

##### inviteCode

`string`

Invitation code for registration

##### name?

`string`

Optional user's full name

#### Returns

`Promise`\<`void`\>

A promise that resolves when account creation is complete

#### Throws

If signup fails

- Calls the registration API endpoint
- Stores access_token and refresh_token in localStorage
- Updates the auth state with new user information
- Throws an error if account creation fails

***

### signUpGuest()

> **signUpGuest**: (`password`, `inviteCode`) => `Promise`\<[`LoginResponse`](LoginResponse.md)\>

Creates a new guest account, which can be upgraded to a normal account later with email.

#### Parameters

##### password

`string`

User's chosen password, cannot be changed or recovered without adding email address.

##### inviteCode

`string`

Invitation code for registration

#### Returns

`Promise`\<[`LoginResponse`](LoginResponse.md)\>

A promise that resolves to the login response containing the guest ID

#### Throws

If signup fails

- Calls the registration API endpoint
- Stores access_token and refresh_token in localStorage
- Updates the auth state with new user information
- Throws an error if account creation fails

***

### verifyEmail

> **verifyEmail**: *typeof* `api.verifyEmail`
