[**@opensecret/react**](../README.md)

***

# Function: listConversations()

> **listConversations**(`params?`): `Promise`\<[`ConversationsListResponse`](../type-aliases/ConversationsListResponse.md)\>

Lists all conversations with pagination (non-standard endpoint)

## Parameters

### params?

[`ConversationsListParams`](../type-aliases/ConversationsListParams.md)

Optional pagination parameters

## Returns

`Promise`\<[`ConversationsListResponse`](../type-aliases/ConversationsListResponse.md)\>

A promise resolving to a paginated list of conversations

## Throws

If:
- The user is not authenticated
- The request fails

## Description

This is a custom extension not part of the standard OpenAI Conversations API.
This function fetches a paginated list of the user's conversations.
Conversations are sorted by last_activity_at (most recent activity first).

## Example

```typescript
const conversations = await listConversations({ limit: 20 });
for (const conv of conversations.data) {
  console.log(conv.id, conv.metadata);
}
```
