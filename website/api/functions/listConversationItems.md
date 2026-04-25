[**@opensecret/react**](../README.md)

***

# ~~Function: listConversationItems()~~

> **listConversationItems**(`conversationId`, `params?`): `Promise`\<[`ConversationItemsResponse`](../type-aliases/ConversationItemsResponse.md)\>

## Parameters

### conversationId

`string`

The UUID of the conversation

### params?

Optional pagination parameters

#### after?

`string`

#### before?

`string`

#### limit?

`number`

## Returns

`Promise`\<[`ConversationItemsResponse`](../type-aliases/ConversationItemsResponse.md)\>

A promise resolving to a paginated list of conversation items

## Deprecated

Use openai.conversations.items.list() instead
Lists items in a conversation

## Throws

If:
- The user is not authenticated
- The conversation is not found
- The user doesn't have access to the conversation

## Example

```typescript
const items = await listConversationItems("550e8400-e29b-41d4-a716-446655440000", {
  limit: 20
});
for (const item of items.data) {
  console.log(item.role, item.content);
}
```
