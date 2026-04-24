[**@opensecret/react**](../README.md)

***

# ~~Function: createConversation()~~

> **createConversation**(`metadata?`, `options?`): `Promise`\<[`Conversation`](../type-aliases/Conversation.md)\>

Creates a new conversation

## Parameters

### metadata?

`Record`\<`string`, `unknown`\>

Optional metadata to attach to the conversation

### options?

[`ConversationCreateOptions`](../type-aliases/ConversationCreateOptions.md)

## Returns

`Promise`\<[`Conversation`](../type-aliases/Conversation.md)\>

A promise resolving to the created conversation

## Throws

If:
- The user is not authenticated
- The request fails

## Description

This function creates a new conversation that can be used to group
related responses together. The conversation can have metadata
attached for organization and filtering purposes.

NOTE: Prefer using the OpenAI client directly for conversation operations:
```typescript
const openai = new OpenAI({ fetch: customFetch });
const conversation = await openai.conversations.create({
  metadata: { title: "Product Support", category: "technical" }
});
```

## Deprecated

Use openai.conversations.create() instead
