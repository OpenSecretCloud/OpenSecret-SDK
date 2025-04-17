---
title: Data Encryption
sidebar_position: 6
---

# Data Encryption and Decryption

OpenSecret provides powerful client-side encryption capabilities that enable your application to protect sensitive data with the user's cryptographic keys. This guide explains how to use the encryption and decryption features of the SDK.

## Overview

OpenSecret's data encryption features allow you to:

- Encrypt arbitrary data using the user's cryptographic keys
- Support various key derivation methods (master key, BIP-32, BIP-85)
- Decrypt data previously encrypted with the same keys
- Maintain end-to-end encryption for sensitive information

## Prerequisites

Before using data encryption, make sure:

1. Your application is wrapped with `OpenSecretProvider`
2. The user is authenticated
3. You have imported the `useOpenSecret` hook

## Basic Encryption and Decryption

### Encrypting Data

To encrypt a string:

```tsx
import { useState } from "react";
import { useOpenSecret } from "@opensecret/react";

function EncryptionExample() {
  const os = useOpenSecret();
  const [plaintext, setPlaintext] = useState("");
  const [encryptedData, setEncryptedData] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  
  async function handleEncrypt() {
    if (!plaintext.trim()) {
      setError("Please enter text to encrypt");
      return;
    }
    
    setLoading(true);
    setError("");
    try {
      // Encrypt the data with the user's master key
      const response = await os.encryptData(plaintext);
      setEncryptedData(response.encrypted_data);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Encryption failed");
    } finally {
      setLoading(false);
    }
  }
  
  return (
    <div>
      <h3>Encrypt Data</h3>
      
      <div className="input-group">
        <label htmlFor="plaintext">Text to Encrypt:</label>
        <textarea
          id="plaintext"
          value={plaintext}
          onChange={(e) => setPlaintext(e.target.value)}
          rows={3}
          placeholder="Enter sensitive text to encrypt"
          disabled={loading}
        />
      </div>
      
      <button onClick={handleEncrypt} disabled={loading || !plaintext.trim()}>
        {loading ? "Encrypting..." : "Encrypt"}
      </button>
      
      {error && <div className="error">{error}</div>}
      
      {encryptedData && (
        <div className="result-section">
          <h4>Encrypted Data:</h4>
          <textarea
            readOnly
            value={encryptedData}
            rows={4}
            className="encrypted-output"
          />
          <button
            onClick={() => navigator.clipboard.writeText(encryptedData)}
            className="copy-button"
          >
            Copy to Clipboard
          </button>
        </div>
      )}
    </div>
  );
}
```

### Decrypting Data

To decrypt previously encrypted data:

```tsx
import { useState } from "react";
import { useOpenSecret } from "@opensecret/react";

function DecryptionExample() {
  const os = useOpenSecret();
  const [encryptedData, setEncryptedData] = useState("");
  const [decryptedText, setDecryptedText] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  
  async function handleDecrypt() {
    if (!encryptedData.trim()) {
      setError("Please enter encrypted data to decrypt");
      return;
    }
    
    setLoading(true);
    setError("");
    try {
      // Decrypt the data with the user's master key
      const decrypted = await os.decryptData(encryptedData);
      setDecryptedText(decrypted);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Decryption failed");
    } finally {
      setLoading(false);
    }
  }
  
  return (
    <div>
      <h3>Decrypt Data</h3>
      
      <div className="input-group">
        <label htmlFor="encryptedData">Encrypted Data:</label>
        <textarea
          id="encryptedData"
          value={encryptedData}
          onChange={(e) => setEncryptedData(e.target.value)}
          rows={4}
          placeholder="Paste encrypted data here"
          disabled={loading}
        />
      </div>
      
      <button onClick={handleDecrypt} disabled={loading || !encryptedData.trim()}>
        {loading ? "Decrypting..." : "Decrypt"}
      </button>
      
      {error && <div className="error">{error}</div>}
      
      {decryptedText && (
        <div className="result-section">
          <h4>Decrypted Text:</h4>
          <div className="decrypted-output">
            {decryptedText}
          </div>
        </div>
      )}
    </div>
  );
}
```

## Using Key Derivation Options

Both `encryptData` and `decryptData` support key derivation options, allowing you to use specific keys for different encryption contexts.

### Encrypting with Derived Keys

```tsx
import { useState } from "react";
import { useOpenSecret } from "@opensecret/react";

function DerivedKeyEncryption() {
  const os = useOpenSecret();
  const [plaintext, setPlaintext] = useState("");
  const [derivationPath, setDerivationPath] = useState("m/44'/0'/0'/0/0");
  const [encryptedData, setEncryptedData] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  
  async function handleEncrypt() {
    if (!plaintext.trim()) {
      setError("Please enter text to encrypt");
      return;
    }
    
    setLoading(true);
    setError("");
    try {
      // Encrypt with a derived key
      const response = await os.encryptData(plaintext, {
        private_key_derivation_path: derivationPath
      });
      setEncryptedData(response.encrypted_data);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Encryption failed");
    } finally {
      setLoading(false);
    }
  }
  
  return (
    <div>
      <h3>Derived Key Encryption</h3>
      
      <div className="input-group">
        <label htmlFor="derivationPath">Derivation Path:</label>
        <input
          id="derivationPath"
          type="text"
          value={derivationPath}
          onChange={(e) => setDerivationPath(e.target.value)}
          placeholder="e.g. m/44'/0'/0'/0/0"
          disabled={loading}
        />
        <div className="help-text">
          Must use the same path for decryption!
        </div>
      </div>
      
      <div className="input-group">
        <label htmlFor="plaintextDerived">Text to Encrypt:</label>
        <textarea
          id="plaintextDerived"
          value={plaintext}
          onChange={(e) => setPlaintext(e.target.value)}
          rows={3}
          placeholder="Enter sensitive text to encrypt"
          disabled={loading}
        />
      </div>
      
      <button onClick={handleEncrypt} disabled={loading || !plaintext.trim()}>
        Encrypt with Derived Key
      </button>
      
      {error && <div className="error">{error}</div>}
      
      {encryptedData && (
        <div className="result-section">
          <h4>Encrypted Data:</h4>
          <textarea
            readOnly
            value={encryptedData}
            rows={4}
            className="encrypted-output"
          />
          <div className="warning-text">
            Remember to use the same derivation path for decryption!
          </div>
        </div>
      )}
    </div>
  );
}
```

### BIP-85 Key Derivation Example

You can also use BIP-85 to derive a child mnemonic for encryption:

```tsx
// Using BIP-85 derivation
const { encrypted_data } = await os.encryptData("Secret message", {
  seed_phrase_derivation_path: "m/83696968'/39'/0'/12'/0'"
});

// For decryption, use the same BIP-85 path
const decrypted = await os.decryptData(encrypted_data, {
  seed_phrase_derivation_path: "m/83696968'/39'/0'/12'/0'"
});
```

### Combined BIP-85 and BIP-32 Derivation

For maximum flexibility, you can combine both derivation methods:

```tsx
// Encrypt with combined derivation
const { encrypted_data } = await os.encryptData("Secret message", {
  seed_phrase_derivation_path: "m/83696968'/39'/0'/12'/0'",
  private_key_derivation_path: "m/44'/0'/0'/0/0"
});

// Must use exactly the same derivation paths for decryption
const decrypted = await os.decryptData(encrypted_data, {
  seed_phrase_derivation_path: "m/83696968'/39'/0'/12'/0'",
  private_key_derivation_path: "m/44'/0'/0'/0/0"
});
```

## Complete Encryption/Decryption Example

Here's a complete example that demonstrates encryption and decryption with various key derivation options:

```tsx
import React, { useState } from "react";
import { useOpenSecret } from "@opensecret/react";

type DerivationMethod = "master" | "bip32" | "bip85" | "combined";

function DataEncryptionDemo() {
  const os = useOpenSecret();
  
  const [plaintext, setPlaintext] = useState("");
  const [encryptedData, setEncryptedData] = useState("");
  const [decryptedText, setDecryptedText] = useState("");
  
  const [derivationMethod, setDerivationMethod] = useState<DerivationMethod>("master");
  const [bip32Path, setBip32Path] = useState("m/44'/0'/0'/0/0");
  const [bip85Path, setBip85Path] = useState("m/83696968'/39'/0'/12'/0'");
  
  const [loading, setLoading] = useState({
    encrypt: false,
    decrypt: false
  });
  const [error, setError] = useState("");
  
  function getKeyOptions() {
    switch (derivationMethod) {
      case "master":
        return undefined;
      case "bip32":
        return { private_key_derivation_path: bip32Path };
      case "bip85":
        return { seed_phrase_derivation_path: bip85Path };
      case "combined":
        return {
          seed_phrase_derivation_path: bip85Path,
          private_key_derivation_path: bip32Path
        };
    }
  }
  
  async function handleEncrypt() {
    if (!plaintext.trim()) {
      setError("Please enter text to encrypt");
      return;
    }
    
    setLoading({ ...loading, encrypt: true });
    setError("");
    try {
      const keyOptions = getKeyOptions();
      const response = await os.encryptData(plaintext, keyOptions);
      setEncryptedData(response.encrypted_data);
      setDecryptedText(""); // Clear previous decryption
    } catch (err) {
      setError(err instanceof Error ? err.message : "Encryption failed");
    } finally {
      setLoading({ ...loading, encrypt: false });
    }
  }
  
  async function handleDecrypt() {
    if (!encryptedData.trim()) {
      setError("Please encrypt some data first");
      return;
    }
    
    setLoading({ ...loading, decrypt: true });
    setError("");
    try {
      const keyOptions = getKeyOptions();
      const decrypted = await os.decryptData(encryptedData, keyOptions);
      setDecryptedText(decrypted);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Decryption failed");
    } finally {
      setLoading({ ...loading, decrypt: false });
    }
  }
  
  return (
    <div className="encryption-demo">
      <h2>Data Encryption and Decryption</h2>
      
      {error && <div className="error">{error}</div>}
      
      <div className="section">
        <h3>Key Derivation Settings</h3>
        
        <div className="input-group">
          <label htmlFor="derivationMethod">Derivation Method:</label>
          <select
            id="derivationMethod"
            value={derivationMethod}
            onChange={(e) => setDerivationMethod(e.target.value as DerivationMethod)}
            disabled={loading.encrypt || loading.decrypt}
          >
            <option value="master">Master Key (No Derivation)</option>
            <option value="bip32">BIP-32 Derivation</option>
            <option value="bip85">BIP-85 Derivation</option>
            <option value="combined">Combined BIP-85 + BIP-32</option>
          </select>
        </div>
        
        {(derivationMethod === "bip32" || derivationMethod === "combined") && (
          <div className="input-group">
            <label htmlFor="bip32Path">BIP-32 Derivation Path:</label>
            <input
              id="bip32Path"
              type="text"
              value={bip32Path}
              onChange={(e) => setBip32Path(e.target.value)}
              placeholder="e.g. m/44'/0'/0'/0/0"
              disabled={loading.encrypt || loading.decrypt}
            />
          </div>
        )}
        
        {(derivationMethod === "bip85" || derivationMethod === "combined") && (
          <div className="input-group">
            <label htmlFor="bip85Path">BIP-85 Derivation Path:</label>
            <input
              id="bip85Path"
              type="text"
              value={bip85Path}
              onChange={(e) => setBip85Path(e.target.value)}
              placeholder="e.g. m/83696968'/39'/0'/12'/0'"
              disabled={loading.encrypt || loading.decrypt}
            />
          </div>
        )}
        
        <div className="info-box">
          <h4>Derivation Method Details:</h4>
          <ul>
            <li><strong>Master Key</strong>: Uses the main private key for encryption</li>
            <li><strong>BIP-32</strong>: Derives a child key from the master key</li>
            <li><strong>BIP-85</strong>: Derives a new mnemonic, then uses its master key</li>
            <li><strong>Combined</strong>: Derives a new mnemonic via BIP-85, then applies BIP-32</li>
          </ul>
          <p className="warning-text">
            Important: You must use the same derivation method and paths for both encryption and decryption!
          </p>
        </div>
      </div>
      
      <div className="section">
        <h3>Encryption</h3>
        
        <div className="input-group">
          <label htmlFor="plaintext">Text to Encrypt:</label>
          <textarea
            id="plaintext"
            value={plaintext}
            onChange={(e) => setPlaintext(e.target.value)}
            rows={4}
            placeholder="Enter sensitive data to encrypt"
            disabled={loading.encrypt}
          />
        </div>
        
        <button
          onClick={handleEncrypt}
          disabled={loading.encrypt || !plaintext.trim()}
        >
          {loading.encrypt ? "Encrypting..." : "Encrypt Data"}
        </button>
      </div>
      
      {encryptedData && (
        <div className="section result-section">
          <h3>Encrypted Result:</h3>
          <textarea
            readOnly
            value={encryptedData}
            rows={4}
            className="encrypted-output"
          />
          <div className="button-group">
            <button
              onClick={() => navigator.clipboard.writeText(encryptedData)}
              className="copy-button"
            >
              Copy to Clipboard
            </button>
            <button
              onClick={handleDecrypt}
              disabled={loading.decrypt}
            >
              {loading.decrypt ? "Decrypting..." : "Decrypt Data"}
            </button>
          </div>
        </div>
      )}
      
      {decryptedText && (
        <div className="section result-section">
          <h3>Decrypted Result:</h3>
          <div className="decrypted-output">
            {decryptedText}
          </div>
          <div className="success-message">
            Successfully decrypted with the {derivationMethod} key!
          </div>
        </div>
      )}
    </div>
  );
}
```

## Practical Use Cases

### Storing Encrypted User Data

Combine encryption with the key-value storage API for enhanced security:

```tsx
import { useOpenSecret } from "@opensecret/react";

async function storeEncryptedProfile(profileData) {
  const os = useOpenSecret();
  
  // Convert data to string
  const profileString = JSON.stringify(profileData);
  
  // First, encrypt the data
  const { encrypted_data } = await os.encryptData(profileString, {
    // Use a dedicated derivation path for profile data
    private_key_derivation_path: "m/44'/0'/0'/0/1"
  });
  
  // Then store the encrypted data
  await os.put("user_profile_encrypted", encrypted_data);
  
  return true;
}

async function getEncryptedProfile() {
  const os = useOpenSecret();
  
  // Get the encrypted data
  const encryptedData = await os.get("user_profile_encrypted");
  if (!encryptedData) return null;
  
  // Decrypt with the same derivation path used for encryption
  const decrypted = await os.decryptData(encryptedData, {
    private_key_derivation_path: "m/44'/0'/0'/0/1"
  });
  
  // Parse the JSON data
  return JSON.parse(decrypted);
}
```

### Client-Side Encrypting Form Data

Encrypt sensitive form data before submission:

```tsx
import React, { useState } from "react";
import { useOpenSecret } from "@opensecret/react";

function SecureFormExample() {
  const os = useOpenSecret();
  const [formData, setFormData] = useState({
    name: "",
    email: "",
    ssn: "", // Sensitive data
    dob: ""  // Sensitive data
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [success, setSuccess] = useState(false);
  
  function handleChange(e) {
    const { name, value } = e.target;
    setFormData({ ...formData, [name]: value });
  }
  
  async function handleSubmit(e) {
    e.preventDefault();
    setLoading(true);
    setError("");
    setSuccess(false);
    
    try {
      // Extract sensitive fields for encryption
      const { name, email, ssn, dob } = formData;
      const sensitiveData = JSON.stringify({ ssn, dob });
      
      // Encrypt sensitive data
      const { encrypted_data } = await os.encryptData(sensitiveData);
      
      // Create submission with encrypted data
      const submission = {
        name,
        email,
        encrypted_sensitive_data: encrypted_data
      };
      
      // Submit to your API (just a mock example)
      // await fetch('/api/submit-form', {
      //   method: 'POST',
      //   headers: { 'Content-Type': 'application/json' },
      //   body: JSON.stringify(submission)
      // });
      
      console.log("Submission with encrypted data:", submission);
      setSuccess(true);
      
      // Clear form
      setFormData({
        name: "",
        email: "",
        ssn: "",
        dob: ""
      });
    } catch (err) {
      setError(err instanceof Error ? err.message : "Form submission failed");
    } finally {
      setLoading(false);
    }
  }
  
  return (
    <div className="secure-form">
      <h3>Secure Form with Encryption</h3>
      
      {error && <div className="error">{error}</div>}
      {success && <div className="success">Form submitted successfully!</div>}
      
      <form onSubmit={handleSubmit}>
        <div className="form-group">
          <label htmlFor="name">Name:</label>
          <input
            id="name"
            type="text"
            name="name"
            value={formData.name}
            onChange={handleChange}
            required
            disabled={loading}
          />
        </div>
        
        <div className="form-group">
          <label htmlFor="email">Email:</label>
          <input
            id="email"
            type="email"
            name="email"
            value={formData.email}
            onChange={handleChange}
            required
            disabled={loading}
          />
        </div>
        
        <div className="form-group secure-field">
          <label htmlFor="ssn">Social Security Number:</label>
          <input
            id="ssn"
            type="text"
            name="ssn"
            value={formData.ssn}
            onChange={handleChange}
            pattern="[0-9]{3}-[0-9]{2}-[0-9]{4}"
            placeholder="XXX-XX-XXXX"
            required
            disabled={loading}
          />
          <div className="secure-badge">
            🔒 Will be encrypted
          </div>
        </div>
        
        <div className="form-group secure-field">
          <label htmlFor="dob">Date of Birth:</label>
          <input
            id="dob"
            type="date"
            name="dob"
            value={formData.dob}
            onChange={handleChange}
            required
            disabled={loading}
          />
          <div className="secure-badge">
            🔒 Will be encrypted
          </div>
        </div>
        
        <button type="submit" disabled={loading}>
          {loading ? "Submitting..." : "Submit Securely"}
        </button>
      </form>
    </div>
  );
}
```

## Technical Details

### How the Encryption Works

OpenSecret uses a combination of:

1. **AES-256-GCM** for symmetric encryption
2. **Random nonces** for each encryption operation
3. **Authenticated encryption** to prevent tampering
4. **Base64 encoding** for the final output

The encrypted data format includes metadata like nonces and is base64-encoded for safe storage and transmission.

### Security Considerations

1. **Different keys produce different ciphertexts**: Data encrypted with one key cannot be decrypted with another

2. **Key derivation is critical**: You must use exactly the same derivation options for encryption and decryption

3. **Metadata is not encrypted**: The fact that encryption happened is not hidden, only the content is protected

4. **Reusing encryption keys**: Using the same key for different types of data may have security implications

## Best Practices

1. **Use dedicated derivation paths**: Create separate derivation paths for different types of sensitive data

2. **Document your derivation paths**: Keep track of which paths you use for which data types

3. **Validate data after decryption**: Always validate structure and content after decryption

4. **Implement proper error handling**: Encryption/decryption failures should be handled gracefully

5. **Consider key rotation**: For long-term data, have a strategy for rotating encryption keys

## Next Steps

- [Cryptographic Operations](./cryptographic-operations) - Learn more about key management and signing
- [AI Integration](./ai-integration) - Explore OpenSecret's privacy-preserving AI features
- [Remote Attestation](./remote-attestation) - Understand how OpenSecret verifies server security