[**@opensecret/react**](../README.md)

***

# Function: createApiKey()

> **createApiKey**(`name`): `Promise`\<[`ApiKeyCreateResponse`](../type-aliases/ApiKeyCreateResponse.md)\>

Creates a new API key for the authenticated user

## Parameters

### name

`string`

A descriptive name for the API key

## Returns

`Promise`\<[`ApiKeyCreateResponse`](../type-aliases/ApiKeyCreateResponse.md)\>

A promise resolving to the API key details with the key value (only shown once)

## Throws

If:
- The user is not authenticated
- The name is invalid
- The request fails

## Description

IMPORTANT: The `key` field is only returned once during creation and cannot be retrieved again.
The SDK consumer should prompt users to save the key immediately.

Example usage:
```typescript
const apiKey = await createApiKey("Production Key");
console.log(apiKey.key); // UUID format: 550e8400-e29b-41d4-a716-446655440000
// Save this key securely - it won't be shown again!
```
