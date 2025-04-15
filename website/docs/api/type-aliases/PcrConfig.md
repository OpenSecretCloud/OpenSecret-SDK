[**@opensecret/react**](../README.md)

***

[@opensecret/react](../README.md) / PcrConfig

# Type Alias: PcrConfig

> **PcrConfig** = `object`

Configuration options for PCR validation

## Properties

### pcr0DevValues?

> `optional` **pcr0DevValues**: `string`[]

Additional custom PCR0 values for development environments

***

### pcr0Values?

> `optional` **pcr0Values**: `string`[]

Additional custom PCR0 values for production environments

***

### remoteAttestation?

> `optional` **remoteAttestation**: `boolean`

Enable/disable remote attestation (defaults to true)

***

### remoteAttestationUrls?

> `optional` **remoteAttestationUrls**: `object`

Custom URLs for remote attestation

#### dev?

> `optional` **dev**: `string`

URL for development PCR history

#### prod?

> `optional` **prod**: `string`

URL for production PCR history
