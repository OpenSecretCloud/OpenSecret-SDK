# Function: OpenSecretDeveloper()

> **OpenSecretDeveloper**(`props`): `Element`

Provider component for OpenSecret developer operations.
This provider is used for managing organizations, projects, and developer access.

## Parameters

### props

Configuration properties for the OpenSecret developer provider

#### apiUrl

`string`

URL of OpenSecret developer API

#### children

`ReactNode`

React child components to be wrapped by the provider

#### pcrConfig?

[`PcrConfig`](../type-aliases/PcrConfig.md) = `{}`

## Returns

`Element`

## Example

```tsx
<OpenSecretDeveloper
  apiUrl='https://developer.opensecret.cloud'
>
  <App />
</OpenSecretDeveloper>
```
