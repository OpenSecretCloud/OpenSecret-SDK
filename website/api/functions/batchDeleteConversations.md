[**@opensecret/react**](../README.md)

***

# Function: batchDeleteConversations()

> **batchDeleteConversations**(`ids`): `Promise`\<[`BatchDeleteConversationsResponse`](../type-aliases/BatchDeleteConversationsResponse.md)\>

Batch deletes multiple conversations by their IDs

## Parameters

### ids

`string`[]

Array of conversation UUIDs to delete

## Returns

`Promise`\<[`BatchDeleteConversationsResponse`](../type-aliases/BatchDeleteConversationsResponse.md)\>

A promise resolving to per-item deletion results

## Throws

If:
- The user is not authenticated
- The request fails

## Description

This function deletes multiple conversations in a single request.
Returns per-item results so callers can handle partial failures.

## Example

```typescript
const result = await batchDeleteConversations([
  "550e8400-e29b-41d4-a716-446655440000",
  "550e8400-e29b-41d4-a716-446655440001"
]);
for (const item of result.data) {
  if (item.deleted) {
    console.log(`Conversation ${item.id} deleted`);
  } else {
    console.log(`Failed to delete ${item.id}: ${item.error}`);
  }
}
```
