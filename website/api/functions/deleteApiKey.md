[**@opensecret/react**](../README.md)

***

# Function: deleteApiKey()

> **deleteApiKey**(`name`): `Promise`\<`void`\>

Deletes an API key by its name

## Parameters

### name

`string`

The name of the API key to delete

## Returns

`Promise`\<`void`\>

A promise resolving to void

## Throws

If:
- The user is not authenticated
- The API key with this name is not found
- The user doesn't own the API key
- The request fails

## Description

Permanently deletes an API key. This action cannot be undone.
Any requests using the deleted key will immediately fail with 401 Unauthorized.
Names are unique per user, so this uniquely identifies the key to delete.

Example usage:
```typescript
await deleteApiKey("Production Key");
console.log("API key deleted successfully");
```
