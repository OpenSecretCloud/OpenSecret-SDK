# Type Alias: ParsedAttestationView

> **ParsedAttestationView** = `object`

## Properties

### cert0hash

> **cert0hash**: `string`

***

### certificates

> **certificates**: `object`[]

#### isRoot

> **isRoot**: `boolean`

#### notAfter

> **notAfter**: `string`

#### notBefore

> **notBefore**: `string`

#### pem

> **pem**: `string`

#### subject

> **subject**: `string`

***

### digest

> **digest**: `string`

***

### moduleId

> **moduleId**: `string`

***

### nonce

> **nonce**: `string` \| `null`

***

### pcrs

> **pcrs**: `object`[]

#### id

> **id**: `number`

#### value

> **value**: `string`

***

### publicKey

> **publicKey**: `string` \| `null`

***

### timestamp

> **timestamp**: `string`

***

### userData

> **userData**: `string` \| `null`

***

### validatedPcr0Hash

> **validatedPcr0Hash**: [`Pcr0ValidationResult`](Pcr0ValidationResult.md) \| `null`
