---
title: Third-Party Tokens
sidebar_position: 9
---

# Third-Party Tokens

OpenSecret allows your application to generate secure JWT tokens for authenticated users, enabling integration with third-party services while maintaining security and identity. This guide explains how to generate and use third-party tokens.

## Overview

Third-party tokens provide:

- A secure way to authenticate users with external services
- JWT-based token format with standard claims
- Audience-specific tokens for targeted services
- User identity information controlled by the application

## Prerequisites

Before using third-party tokens, make sure:

1. Your application is wrapped with `OpenSecretProvider`
2. The user is authenticated
3. You have imported the `useOpenSecret` hook

## Generating Third-Party Tokens

### Basic Token Generation

To generate a token:

```tsx
import { useState } from "react";
import { useOpenSecret } from "@opensecret/react";

function TokenGenerator() {
  const os = useOpenSecret();
  const [audience, setAudience] = useState("https://api.yourservice.com");
  const [token, setToken] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  
  async function generateToken() {
    setLoading(true);
    setError("");
    try {
      const response = await os.generateThirdPartyToken(audience);
      setToken(response.token);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to generate token");
    } finally {
      setLoading(false);
    }
  }
  
  return (
    <div className="token-generator">
      <h3>Generate Third-Party Token</h3>
      
      <div className="input-group">
        <label htmlFor="audience">Audience URL:</label>
        <input
          id="audience"
          type="url"
          value={audience}
          onChange={(e) => setAudience(e.target.value)}
          placeholder="https://api.yourservice.com"
          disabled={loading}
        />
        <small className="help-text">
          The audience is typically the base URL of the service you're authenticating with.
        </small>
      </div>
      
      <button onClick={generateToken} disabled={loading}>
        {loading ? "Generating..." : "Generate Token"}
      </button>
      
      {error && <div className="error">{error}</div>}
      
      {token && (
        <div className="token-display">
          <h4>Generated Token:</h4>
          <textarea
            readOnly
            value={token}
            rows={6}
            className="token-value"
          />
          <div className="button-group">
            <button onClick={() => navigator.clipboard.writeText(token)}>
              Copy Token
            </button>
            <button onClick={() => setToken("")}>
              Clear
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
```

### Generating Tokens Without Audience

You can also generate tokens without specifying an audience for general-purpose use:

```tsx
// Generate a token with no specific audience
const { token } = await os.generateThirdPartyToken();
```

## Understanding Token Structure

The tokens generated are standard JWT (JSON Web Tokens) with the following claims:

- `iss` (Issuer): The OpenSecret issuer URL
- `sub` (Subject): The user's ID
- `aud` (Audience): The audience URL (if specified)
- `exp` (Expiration Time): When the token expires
- `iat` (Issued At): When the token was issued
- User information: Custom claims with user details

You can inspect JWT tokens at [jwt.io](https://jwt.io/) to view their contents.

## Using Third-Party Tokens

### Authentication with External Services

To use a token with an external service:

```typescript
// Generate token for specific service
const { token } = await os.generateThirdPartyToken("https://api.example.com");

// Use the token in an API request
const response = await fetch("https://api.example.com/data", {
  method: "GET",
  headers: {
    "Authorization": `Bearer ${token}`,
    "Content-Type": "application/json"
  }
});

const data = await response.json();
```

### JWT Verification on the Server Side

On your server, you'll need to verify the JWT token. Here's an example using Node.js and the `jsonwebtoken` library:

```javascript
// Server-side code (Node.js)
const jwt = require('jsonwebtoken');
const jwksClient = require('jwks-rsa');

// Set up a JWKS client to fetch the public keys
const client = jwksClient({
  jwksUri: 'https://api.opensecret.cloud/.well-known/jwks.json'
});

// Function to get the signing key
function getSigningKey(header, callback) {
  client.getSigningKey(header.kid, (err, key) => {
    if (err) return callback(err);
    const signingKey = key.publicKey || key.rsaPublicKey;
    callback(null, signingKey);
  });
}

// Middleware to verify the token
function verifyToken(req, res, next) {
  const token = req.headers.authorization?.split(' ')[1];
  if (!token) {
    return res.status(401).json({ message: 'No token provided' });
  }

  jwt.verify(token, getSigningKey, {
    algorithms: ['RS256'],
    audience: 'https://api.example.com' // Your service URL
  }, (err, decoded) => {
    if (err) {
      return res.status(401).json({ message: 'Invalid token' });
    }
    
    // Token is valid, attach the user info to the request
    req.user = decoded;
    next();
  });
}

// Use the middleware in your routes
app.get('/protected-data', verifyToken, (req, res) => {
  // Access user info from the token
  const userId = req.user.sub;
  // Process the request...
  res.json({ data: 'Your protected data' });
});
```

## Complete Token Management Component

Here's a more complete component for token management:

```tsx
import React, { useState, useEffect } from "react";
import { useOpenSecret } from "@opensecret/react";

interface JwtClaims {
  iss: string;
  sub: string;
  aud?: string;
  exp: number;
  iat: number;
  [key: string]: any;
}

function TokenManager() {
  const os = useOpenSecret();
  const [audience, setAudience] = useState("");
  const [token, setToken] = useState("");
  const [decodedToken, setDecodedToken] = useState<JwtClaims | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [tokenExpiry, setTokenExpiry] = useState<Date | null>(null);
  const [timeRemaining, setTimeRemaining] = useState("");
  
  // Parse JWT token to get payload
  function parseJwt(token: string): JwtClaims | null {
    try {
      const base64Url = token.split('.')[1];
      const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/');
      const jsonPayload = decodeURIComponent(
        atob(base64).split('').map(c => {
          return '%' + ('00' + c.charCodeAt(0).toString(16)).slice(-2);
        }).join('')
      );
      return JSON.parse(jsonPayload);
    } catch (e) {
      console.error("Failed to parse JWT:", e);
      return null;
    }
  }
  
  // Update token time remaining
  useEffect(() => {
    if (!tokenExpiry) return;
    
    const updateTimeRemaining = () => {
      const now = new Date();
      const diff = tokenExpiry.getTime() - now.getTime();
      
      if (diff <= 0) {
        setTimeRemaining("Expired");
        return;
      }
      
      const minutes = Math.floor(diff / 60000);
      const seconds = Math.floor((diff % 60000) / 1000);
      setTimeRemaining(`${minutes}m ${seconds}s`);
    };
    
    updateTimeRemaining();
    const interval = setInterval(updateTimeRemaining, 1000);
    
    return () => clearInterval(interval);
  }, [tokenExpiry]);
  
  // Update decoded token when token changes
  useEffect(() => {
    if (!token) {
      setDecodedToken(null);
      setTokenExpiry(null);
      return;
    }
    
    const decoded = parseJwt(token);
    setDecodedToken(decoded);
    
    if (decoded && decoded.exp) {
      setTokenExpiry(new Date(decoded.exp * 1000));
    }
  }, [token]);
  
  async function handleGenerateToken() {
    setLoading(true);
    setError("");
    try {
      const aud = audience.trim() ? audience : undefined;
      const response = await os.generateThirdPartyToken(aud);
      setToken(response.token);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to generate token");
    } finally {
      setLoading(false);
    }
  }
  
  if (!os.auth.user) {
    return <div className="auth-message">Please log in to generate tokens</div>;
  }
  
  return (
    <div className="token-manager">
      <h3>Third-Party Token Generator</h3>
      
      <div className="input-group">
        <label htmlFor="audience">Audience URL (Optional):</label>
        <input
          id="audience"
          type="url"
          value={audience}
          onChange={(e) => setAudience(e.target.value)}
          placeholder="https://api.yourservice.com"
          disabled={loading}
        />
        <small className="help-text">
          Leave blank for a token with no audience restriction
        </small>
      </div>
      
      <button 
        onClick={handleGenerateToken} 
        disabled={loading}
        className="generate-button"
      >
        {loading ? "Generating..." : "Generate JWT Token"}
      </button>
      
      {error && <div className="error">{error}</div>}
      
      {token && (
        <div className="token-result">
          <div className="token-header">
            <h4>JWT Token</h4>
            {tokenExpiry && (
              <div className="token-expiry">
                Expires in: <span className="time-remaining">{timeRemaining}</span>
              </div>
            )}
          </div>
          
          <textarea
            readOnly
            value={token}
            rows={4}
            className="token-value"
          />
          
          <div className="button-group">
            <button 
              onClick={() => navigator.clipboard.writeText(token)}
              className="copy-button"
            >
              Copy Token
            </button>
            <a 
              href={`https://jwt.io/#debugger-io?token=${token}`} 
              target="_blank" 
              rel="noopener noreferrer"
              className="inspect-button"
            >
              Inspect on JWT.io
            </a>
          </div>
          
          {decodedToken && (
            <div className="token-claims">
              <h4>Token Claims:</h4>
              <div className="claims-list">
                <div className="claim-item">
                  <span className="claim-name">Subject (User ID):</span>
                  <span className="claim-value">{decodedToken.sub}</span>
                </div>
                {decodedToken.aud && (
                  <div className="claim-item">
                    <span className="claim-name">Audience:</span>
                    <span className="claim-value">{decodedToken.aud}</span>
                  </div>
                )}
                <div className="claim-item">
                  <span className="claim-name">Issuer:</span>
                  <span className="claim-value">{decodedToken.iss}</span>
                </div>
                <div className="claim-item">
                  <span className="claim-name">Issued At:</span>
                  <span className="claim-value">
                    {new Date(decodedToken.iat * 1000).toLocaleString()}
                  </span>
                </div>
                <div className="claim-item">
                  <span className="claim-name">Expires At:</span>
                  <span className="claim-value">
                    {new Date(decodedToken.exp * 1000).toLocaleString()}
                  </span>
                </div>
                
                {/* Display custom claims */}
                {Object.entries(decodedToken)
                  .filter(([key]) => !['iss', 'sub', 'aud', 'exp', 'iat'].includes(key))
                  .map(([key, value]) => (
                    <div className="claim-item" key={key}>
                      <span className="claim-name">{key}:</span>
                      <span className="claim-value">
                        {typeof value === 'object' 
                          ? JSON.stringify(value) 
                          : String(value)
                        }
                      </span>
                    </div>
                  ))
                }
              </div>
            </div>
          )}
          
          <div className="usage-example">
            <h4>Usage Example:</h4>
            <pre>
              {`
// JavaScript fetch example
fetch('${audience || "https://your-api-endpoint.com"}', {
  method: 'GET',
  headers: {
    'Authorization': 'Bearer ${token.slice(0, 20)}...',
    'Content-Type': 'application/json'
  }
})
.then(response => response.json())
.then(data => console.log(data));
              `.trim()}
            </pre>
          </div>
        </div>
      )}
    </div>
  );
}
```

## Use Cases for Third-Party Tokens

### Microservices Authentication

Use third-party tokens to authenticate between your microservices:

```typescript
// Service A - Generate token for Service B
const { token } = await os.generateThirdPartyToken("https://service-b.example.com");

// Call Service B with the token
const response = await fetch("https://service-b.example.com/api/data", {
  headers: {
    "Authorization": `Bearer ${token}`
  }
});
```

### Single Sign-On (SSO)

Implement SSO across multiple applications:

```typescript
// Main application - Generate token for satellite app
const { token } = await os.generateThirdPartyToken("https://satellite-app.example.com");

// Redirect to satellite app with token
window.location.href = `https://satellite-app.example.com/sso?token=${token}`;

// Satellite app - Verify the token and create a session
// (Server-side code)
```

### API Authorization

Control access to your API:

```typescript
// Generate token with specific permissions in the payload
const { token } = await os.generateThirdPartyToken("https://api.example.com");

// Use token to access API
const response = await fetch("https://api.example.com/admin/settings", {
  headers: {
    "Authorization": `Bearer ${token}`
  }
});
```

## Security Considerations

1. **Token expiration**: All tokens have an expiration time; refresh them as needed

2. **Audience validation**: Always validate the `aud` claim on the server to prevent token misuse

3. **Transport security**: Always use HTTPS when transmitting tokens

4. **Minimize token scope**: Generate tokens with the minimum permissions needed

5. **Don't store tokens in localStorage**: For sensitive operations, keep tokens in memory only

## Best Practices

1. **Generate tokens on demand**: Don't store tokens; generate new ones when needed

2. **Use specific audiences**: Specify the exact audience URL for better security

3. **Implement proper error handling**: Token generation or verification might fail

4. **Handle token expiration gracefully**: Refresh tokens before they expire

5. **Audit token usage**: Log token creation and usage for security monitoring

## Next Steps

- [Cryptographic Operations](./cryptographic-operations) - Learn about other cryptographic features
- [Remote Attestation](./remote-attestation) - Understand how OpenSecret verifies server security
- [Data Encryption](./data-encryption) - Explore client-side encryption capabilities