[**@opensecret/react**](../README.md)

***

# ~~Function: getConversation()~~

> **getConversation**(`conversationId`): `Promise`\<[`Conversation`](../type-aliases/Conversation.md)\>

## Parameters

### conversationId

`string`

The UUID of the conversation to retrieve

## Returns

`Promise`\<[`Conversation`](../type-aliases/Conversation.md)\>

A promise resolving to the conversation

## Deprecated

Use openai.conversations.retrieve() instead
Retrieves a conversation by ID

## Throws

If:
- The user is not authenticated
- The conversation is not found
- The user doesn't have access to the conversation

## Example

```typescript
const conversation = await getConversation("550e8400-e29b-41d4-a716-446655440000");
console.log(conversation.metadata);
```
