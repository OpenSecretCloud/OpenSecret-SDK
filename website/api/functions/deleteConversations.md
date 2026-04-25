[**@opensecret/react**](../README.md)

***

# Function: deleteConversations()

> **deleteConversations**(): `Promise`\<[`ConversationsDeleteResponse`](../type-aliases/ConversationsDeleteResponse.md)\>

Deletes all conversations

## Returns

`Promise`\<[`ConversationsDeleteResponse`](../type-aliases/ConversationsDeleteResponse.md)\>

A promise resolving to deletion confirmation

## Throws

If:
- The user is not authenticated
- The request fails

## Description

This function permanently deletes all conversations and their associated items.
This action cannot be undone.

## Example

```typescript
const result = await deleteConversations();
if (result.deleted) {
  console.log("All conversations deleted successfully");
}
```
