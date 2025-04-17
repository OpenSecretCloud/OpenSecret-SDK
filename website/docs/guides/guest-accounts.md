---
title: Guest Accounts
sidebar_position: 3
---

# Guest Accounts

OpenSecret provides support for guest accounts, allowing users to access your application without providing an email address. This guide explains how to implement and manage guest accounts in your application.

## Overview

Guest accounts offer several benefits:

- Reduce friction in user onboarding
- Allow users to try your application before committing
- Support scenarios where email collection isn't initially required
- Maintain security and user-specific data without email authentication

## Prerequisites

Before implementing guest accounts, make sure:

1. Your application is wrapped with `OpenSecretProvider`
2. Guest accounts are enabled in your project settings

## Creating Guest Accounts

To create a guest account, use the `signUpGuest` method:

```tsx
import { useState } from "react";
import { useOpenSecret } from "@opensecret/react";

function GuestSignupForm() {
  const os = useOpenSecret();
  const [password, setPassword] = useState("");
  const [inviteCode, setInviteCode] = useState("");
  const [guestId, setGuestId] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  
  async function handleGuestSignup(e) {
    e.preventDefault();
    setLoading(true);
    setError("");
    
    try {
      const response = await os.signUpGuest(password, inviteCode);
      setGuestId(response.id);
      // Guest is now registered and logged in
    } catch (err) {
      setError(err instanceof Error ? err.message : "Guest signup failed");
    } finally {
      setLoading(false);
    }
  }
  
  return (
    <div>
      <h3>Create Guest Account</h3>
      <form onSubmit={handleGuestSignup}>
        <div className="form-group">
          <label htmlFor="password">Password:</label>
          <input
            id="password"
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            minLength={8}
            required
            disabled={loading}
          />
        </div>
        <div className="form-group">
          <label htmlFor="inviteCode">Invite Code:</label>
          <input
            id="inviteCode"
            type="text"
            value={inviteCode}
            onChange={(e) => setInviteCode(e.target.value)}
            required
            disabled={loading}
          />
        </div>
        {error && <div className="error">{error}</div>}
        <button type="submit" disabled={loading}>
          {loading ? "Creating..." : "Create Guest Account"}
        </button>
      </form>
      
      {guestId && (
        <div className="success-message">
          <h4>Guest Account Created!</h4>
          <p><strong>Important:</strong> Please save your guest ID:</p>
          <div className="guest-id">{guestId}</div>
          <p>You will need this ID to log in again. Without it, you cannot access your account.</p>
          <button onClick={() => navigator.clipboard.writeText(guestId)}>
            Copy ID to Clipboard
          </button>
        </div>
      )}
    </div>
  );
}
```

## Guest Login

Once a guest account is created, users can log in using their guest ID and password:

```tsx
import { useState } from "react";
import { useOpenSecret } from "@opensecret/react";

function GuestLoginForm() {
  const os = useOpenSecret();
  const [guestId, setGuestId] = useState("");
  const [password, setPassword] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  
  async function handleGuestLogin(e) {
    e.preventDefault();
    setLoading(true);
    setError("");
    
    try {
      await os.signInGuest(guestId, password);
      // Guest is now logged in
    } catch (err) {
      setError(err instanceof Error ? err.message : "Guest login failed");
    } finally {
      setLoading(false);
    }
  }
  
  return (
    <div>
      <h3>Guest Login</h3>
      <form onSubmit={handleGuestLogin}>
        <div className="form-group">
          <label htmlFor="guestId">Guest ID:</label>
          <input
            id="guestId"
            type="text"
            value={guestId}
            onChange={(e) => setGuestId(e.target.value)}
            required
            disabled={loading}
          />
        </div>
        <div className="form-group">
          <label htmlFor="password">Password:</label>
          <input
            id="password"
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            required
            disabled={loading}
          />
        </div>
        {error && <div className="error">{error}</div>}
        <button type="submit" disabled={loading}>
          {loading ? "Logging in..." : "Login as Guest"}
        </button>
      </form>
    </div>
  );
}
```

## Converting Guest Accounts to Regular Accounts

One of the most powerful features of guest accounts is the ability to convert them to regular email-based accounts. This allows users to try your application and then upgrade their account to a permanent one without losing their data.

```tsx
import { useState } from "react";
import { useOpenSecret } from "@opensecret/react";

function ConvertGuestAccount() {
  const os = useOpenSecret();
  const [email, setEmail] = useState("");
  const [name, setName] = useState("");
  const [password, setPassword] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [success, setSuccess] = useState(false);
  
  // Only show for guest accounts
  const isGuest = os.auth.user && !os.auth.user.email;
  if (!isGuest) {
    return null;
  }
  
  async function handleConversion(e) {
    e.preventDefault();
    setLoading(true);
    setError("");
    setSuccess(false);
    
    try {
      await os.convertGuestToUserAccount(email, password, name);
      setSuccess(true);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Account conversion failed");
    } finally {
      setLoading(false);
    }
  }
  
  return (
    <div className="conversion-form">
      <h3>Upgrade to a Permanent Account</h3>
      <p>
        Add an email address to your guest account to make it permanent and 
        easier to access in the future.
      </p>
      
      <form onSubmit={handleConversion}>
        <div className="form-group">
          <label htmlFor="email">Email Address:</label>
          <input
            id="email"
            type="email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            required
            disabled={loading || success}
          />
        </div>
        <div className="form-group">
          <label htmlFor="name">Name (Optional):</label>
          <input
            id="name"
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            disabled={loading || success}
          />
        </div>
        <div className="form-group">
          <label htmlFor="password">New Password:</label>
          <input
            id="password"
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            minLength={8}
            required
            disabled={loading || success}
          />
        </div>
        
        {error && <div className="error">{error}</div>}
        {success && (
          <div className="success-message">
            <h4>Account Upgraded Successfully!</h4>
            <p>
              Your guest account has been converted to a permanent account.
              You can now log in using your email and password.
            </p>
          </div>
        )}
        
        {!success && (
          <button type="submit" disabled={loading}>
            {loading ? "Upgrading..." : "Upgrade Account"}
          </button>
        )}
      </form>
    </div>
  );
}
```

## Authentication UI with Guest Support

Here's a complete authentication UI example that supports both regular and guest accounts:

```tsx
import { useState } from "react";
import { useOpenSecret } from "@opensecret/react";

function AuthenticationUI() {
  const os = useOpenSecret();
  const [activeTab, setActiveTab] = useState("login");
  
  // Log out current user
  function handleLogout() {
    os.signOut();
  }
  
  // Render authenticated user info
  if (os.auth.user) {
    const isGuest = !os.auth.user.email;
    
    return (
      <div className="auth-container">
        <h3>Welcome, {os.auth.user.name || "Guest User"}</h3>
        
        <div className="user-info">
          <div className="info-row">
            <span className="label">ID:</span>
            <span className="value">{os.auth.user.id}</span>
          </div>
          {os.auth.user.email && (
            <div className="info-row">
              <span className="label">Email:</span>
              <span className="value">{os.auth.user.email}</span>
            </div>
          )}
          <div className="info-row">
            <span className="label">Account Type:</span>
            <span className="value">{isGuest ? "Guest Account" : "Regular Account"}</span>
          </div>
        </div>
        
        <button onClick={handleLogout} className="logout-button">
          Log Out
        </button>
        
        {/* Show conversion option for guest accounts */}
        {isGuest && <ConvertGuestAccount />}
      </div>
    );
  }
  
  // Render login/signup UI for unauthenticated users
  return (
    <div className="auth-container">
      <div className="auth-tabs">
        <button
          className={activeTab === "login" ? "active" : ""}
          onClick={() => setActiveTab("login")}
        >
          Login
        </button>
        <button
          className={activeTab === "signup" ? "active" : ""}
          onClick={() => setActiveTab("signup")}
        >
          Sign Up
        </button>
        <button
          className={activeTab === "guest" ? "active" : ""}
          onClick={() => setActiveTab("guest")}
        >
          Guest Access
        </button>
      </div>
      
      <div className="auth-form-container">
        {activeTab === "login" && <RegularLoginForm />}
        {activeTab === "signup" && <RegularSignupForm />}
        {activeTab === "guest" && (
          <div className="guest-options">
            <div className="guest-option-tabs">
              <button
                className={activeGuestTab === "create" ? "active" : ""}
                onClick={() => setActiveGuestTab("create")}
              >
                Create Guest Account
              </button>
              <button
                className={activeGuestTab === "login" ? "active" : ""}
                onClick={() => setActiveGuestTab("login")}
              >
                Guest Login
              </button>
            </div>
            
            {activeGuestTab === "create" && <GuestSignupForm />}
            {activeGuestTab === "login" && <GuestLoginForm />}
          </div>
        )}
      </div>
    </div>
  );
}

// Components for RegularLoginForm and RegularSignupForm would be implemented similarly
// to the examples in the Authentication guide
```

## Handling Guest Account Data

Guest accounts can use all the same features as regular accounts:

### Key-Value Storage

```tsx
// Store data in a guest account
await os.put("preferences", JSON.stringify({ theme: "dark" }));

// Retrieve data
const preferences = await os.get("preferences");
```

### Cryptographic Operations

```tsx
// Get public key for a guest account
const { public_key } = await os.getPublicKey("schnorr");

// Sign messages
const messageBytes = new TextEncoder().encode("Hello, world!");
const signature = await os.signMessage(messageBytes, "schnorr");
```

All data associated with a guest account is preserved when converting to a regular account.

## Security Considerations

1. **Password security is critical**: Since recovery via email isn't possible for guest accounts, encourage strong passwords

2. **Explain the importance of saving the guest ID**: Make it clear that losing the guest ID means losing access to the account

3. **Consider session persistence**: For guest accounts, you might want longer-lived tokens to reduce the frequency of logins

4. **Rate limit guest account creation**: To prevent abuse, consider rate limiting guest account creation by IP address

5. **Encourage conversion to regular accounts**: For important or long-lived data, encourage users to convert to regular accounts

## Best Practices

1. **Simplify the guest experience**: Minimize the information required to create a guest account

2. **Provide clear guest ID instructions**: Make it obvious that users need to save their guest ID

3. **Periodically remind guest users to upgrade**: Consider showing occasional upgrade prompts for guest users

4. **Implement session recovery options**: For guest users who haven't converted, consider client-side storage for ID recovery

5. **Be transparent about limitations**: Clearly communicate any feature limitations for guest accounts

## Example: Progressive Authentication Flow

A common pattern is to implement a progressive authentication flow:

1. **Start as guest**: User starts with a guest account for immediate access
2. **Store user data**: Application stores user preferences and progress
3. **Prompt for conversion**: When the user reaches a certain level of engagement, prompt for account conversion
4. **Seamless upgrade**: Convert to a regular account while preserving all data

This provides a frictionless onboarding experience while eventually capturing user information for those who find value in your application.

## Next Steps

- [Key-Value Storage](./key-value-storage) - Learn how to store data for guest users
- [Data Encryption](./data-encryption) - Explore encryption for sensitive guest user data
- [Cryptographic Operations](./cryptographic-operations) - Use cryptographic features with guest accounts