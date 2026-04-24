[**@opensecret/react**](../README.md)

***

# ~~Function: deleteConversation()~~

> **deleteConversation**(`conversationId`): `Promise`\<[`ConversationDeleteResponse`](../type-aliases/ConversationDeleteResponse.md)\>

## Parameters

### conversationId

`string`

The UUID of the conversation to delete

## Returns

`Promise`\<[`ConversationDeleteResponse`](../type-aliases/ConversationDeleteResponse.md)\>

A promise resolving to deletion confirmation

## Deprecated

Use openai.conversations.delete() instead
Deletes a conversation permanently

## Throws

If:
- The user is not authenticated
- The conversation is not found
- The user doesn't have access to the conversation

## Description

This function permanently deletes a conversation and all associated items.
This action cannot be undone.

## Example

```typescript
const result = await deleteConversation("550e8400-e29b-41d4-a716-446655440000");
if (result.deleted) {
  console.log("Conversation deleted successfully");
}
```
