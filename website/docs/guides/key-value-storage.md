---
title: Key-Value Storage
sidebar_position: 4
---

# Key-Value Storage

OpenSecret provides a secure, encrypted key-value storage system that allows you to store and retrieve data for authenticated users. This guide explains how to use the key-value storage API.

## Overview

The OpenSecret key-value storage:

- Is automatically encrypted at rest in secure enclaves
- Is accessible only to authenticated users
- Provides simple CRUD operations
- Preserves value data types but stores them as strings

## Prerequisites

Before using key-value storage, make sure:

1. Your application is wrapped with `OpenSecretProvider`
2. The user is authenticated
3. You have imported the `useOpenSecret` hook

## Basic Operations

### Storing a Value

Use the `put` method to store a value:

```tsx
import { useOpenSecret } from "@opensecret/react";

function StorageExample() {
  const os = useOpenSecret();
  
  async function storeValue() {
    try {
      await os.put("user_preferences", JSON.stringify({
        theme: "dark",
        notifications: true,
        language: "en"
      }));
      alert("Preferences saved!");
    } catch (error) {
      console.error("Failed to store value:", error);
    }
  }
  
  return <button onClick={storeValue}>Save Preferences</button>;
}
```

### Retrieving a Value

Use the `get` method to retrieve a value:

```tsx
import { useState } from "react";
import { useOpenSecret } from "@opensecret/react";

function RetrieveExample() {
  const os = useOpenSecret();
  const [preferences, setPreferences] = useState(null);
  const [loading, setLoading] = useState(false);
  
  async function loadPreferences() {
    setLoading(true);
    try {
      const data = await os.get("user_preferences");
      if (data) {
        setPreferences(JSON.parse(data));
      } else {
        setPreferences(null);
      }
    } catch (error) {
      console.error("Failed to get value:", error);
    } finally {
      setLoading(false);
    }
  }
  
  return (
    <div>
      <button onClick={loadPreferences} disabled={loading}>
        {loading ? "Loading..." : "Load Preferences"}
      </button>
      
      {preferences && (
        <div>
          <h3>User Preferences</h3>
          <p>Theme: {preferences.theme}</p>
          <p>Notifications: {preferences.notifications ? "Enabled" : "Disabled"}</p>
          <p>Language: {preferences.language}</p>
        </div>
      )}
    </div>
  );
}
```

### Listing All Key-Value Pairs

Use the `list` method to retrieve all key-value pairs:

```tsx
import { useState } from "react";
import { useOpenSecret } from "@opensecret/react";
import type { KVListItem } from "@opensecret/react";

function ListStorageExample() {
  const os = useOpenSecret();
  const [items, setItems] = useState<KVListItem[]>([]);
  const [loading, setLoading] = useState(false);
  
  async function loadAllItems() {
    setLoading(true);
    try {
      const data = await os.list();
      setItems(data);
    } catch (error) {
      console.error("Failed to list items:", error);
    } finally {
      setLoading(false);
    }
  }
  
  return (
    <div>
      <button onClick={loadAllItems} disabled={loading}>
        {loading ? "Loading..." : "List All Items"}
      </button>
      
      {items.length > 0 ? (
        <div>
          <h3>Stored Items ({items.length})</h3>
          <table>
            <thead>
              <tr>
                <th>Key</th>
                <th>Value</th>
                <th>Created</th>
                <th>Updated</th>
              </tr>
            </thead>
            <tbody>
              {items.map((item) => (
                <tr key={item.key}>
                  <td>{item.key}</td>
                  <td>{item.value.length > 20 ? `${item.value.substring(0, 20)}...` : item.value}</td>
                  <td>{new Date(item.created_at).toLocaleString()}</td>
                  <td>{new Date(item.updated_at).toLocaleString()}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : (
        <p>No items found. Try storing some data first.</p>
      )}
    </div>
  );
}
```

### Deleting a Value

Use the `del` method to delete a value:

```tsx
import { useOpenSecret } from "@opensecret/react";

function DeleteExample() {
  const os = useOpenSecret();
  
  async function deletePreferences() {
    try {
      await os.del("user_preferences");
      alert("Preferences deleted!");
    } catch (error) {
      console.error("Failed to delete value:", error);
    }
  }
  
  return <button onClick={deletePreferences}>Delete Preferences</button>;
}
```

## Complete CRUD Example

Here's a complete example that demonstrates all CRUD operations:

```tsx
import React, { useState, useEffect } from "react";
import { useOpenSecret } from "@opensecret/react";
import type { KVListItem } from "@opensecret/react";

function StorageManager() {
  const os = useOpenSecret();
  const [key, setKey] = useState("");
  const [value, setValue] = useState("");
  const [items, setItems] = useState<KVListItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState({ text: "", type: "" });

  useEffect(() => {
    // Load initial data if user is authenticated
    if (os.auth.user && !os.auth.loading) {
      loadItems();
    }
  }, [os.auth.user, os.auth.loading]);

  async function loadItems() {
    setLoading(true);
    try {
      const data = await os.list();
      setItems(data);
    } catch (error) {
      showMessage(`Error: ${error instanceof Error ? error.message : "Failed to load items"}`, "error");
    } finally {
      setLoading(false);
    }
  }

  async function handleStore(e: React.FormEvent) {
    e.preventDefault();
    if (!key.trim() || !value.trim()) {
      showMessage("Both key and value are required", "error");
      return;
    }

    setLoading(true);
    try {
      await os.put(key, value);
      showMessage(`Value stored for key: ${key}`, "success");
      await loadItems();
      // Clear form after success
      setKey("");
      setValue("");
    } catch (error) {
      showMessage(`Error: ${error instanceof Error ? error.message : "Failed to store value"}`, "error");
    } finally {
      setLoading(false);
    }
  }

  async function handleGet(retrieveKey: string) {
    setLoading(true);
    try {
      const data = await os.get(retrieveKey);
      if (data !== undefined) {
        // Pre-fill the form with the retrieved data
        setKey(retrieveKey);
        setValue(data);
        showMessage(`Retrieved value for key: ${retrieveKey}`, "success");
      } else {
        showMessage(`No value found for key: ${retrieveKey}`, "warning");
      }
    } catch (error) {
      showMessage(`Error: ${error instanceof Error ? error.message : "Failed to get value"}`, "error");
    } finally {
      setLoading(false);
    }
  }

  async function handleDelete(deleteKey: string) {
    if (!confirm(`Are you sure you want to delete the key: ${deleteKey}?`)) {
      return;
    }

    setLoading(true);
    try {
      await os.del(deleteKey);
      showMessage(`Deleted key: ${deleteKey}`, "success");
      // If we're editing this key, clear the form
      if (key === deleteKey) {
        setKey("");
        setValue("");
      }
      await loadItems();
    } catch (error) {
      showMessage(`Error: ${error instanceof Error ? error.message : "Failed to delete key"}`, "error");
    } finally {
      setLoading(false);
    }
  }

  function showMessage(text: string, type: string) {
    setMessage({ text, type });
    // Auto-clear success and warning messages after 3 seconds
    if (type !== "error") {
      setTimeout(() => setMessage({ text: "", type: "" }), 3000);
    }
  }

  // Check if user is authenticated
  if (!os.auth.user && !os.auth.loading) {
    return <p>Please log in to use storage features.</p>;
  }

  return (
    <div className="storage-manager">
      <h2>Storage Manager</h2>
      
      {message.text && (
        <div className={`message ${message.type}`}>
          {message.text}
          <button onClick={() => setMessage({ text: "", type: "" })}>×</button>
        </div>
      )}
      
      <form onSubmit={handleStore}>
        <div>
          <label htmlFor="key">Key:</label>
          <input
            id="key"
            type="text"
            value={key}
            onChange={(e) => setKey(e.target.value)}
            required
            disabled={loading}
          />
        </div>
        <div>
          <label htmlFor="value">Value:</label>
          <textarea
            id="value"
            value={value}
            onChange={(e) => setValue(e.target.value)}
            rows={4}
            required
            disabled={loading}
          />
        </div>
        <button type="submit" disabled={loading}>
          {loading ? "Processing..." : "Store Value"}
        </button>
        <button type="button" onClick={loadItems} disabled={loading}>
          Refresh List
        </button>
      </form>
      
      <h3>Stored Items</h3>
      {loading ? (
        <p>Loading...</p>
      ) : items.length > 0 ? (
        <table className="storage-table">
          <thead>
            <tr>
              <th>Key</th>
              <th>Value</th>
              <th>Last Updated</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {items.map((item) => (
              <tr key={item.key}>
                <td>{item.key}</td>
                <td>
                  {item.value.length > 40 
                    ? `${item.value.substring(0, 40)}...` 
                    : item.value
                  }
                </td>
                <td>{new Date(item.updated_at).toLocaleString()}</td>
                <td>
                  <button onClick={() => handleGet(item.key)}>Edit</button>
                  <button onClick={() => handleDelete(item.key)}>Delete</button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      ) : (
        <p>No items stored yet.</p>
      )}
    </div>
  );
}
```

## Best Practices

### Handling Complex Data Types

Since the key-value storage accepts only string values, you'll need to serialize complex data types:

```tsx
// Storing objects
await os.put("userProfile", JSON.stringify({
  name: "John Doe",
  age: 30,
  preferences: { theme: "dark", language: "en" }
}));

// Retrieving and parsing objects
const profileStr = await os.get("userProfile");
const profile = profileStr ? JSON.parse(profileStr) : null;
```

### Error Handling

Always implement proper error handling for storage operations:

```tsx
try {
  await os.put("key", "value");
  // Success case
} catch (error) {
  // Check for specific errors
  if (error instanceof Error) {
    if (error.message.includes("unauthorized")) {
      // Handle authentication errors
    } else if (error.message.includes("network")) {
      // Handle network errors
    } else {
      // Handle other errors
    }
  }
}
```

### Data Validation

Validate data before storing and after retrieving:

```tsx
function storeUserPreferences(preferences) {
  // Validate required fields
  if (!preferences.theme || !preferences.language) {
    throw new Error("Missing required preference fields");
  }
  
  // Validate field types
  if (typeof preferences.notifications !== "boolean") {
    throw new Error("notifications must be a boolean");
  }
  
  // Store the valid data
  return os.put("preferences", JSON.stringify(preferences));
}

async function getUserPreferences() {
  const data = await os.get("preferences");
  if (!data) return null;
  
  try {
    const preferences = JSON.parse(data);
    
    // Validate structure after parsing
    if (!preferences.theme || !preferences.language) {
      console.warn("Retrieved preferences are missing fields");
    }
    
    return preferences;
  } catch (e) {
    console.error("Failed to parse preferences", e);
    return null;
  }
}
```

### Performance Considerations

For better performance:

1. **Batch operations** where possible
2. **Cache frequently accessed data** client-side
3. **Store related data together** to minimize API calls
4. **Implement optimistic updates** for responsive UI

## Security Considerations

The OpenSecret key-value storage provides several security benefits:

1. **Data encryption** at rest and in transit
2. **Authentication-based access control**
3. **Hardware-protected storage** in secure enclaves
4. **Remote attestation** to verify the security of the server

However, be aware of these security considerations:

1. **Client-side security** is your responsibility - don't expose sensitive data in your UI
2. **Keys are not encrypted** - only values are encrypted, so don't put sensitive information in your keys
3. **Be careful with JSON serialization** - ensure you don't accidentally expose sensitive data in error logs

## Advanced Usage

### Using Key-Value Storage with Cryptographic Operations

For additional security, you can encrypt data client-side before storing it:

```tsx
import { useOpenSecret } from "@opensecret/react";

async function storeEncryptedData(plaintext) {
  const os = useOpenSecret();
  
  // Encrypt the data with the user's key
  const { encrypted_data } = await os.encryptData(plaintext);
  
  // Store the encrypted data
  await os.put("encrypted_secret", encrypted_data);
}

async function retrieveEncryptedData() {
  const os = useOpenSecret();
  
  // Get the encrypted data
  const encrypted = await os.get("encrypted_secret");
  if (!encrypted) return null;
  
  // Decrypt the data with the user's key
  const decrypted = await os.decryptData(encrypted);
  return decrypted;
}
```

### Namespacing Keys

For more organized storage, consider using a namespacing convention for keys:

```tsx
// User preferences
await os.put("prefs:theme", "dark");
await os.put("prefs:language", "en");

// App data
await os.put("app:last_viewed", "dashboard");
await os.put("app:notification_count", "5");

// Retrieve all preferences
const allItems = await os.list();
const prefItems = allItems.filter(item => item.key.startsWith("prefs:"));
```

## Next Steps

- [Data Encryption](./data-encryption) - Learn how to use encryption for additional security
- [Cryptographic Operations](./cryptographic-operations) - Explore advanced cryptographic features
- [AI Integration](./ai-integration) - See how to integrate AI with your secure data