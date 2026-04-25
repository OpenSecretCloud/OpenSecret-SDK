[**@opensecret/react**](../README.md)

***

# ~~Function: updateConversation()~~

> **updateConversation**(`conversationId`, `metadata?`, `options?`): `Promise`\<[`Conversation`](../type-aliases/Conversation.md)\>

## Parameters

### conversationId

`string`

The UUID of the conversation to update

### metadata?

`Record`\<`string`, `unknown`\>

The metadata to update

### options?

[`ConversationUpdateOptions`](../type-aliases/ConversationUpdateOptions.md)

## Returns

`Promise`\<[`Conversation`](../type-aliases/Conversation.md)\>

A promise resolving to the updated conversation

## Deprecated

Use openai.conversations.update() instead
Updates a conversation's metadata

## Throws

If:
- The user is not authenticated
- The conversation is not found
- The user doesn't have access to the conversation

## Example

```typescript
const updated = await updateConversation("550e8400-e29b-41d4-a716-446655440000", {
  metadata: { title: "Updated Title", status: "resolved" }
});
```
