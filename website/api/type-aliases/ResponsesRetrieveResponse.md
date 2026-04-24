[**@opensecret/react**](../README.md)

***

# Type Alias: ResponsesRetrieveResponse

> **ResponsesRetrieveResponse** = `object`

## Properties

### created\_at

> **created\_at**: `number`

***

### id

> **id**: `string`

***

### model

> **model**: `string`

***

### object

> **object**: `"response"`

***

### output?

> `optional` **output?**: `string` \| `unknown`[]

***

### status

> **status**: `"queued"` \| `"in_progress"` \| `"completed"` \| `"failed"` \| `"cancelled"`

***

### usage?

> `optional` **usage?**: `object`

#### input\_tokens

> **input\_tokens**: `number`

#### input\_tokens\_details

> **input\_tokens\_details**: `object`

##### input\_tokens\_details.cached\_tokens

> **cached\_tokens**: `number`

#### output\_tokens

> **output\_tokens**: `number`

#### output\_tokens\_details

> **output\_tokens\_details**: `object`

##### output\_tokens\_details.reasoning\_tokens

> **reasoning\_tokens**: `number`

#### total\_tokens

> **total\_tokens**: `number`
