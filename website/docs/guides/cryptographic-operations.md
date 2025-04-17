---
title: Cryptographic Operations
sidebar_position: 5
---

# Cryptographic Operations

OpenSecret provides powerful cryptographic capabilities that enable your application to perform secure key operations, message signing, and more. This guide explains how to use the cryptographic features of the SDK.

## Overview

OpenSecret's cryptographic operations include:

- Retrieving private key mnemonics and bytes
- Accessing public keys for different signing algorithms
- Signing messages with Schnorr or ECDSA
- Supporting flexible key derivation paths
- Client-side encryption and decryption

## Prerequisites

Before using cryptographic operations, make sure:

1. Your application is wrapped with `OpenSecretProvider`
2. The user is authenticated
3. You have imported the `useOpenSecret` hook

## Key Derivation Options

Many cryptographic operations in OpenSecret support key derivation options, allowing you to:

- Use the master key (default)
- Derive a key using BIP-32 path
- Derive a child mnemonic using BIP-85
- Combine BIP-85 and BIP-32 derivation

The `KeyOptions` type is defined as:

```typescript
type KeyOptions = {
  /** 
   * BIP-85 derivation path to derive a child mnemonic
   * Example: "m/83696968'/39'/0'/12'/0'"
   */
  seed_phrase_derivation_path?: string;
  
  /**
   * BIP-32 derivation path to derive a child key from the master (or BIP-85 derived) seed
   * Example: "m/44'/0'/0'/0/0"
   */
  private_key_derivation_path?: string;
};
```

## Private Key Operations

### Retrieving a Private Key Mnemonic

```tsx
import { useState } from "react";
import { useOpenSecret } from "@opensecret/react";

function MnemonicDisplay() {
  const os = useOpenSecret();
  const [mnemonic, setMnemonic] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  
  async function getMasterMnemonic() {
    setLoading(true);
    setError("");
    try {
      // Get the master mnemonic (12 words)
      const response = await os.getPrivateKey();
      setMnemonic(response.mnemonic);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to get mnemonic");
    } finally {
      setLoading(false);
    }
  }
  
  async function getDerivedMnemonic() {
    setLoading(true);
    setError("");
    try {
      // Get a BIP-85 derived child mnemonic
      const response = await os.getPrivateKey({
        seed_phrase_derivation_path: "m/83696968'/39'/0'/12'/0'"
      });
      setMnemonic(response.mnemonic);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to get derived mnemonic");
    } finally {
      setLoading(false);
    }
  }
  
  return (
    <div>
      <h3>Private Key Mnemonic</h3>
      <div className="button-group">
        <button onClick={getMasterMnemonic} disabled={loading}>
          Get Master Mnemonic
        </button>
        <button onClick={getDerivedMnemonic} disabled={loading}>
          Get Derived Mnemonic
        </button>
      </div>
      
      {error && <div className="error">{error}</div>}
      
      {mnemonic && (
        <div className="mnemonic-display">
          <p>Your mnemonic phrase:</p>
          <div className="mnemonic-words">
            {mnemonic.split(" ").map((word, index) => (
              <span key={index} className="mnemonic-word">{word}</span>
            ))}
          </div>
          <p className="warning">
            Keep this phrase secure and never share it with anyone.
          </p>
        </div>
      )}
    </div>
  );
}
```

### Retrieving Private Key Bytes

```tsx
import { useState } from "react";
import { useOpenSecret } from "@opensecret/react";

function PrivateKeyDisplay() {
  const os = useOpenSecret();
  const [derivationPath, setDerivationPath] = useState("m/44'/0'/0'/0/0");
  const [privateKey, setPrivateKey] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  
  async function getMasterKeyBytes() {
    setLoading(true);
    setError("");
    try {
      // Get the master private key bytes (no derivation)
      const response = await os.getPrivateKeyBytes();
      setPrivateKey(response.private_key);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to get private key");
    } finally {
      setLoading(false);
    }
  }
  
  async function getDerivedKeyBytes() {
    setLoading(true);
    setError("");
    try {
      // Get derived private key bytes using BIP-32
      const response = await os.getPrivateKeyBytes({
        private_key_derivation_path: derivationPath
      });
      setPrivateKey(response.private_key);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to get derived key");
    } finally {
      setLoading(false);
    }
  }
  
  return (
    <div>
      <h3>Private Key Bytes</h3>
      
      <div className="input-group">
        <label htmlFor="derivationPath">BIP-32 Derivation Path:</label>
        <input
          id="derivationPath"
          type="text"
          value={derivationPath}
          onChange={(e) => setDerivationPath(e.target.value)}
          placeholder="e.g. m/44'/0'/0'/0/0"
        />
      </div>
      
      <div className="button-group">
        <button onClick={getMasterKeyBytes} disabled={loading}>
          Get Master Key
        </button>
        <button onClick={getDerivedKeyBytes} disabled={loading}>
          Get Derived Key
        </button>
      </div>
      
      {error && <div className="error">{error}</div>}
      
      {privateKey && (
        <div className="key-display">
          <p>Private key (hex):</p>
          <code className="key-value">{privateKey}</code>
          <p className="warning">
            Keep this key secure and never share it with anyone.
          </p>
        </div>
      )}
    </div>
  );
}
```

## Public Key Operations

### Retrieving a Public Key

```tsx
import { useState } from "react";
import { useOpenSecret } from "@opensecret/react";

function PublicKeyDisplay() {
  const os = useOpenSecret();
  const [algorithm, setAlgorithm] = useState<"schnorr" | "ecdsa">("schnorr");
  const [derivationPath, setDerivationPath] = useState("m/84'/0'/0'/0/0");
  const [publicKey, setPublicKey] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  
  async function getPublicKey() {
    setLoading(true);
    setError("");
    try {
      // Get public key using selected algorithm and derivation path
      const response = await os.getPublicKey(algorithm, {
        private_key_derivation_path: derivationPath
      });
      setPublicKey(response.public_key);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to get public key");
    } finally {
      setLoading(false);
    }
  }
  
  return (
    <div>
      <h3>Public Key</h3>
      
      <div className="input-group">
        <label htmlFor="algorithm">Signing Algorithm:</label>
        <select
          id="algorithm"
          value={algorithm}
          onChange={(e) => setAlgorithm(e.target.value as "schnorr" | "ecdsa")}
        >
          <option value="schnorr">Schnorr</option>
          <option value="ecdsa">ECDSA</option>
        </select>
      </div>
      
      <div className="input-group">
        <label htmlFor="derivationPath">BIP-32 Derivation Path:</label>
        <input
          id="derivationPath"
          type="text"
          value={derivationPath}
          onChange={(e) => setDerivationPath(e.target.value)}
          placeholder="e.g. m/84'/0'/0'/0/0"
        />
      </div>
      
      <button onClick={getPublicKey} disabled={loading}>
        Get Public Key
      </button>
      
      {error && <div className="error">{error}</div>}
      
      {publicKey && (
        <div className="key-display">
          <p>{algorithm === "schnorr" ? "Schnorr" : "ECDSA"} public key:</p>
          <code className="key-value">{publicKey}</code>
        </div>
      )}
    </div>
  );
}
```

## Message Signing and Verification

### Signing a Message

```tsx
import { useState } from "react";
import { useOpenSecret } from "@opensecret/react";

function MessageSigner() {
  const os = useOpenSecret();
  const [message, setMessage] = useState("");
  const [algorithm, setAlgorithm] = useState<"schnorr" | "ecdsa">("schnorr");
  const [derivationPath, setDerivationPath] = useState("m/84'/0'/0'/0/0");
  const [signature, setSignature] = useState("");
  const [messageHash, setMessageHash] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  
  async function signMessage() {
    if (!message.trim()) {
      setError("Please enter a message to sign");
      return;
    }
    
    setLoading(true);
    setError("");
    try {
      // Convert message to bytes
      const messageBytes = new TextEncoder().encode(message);
      
      // Sign the message
      const response = await os.signMessage(
        messageBytes,
        algorithm,
        derivationPath ? { private_key_derivation_path: derivationPath } : undefined
      );
      
      setSignature(response.signature);
      setMessageHash(response.message_hash);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to sign message");
    } finally {
      setLoading(false);
    }
  }
  
  return (
    <div>
      <h3>Message Signing</h3>
      
      <div className="input-group">
        <label htmlFor="message">Message to Sign:</label>
        <textarea
          id="message"
          value={message}
          onChange={(e) => setMessage(e.target.value)}
          rows={3}
          placeholder="Enter your message here"
        />
      </div>
      
      <div className="input-group">
        <label htmlFor="algorithm">Signing Algorithm:</label>
        <select
          id="algorithm"
          value={algorithm}
          onChange={(e) => setAlgorithm(e.target.value as "schnorr" | "ecdsa")}
        >
          <option value="schnorr">Schnorr</option>
          <option value="ecdsa">ECDSA</option>
        </select>
      </div>
      
      <div className="input-group">
        <label htmlFor="derivationPath">BIP-32 Derivation Path (optional):</label>
        <input
          id="derivationPath"
          type="text"
          value={derivationPath}
          onChange={(e) => setDerivationPath(e.target.value)}
          placeholder="e.g. m/84'/0'/0'/0/0"
        />
      </div>
      
      <button onClick={signMessage} disabled={loading}>
        {loading ? "Signing..." : "Sign Message"}
      </button>
      
      {error && <div className="error">{error}</div>}
      
      {signature && (
        <div className="signature-display">
          <div className="result-item">
            <h4>Message Hash:</h4>
            <code>{messageHash}</code>
          </div>
          
          <div className="result-item">
            <h4>Signature:</h4>
            <code>{signature}</code>
          </div>
          
          <p className="info">
            Use these values with the corresponding public key to verify this signature.
          </p>
        </div>
      )}
    </div>
  );
}
```

### Verifying a Signature

For client-side verification, you can use libraries like `@noble/curves`:

```tsx
import { useState } from "react";
import { useOpenSecret } from "@opensecret/react";
import { schnorr, secp256k1 } from "@noble/curves/secp256k1";

function SignatureVerifier() {
  const os = useOpenSecret();
  const [algorithm, setAlgorithm] = useState<"schnorr" | "ecdsa">("schnorr");
  const [publicKey, setPublicKey] = useState("");
  const [signature, setSignature] = useState("");
  const [messageHash, setMessageHash] = useState("");
  const [verificationResult, setVerificationResult] = useState<boolean | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  
  async function verifySignature() {
    if (!publicKey || !signature || !messageHash) {
      setError("Please provide all required fields");
      return;
    }
    
    setLoading(true);
    setError("");
    setVerificationResult(null);
    
    try {
      // Verify using the appropriate algorithm
      let isValid: boolean;
      
      if (algorithm === "schnorr") {
        isValid = schnorr.verify(signature, messageHash, publicKey);
      } else {
        isValid = secp256k1.verify(signature, messageHash, publicKey);
      }
      
      setVerificationResult(isValid);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Verification failed");
    } finally {
      setLoading(false);
    }
  }
  
  return (
    <div>
      <h3>Signature Verification</h3>
      
      <div className="input-group">
        <label htmlFor="verifyAlgorithm">Signing Algorithm:</label>
        <select
          id="verifyAlgorithm"
          value={algorithm}
          onChange={(e) => setAlgorithm(e.target.value as "schnorr" | "ecdsa")}
        >
          <option value="schnorr">Schnorr</option>
          <option value="ecdsa">ECDSA</option>
        </select>
      </div>
      
      <div className="input-group">
        <label htmlFor="publicKey">Public Key:</label>
        <input
          id="publicKey"
          type="text"
          value={publicKey}
          onChange={(e) => setPublicKey(e.target.value)}
          placeholder="Enter public key"
        />
      </div>
      
      <div className="input-group">
        <label htmlFor="signature">Signature:</label>
        <input
          id="signature"
          type="text"
          value={signature}
          onChange={(e) => setSignature(e.target.value)}
          placeholder="Enter signature"
        />
      </div>
      
      <div className="input-group">
        <label htmlFor="messageHash">Message Hash:</label>
        <input
          id="messageHash"
          type="text"
          value={messageHash}
          onChange={(e) => setMessageHash(e.target.value)}
          placeholder="Enter message hash"
        />
      </div>
      
      <button onClick={verifySignature} disabled={loading}>
        {loading ? "Verifying..." : "Verify Signature"}
      </button>
      
      {error && <div className="error">{error}</div>}
      
      {verificationResult !== null && (
        <div className={`verification-result ${verificationResult ? "valid" : "invalid"}`}>
          Signature is <strong>{verificationResult ? "valid" : "invalid"}</strong>
        </div>
      )}
    </div>
  );
}
```

## Complete Signing Example

Here's a complete example that demonstrates getting a public key, signing a message, and verifying the signature:

```tsx
import React, { useState } from "react";
import { useOpenSecret } from "@opensecret/react";
import { schnorr, secp256k1 } from "@noble/curves/secp256k1";

function CryptographicDemo() {
  const os = useOpenSecret();
  
  const [algorithm, setAlgorithm] = useState<"schnorr" | "ecdsa">("schnorr");
  const [derivationPath, setDerivationPath] = useState("m/84'/0'/0'/0/0");
  const [message, setMessage] = useState("");
  
  const [publicKey, setPublicKey] = useState("");
  const [lastSignature, setLastSignature] = useState<{
    signature: string;
    messageHash: string;
    message: string;
  } | null>(null);
  const [verificationResult, setVerificationResult] = useState<boolean | null>(null);
  
  const [loading, setLoading] = useState({
    publicKey: false,
    signing: false,
    verification: false
  });
  const [error, setError] = useState("");
  
  async function handleGetPublicKey() {
    setLoading({ ...loading, publicKey: true });
    setError("");
    try {
      const keyOptions = derivationPath
        ? { private_key_derivation_path: derivationPath }
        : undefined;
      const response = await os.getPublicKey(algorithm, keyOptions);
      setPublicKey(response.public_key);
      setVerificationResult(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to get public key");
    } finally {
      setLoading({ ...loading, publicKey: false });
    }
  }
  
  async function handleSignMessage(e: React.FormEvent) {
    e.preventDefault();
    if (!message.trim()) {
      setError("Please enter a message to sign");
      return;
    }
    
    setLoading({ ...loading, signing: true });
    setError("");
    try {
      const messageBytes = new TextEncoder().encode(message);
      const keyOptions = derivationPath
        ? { private_key_derivation_path: derivationPath }
        : undefined;
      const response = await os.signMessage(messageBytes, algorithm, keyOptions);
      
      setLastSignature({
        signature: response.signature,
        messageHash: response.message_hash,
        message
      });
      setVerificationResult(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to sign message");
    } finally {
      setLoading({ ...loading, signing: false });
    }
  }
  
  async function handleVerifySignature() {
    if (!lastSignature || !publicKey) {
      setError("Please get public key and sign a message first");
      return;
    }
    
    setLoading({ ...loading, verification: true });
    setError("");
    try {
      let isValid: boolean;
      if (algorithm === "schnorr") {
        isValid = schnorr.verify(
          lastSignature.signature,
          lastSignature.messageHash,
          publicKey
        );
      } else {
        isValid = secp256k1.verify(
          lastSignature.signature,
          lastSignature.messageHash,
          publicKey
        );
      }
      setVerificationResult(isValid);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Verification failed");
    } finally {
      setLoading({ ...loading, verification: false });
    }
  }
  
  return (
    <div className="crypto-demo">
      <h2>Cryptographic Operations Demo</h2>
      
      {error && <div className="error">{error}</div>}
      
      <div className="section">
        <h3>Configuration</h3>
        
        <div className="input-group">
          <label htmlFor="algorithm">Signing Algorithm:</label>
          <select
            id="algorithm"
            value={algorithm}
            onChange={(e) => setAlgorithm(e.target.value as "schnorr" | "ecdsa")}
            disabled={loading.publicKey || loading.signing}
          >
            <option value="schnorr">Schnorr</option>
            <option value="ecdsa">ECDSA</option>
          </select>
        </div>
        
        <div className="input-group">
          <label htmlFor="derivationPath">Derivation Path:</label>
          <input
            id="derivationPath"
            type="text"
            value={derivationPath}
            onChange={(e) => setDerivationPath(e.target.value)}
            placeholder="e.g. m/84'/0'/0'/0/0"
            disabled={loading.publicKey || loading.signing}
          />
          <div className="help-text">
            <details>
              <summary>Common derivation paths</summary>
              <ul>
                <li>BIP44 (Legacy): m/44'/0'/0'/0/0</li>
                <li>BIP49 (SegWit): m/49'/0'/0'/0/0</li>
                <li>BIP84 (Native SegWit): m/84'/0'/0'/0/0</li>
                <li>BIP86 (Taproot): m/86'/0'/0'/0/0</li>
              </ul>
            </details>
          </div>
        </div>
        
        <button
          onClick={handleGetPublicKey}
          disabled={loading.publicKey || loading.signing}
        >
          {loading.publicKey ? "Getting Public Key..." : "Get Public Key"}
        </button>
      </div>
      
      {publicKey && (
        <div className="section result-section">
          <h3>Public Key:</h3>
          <code className="key-display">{publicKey}</code>
        </div>
      )}
      
      <div className="section">
        <h3>Message Signing</h3>
        
        <form onSubmit={handleSignMessage}>
          <div className="input-group">
            <label htmlFor="message">Message to Sign:</label>
            <textarea
              id="message"
              value={message}
              onChange={(e) => setMessage(e.target.value)}
              placeholder="Enter your message here"
              rows={3}
              disabled={loading.signing}
              required
            />
          </div>
          
          <button
            type="submit"
            disabled={loading.signing || !derivationPath}
          >
            {loading.signing ? "Signing..." : "Sign Message"}
          </button>
        </form>
      </div>
      
      {lastSignature && (
        <div className="section result-section">
          <h3>Signature Results:</h3>
          
          <div className="result-item">
            <h4>Message:</h4>
            <div className="message-content">{lastSignature.message}</div>
          </div>
          
          <div className="result-item">
            <h4>Message Hash:</h4>
            <code>{lastSignature.messageHash}</code>
          </div>
          
          <div className="result-item">
            <h4>Signature:</h4>
            <code>{lastSignature.signature}</code>
          </div>
          
          <button
            onClick={handleVerifySignature}
            disabled={loading.verification || !publicKey}
          >
            {loading.verification ? "Verifying..." : "Verify Signature"}
          </button>
          
          {verificationResult !== null && (
            <div className={`verification-result ${verificationResult ? "valid" : "invalid"}`}>
              Signature is <strong>{verificationResult ? "valid" : "invalid"}</strong>!
            </div>
          )}
        </div>
      )}
    </div>
  );
}
```

## Data Encryption and Decryption

OpenSecret provides methods for client-side encryption and decryption of arbitrary data. For more details, see the [Data Encryption](./data-encryption) guide.

## BIP-85 and BIP-32 Derivation

### Understanding BIP-85

BIP-85 allows deriving child mnemonic phrases from a master mnemonic. Common path format:

```
m/83696968'/39'/0'/12'/0'
```

Where:
- `83696968'` is the application number (ASCII for "BIPS")
- `39'` is for BIP-39 mnemonic derivation
- `0'` is the coin type (Bitcoin)
- `12'` is the number of words (12-word mnemonic)
- `0'` is the index (can increment for different phrases)

### Understanding BIP-32

BIP-32 allows deriving child keys from a master key. Common path formats:

- BIP44 (Legacy): `m/44'/0'/0'/0/0`
- BIP49 (SegWit): `m/49'/0'/0'/0/0`
- BIP84 (Native SegWit): `m/84'/0'/0'/0/0`
- BIP86 (Taproot): `m/86'/0'/0'/0/0`

Where:
- `m/` means absolute path from the master key
- `44'`, `49'`, etc. are purpose identifiers
- `0'` is the coin type (Bitcoin)
- `0'` is the account index
- `0/0` are change/address indices

### Combining BIP-85 and BIP-32

For maximum flexibility, you can combine both derivations:

```tsx
const response = await os.getPrivateKeyBytes({
  seed_phrase_derivation_path: "m/83696968'/39'/0'/12'/0'",
  private_key_derivation_path: "m/44'/0'/0'/0/0"
});
```

This:
1. First derives a child mnemonic using BIP-85
2. Then applies BIP-32 derivation to get a private key from that derived mnemonic

## Best Practices

1. **Avoid unnecessary key exposure**: Only retrieve private keys when absolutely necessary

2. **Use derived keys**: Prefer derived keys for different purposes rather than reusing the master key

3. **Verify signatures client-side**: Perform signature verification in the client to reduce server trust

4. **Store public keys**: Cache public keys rather than regenerating them frequently

5. **Handle errors gracefully**: Cryptographic operations can fail for various reasons, including invalid paths

## Security Considerations

When working with cryptographic operations:

1. **Never expose private keys**: Don't log, transmit, or store private keys unencrypted

2. **Use different keys for different purposes**: Derive different keys for different applications

3. **Consider memory security**: Clear sensitive data from memory when no longer needed

4. **Be aware of browser security**: Consider the security context of your application

## Next Steps

- [Data Encryption](./data-encryption) - Learn how to use encryption for data protection
- [AI Integration](./ai-integration) - Explore AI features with privacy guarantees
- [Remote Attestation](./remote-attestation) - Understand how OpenSecret verifies server security