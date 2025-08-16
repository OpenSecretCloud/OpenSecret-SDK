# OpenSecret Rust SDK

Rust SDK for OpenSecret - secure AI API interactions with nitro attestation.

## Features

- 🔐 **Nitro Attestation**: Verify server identity through AWS Nitro Enclaves
- 🔑 **End-to-End Encryption**: All API calls encrypted with session keys
- 👤 **Authentication**: Support for both email-based and guest users
- 🔄 **Token Management**: Automatic token refresh and session management
- 🛡️ **Secure by Default**: No plaintext data transmission

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
opensecret-sdk = "0.1.0"
```

## Quick Start

```rust
use opensecret_sdk::{OpenSecretClient, Result};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize client
    let client = OpenSecretClient::new("https://api.opensecret.com")?;
    let client_id = Uuid::parse_str("your-client-id")?;

    // Establish secure session
    client.perform_attestation_handshake().await?;

    // Register and login
    let response = client.register(
        "user@example.com".to_string(),
        "password".to_string(),
        client_id,
        Some("John Doe".to_string())
    ).await?;

    println!("Logged in as: {}", response.id);
    Ok(())
}
```

## Authentication

### User Registration

Register with email:
```rust
let response = client.register(
    email,
    password,
    client_id,
    Some(name)  // Optional
).await?;
```

Register as guest (no email):
```rust
let response = client.register_guest(
    password,
    client_id
).await?;
```

### Login

Login with email:
```rust
let response = client.login(
    email,
    password,
    client_id
).await?;
```

Login with user ID (guests only):
```rust
let response = client.login_with_id(
    user_id,
    password,
    client_id
).await?;
```

### Token Management

Tokens are automatically stored after login/registration. You can:

```rust
// Get current tokens
let access_token = client.get_access_token()?;
let refresh_token = client.get_refresh_token()?;

// Refresh tokens
client.refresh_token().await?;

// Logout (clears session and tokens)
client.logout().await?;
```

## Session Management

Every API call requires an encrypted session:

1. **Attestation Handshake**: Establishes trust and exchanges encryption keys
2. **Encrypted Communication**: All subsequent calls use the session key
3. **Token Authentication**: Protected endpoints require valid access tokens

```rust
// Required before any API calls
client.perform_attestation_handshake().await?;

// Check session status
if let Some(session_id) = client.get_session_id()? {
    println!("Active session: {}", session_id);
}
```

## Error Handling

The SDK uses a custom `Error` type with detailed error variants:

```rust
use opensecret_sdk::Error;

match client.login(email, password, client_id).await {
    Ok(response) => println!("Success!"),
    Err(Error::Authentication(msg)) => println!("Auth failed: {}", msg),
    Err(Error::Api { status, message }) => println!("API error {}: {}", status, message),
    Err(e) => println!("Other error: {}", e),
}
```

## Testing

The SDK reads configuration from `.env.local` in the parent directory (OpenSecret-SDK root), matching the TypeScript SDK setup.

Required environment variables in `.env.local`:
```bash
VITE_OPEN_SECRET_API_URL=http://localhost:3000
VITE_TEST_CLIENT_ID=your-client-id-uuid
```

Run tests:
```bash
# All tests (requires running server on localhost:3000)
cargo test

# With output
cargo test -- --nocapture

# Specific test
cargo test test_login_signup_flow -- --nocapture
```

## Examples

See the `examples/` directory for complete examples:

```bash
# Basic authentication flow
cargo run --example auth_example
```

## Security Considerations

1. **Always verify attestation** in production environments
2. **Store tokens securely** - the SDK keeps them in memory only
3. **Use HTTPS** for all production API calls
4. **Rotate tokens regularly** using the refresh mechanism
5. **Clear sessions** after use with `logout()`

## License

MIT