[**@opensecret/react**](../README.md)

***

# Function: listApiKeys()

> **listApiKeys**(): `Promise`\<\{ `keys`: [`ApiKeyListResponse`](../type-aliases/ApiKeyListResponse.md); \}\>

Lists all API keys for the authenticated user

## Returns

`Promise`\<\{ `keys`: [`ApiKeyListResponse`](../type-aliases/ApiKeyListResponse.md); \}\>

A promise resolving to an object containing an array of API key metadata (without the actual keys)

## Throws

If:
- The user is not authenticated
- The request fails

## Description

Returns metadata about all API keys associated with the user's account.
Note that the actual key values are never returned - they are only shown once during creation.
The keys are sorted by created_at in descending order (newest first).

Example usage:
```typescript
const response = await listApiKeys();
response.keys.forEach(key => {
  console.log(`${key.name} created at ${key.created_at}`);
});
```
