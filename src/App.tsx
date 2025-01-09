import "./App.css";
import { useState } from "react";
import React from "react";
import { useOpenSecret } from "./lib";
import type { KVListItem } from "./lib";
import { schnorr, secp256k1 } from '@noble/curves/secp256k1';
import { AI } from "./AI";

function TokenGenerator() {
  const [audience, setAudience] = useState("http://localhost:3001");
  const [token, setToken] = useState("");
  const [error, setError] = useState("");
  const os = useOpenSecret();

  const generateToken = async () => {
    try {
      setError("");
      const response = await os.generateThirdPartyToken(audience);
      setToken(response.token);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to generate token");
    }
  };

  return (
    <div>
      <div className="auth-form">
        <input
          type="url"
          value={audience}
          onChange={(e) => setAudience(e.target.value)}
          placeholder="Enter audience URL"
        />
        <button onClick={generateToken}>Generate Token</button>
      </div>
      {error && (
        <div className="api-message" style={{ color: "red" }}>
          {error}
        </div>
      )}
      {token && (
        <div className="data-display" style={{ marginTop: "1rem" }}>
          <h4>Generated Token:</h4>
          <textarea 
            readOnly 
            value={token} 
            rows={4}
            className="json-input"
            style={{ width: "100%" }}
          />
          <button 
            onClick={() => navigator.clipboard.writeText(token)}
            style={{ marginTop: "0.5rem" }}
          >
            Copy to Clipboard
          </button>
        </div>
      )}
    </div>
  );
}

function App() {
  const os = useOpenSecret();
  const [listData, setListData] = useState<KVListItem[]>([]);
  const [algorithm, setAlgorithm] = useState<"schnorr" | "ecdsa">("schnorr");
  const [publicKey, setPublicKey] = useState<string>("");
  const [lastSignature, setLastSignature] = useState<{
    signature: string;
    messageHash: string;
    message: string;
  } | null>(null);
  const [verificationResult, setVerificationResult] = useState<boolean | null>(null);
  const [derivationPath, setDerivationPath] = useState<string>("");

  const handleSignup = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    const name = formData.get("name") as string;
    const email = formData.get("email") as string;
    const password = formData.get("password") as string;
    const inviteCode = formData.get("inviteCode") as string;

    console.log("Signup request:", { name, email, password, inviteCode });

    try {
      const response = await os.signUp(email, password, inviteCode, name);
      console.log("Signup response:", response);
      alert("Signup successful!");
    } catch (error) {
      console.error("Signup error:", error);
      alert("Signup failed: " + (error as Error).message);
    }
  };

  const handleGuestSignup = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    const password = formData.get("password") as string;
    const inviteCode = formData.get("inviteCode") as string;

    console.log("Guest Signup request:", { password, inviteCode });

    try {
      const response = await os.signUpGuest(password, inviteCode);
      console.log("Guest Signup response:", response);
      alert("Guest Signup successful! Your ID is: " + response.id);
    } catch (error) {
      console.error("Guest Signup error:", error);
      alert("Guest Signup failed: " + (error as Error).message);
    }
  };

  const handleGuestLogin = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    const id = formData.get("id") as string;
    const password = formData.get("password") as string;

    console.log("Guest Login request:", { id, password });

    try {
      const response = await os.signInGuest(id, password);
      console.log("Guest Login response:", response);
      alert("Guest Login successful!");
    } catch (error) {
      console.error("Guest Login error:", error);
      alert("Guest Login failed: " + (error as Error).message);
    }
  };

  const handleGuestConversion = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    const email = formData.get("email") as string;
    const password = formData.get("password") as string;
    const name = formData.get("name") as string;

    console.log("Guest Conversion request:", { email, password, name });

    try {
      await os.convertGuestToUserAccount(email, password, name);
      console.log("Guest Conversion successful");
      alert("Account successfully converted to email login!");
    } catch (error) {
      console.error("Guest Conversion error:", error);
      alert("Guest Conversion failed: " + (error as Error).message);
    }
  };

  const handleLogin = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    const email = formData.get("email") as string;
    const password = formData.get("password") as string;

    console.log("Login request:", { email, password });

    try {
      const response = await os.signIn(email, password);
      console.log("Login response:", response);
      alert("Login successful!");
    } catch (error) {
      console.error("Login error:", error);
      alert("Login failed: " + (error as Error).message);
    }
  };

  const handleGet = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    const key = formData.get("key") as string;

    console.log("Get request for key:", key);

    try {
      const data = await os.get(key);
      console.log("Get response:", data);
      alert(`Data for key "${key}": ${data}`);
    } catch (error) {
      console.error("Get error:", error);
      alert("Get failed: " + (error as Error).message);
    }
  };

  const handlePut = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    const key = formData.get("key") as string;
    const value = formData.get("value") as string;

    console.log("Put request:", { key, value });

    try {
      const response = await os.put(key, value);
      console.log("Put response:", response);
      alert("Put successful!");
    } catch (error) {
      console.error("Put error:", error);
      alert("Put failed: " + (error as Error).message);
    }
  };

  const handleList = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    console.log("List request");

    try {
      const data = await os.list();
      console.log("List response:", data);
      setListData(data);
    } catch (error) {
      console.error("List error:", error);
      alert("List failed: " + (error as Error).message);
    }
  };

  const handleLogout = async () => {
    console.log("Logout request");

    try {
      await os.signOut();
      console.log("Logout successful");
    } catch (error) {
      console.error("Logout error:", error);
      alert("Logout failed: " + (error as Error).message);
    }
  };

  const handleDelete = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    const key = formData.get("key") as string;

    console.log("Delete request for key:", key);

    try {
      await os.del(key);
      console.log("Delete successful");
      alert(`Key "${key}" deleted successfully`);
    } catch (error) {
      console.error("Delete error:", error);
      alert("Delete failed: " + (error as Error).message);
    }
  };

  const handleGetPublicKey = async () => {
    try {
      const response = await os.getPublicKey(algorithm);
      setPublicKey(response.public_key);
      setVerificationResult(null); // Reset verification when changing keys
    } catch (error) {
      console.error("Failed to get public key:", error);
      alert("Failed to get public key: " + (error as Error).message);
    }
  };

  const handleSignMessage = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    const message = formData.get("message") as string;
    
    try {
      const messageBytes = new TextEncoder().encode(message);
      const response = await os.signMessage(messageBytes, algorithm, derivationPath || undefined);
      
      setLastSignature({
        signature: response.signature,
        messageHash: response.message_hash,
        message
      });
      setVerificationResult(null); // Reset verification on new signature
    } catch (error) {
      console.error("Failed to sign message:", error);
      alert("Failed to sign message: " + (error as Error).message);
    }
  };

  const handleVerifySignature = async () => {
    if (!lastSignature || !publicKey) {
      alert("Please get public key and sign a message first");
      return;
    }

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
    } catch (error) {
      console.error("Verification error:", error);
      alert("Verification failed: " + (error as Error).message);
    }
  };

  return (
    <main>
      <section>
        <h2>Current User</h2>
        <p>Displays the current authenticated user's data from the auth state.</p>
        <div className="data-display">
          {os.auth.loading ? (
            "Loading..."
          ) : os.auth.user ? (
            <pre>{JSON.stringify(os.auth.user, null, 2)}</pre>
          ) : (
            "No user authenticated"
          )}
        </div>
        {os.auth.user && (
          <div className="auth-form" style={{ marginTop: "1rem", display: "flex", gap: "0.5rem" }}>
            <button onClick={handleLogout}>Logout</button>
            <button onClick={() => os.refetchUser()}>Refetch User</button>
          </div>
        )}
      </section>

      <section>
        <h2>Login</h2>
        <p>Sign in to your existing account with email and password.</p>
        <form onSubmit={handleLogin} className="auth-form">
          <input type="email" name="email" placeholder="Email" autoComplete="email" required />
          <input
            type="password"
            name="password"
            placeholder="Password"
            autoComplete="current-password"
            minLength={8}
            required
          />
          <button type="submit">Login</button>
        </form>
      </section>

      <section>
        <h2>Sign Up</h2>
        <p>Create a new account with name, email, password, and invite code.</p>
        <form onSubmit={handleSignup} className="auth-form">
          <input type="text" name="name" placeholder="Name" autoComplete="name" required />
          <input type="email" name="email" placeholder="Email" autoComplete="email" required />
          <input
            type="password"
            name="password"
            placeholder="Password"
            autoComplete="new-password"
            minLength={8}
            required
          />
          <input type="text" name="inviteCode" placeholder="Invite Code" required />
          <button type="submit">Sign Up</button>
        </form>
      </section>

      <section>
        <h2>Guest Sign Up</h2>
        <p>Create a new guest account with just a password and invite code.</p>
        <form onSubmit={handleGuestSignup} className="auth-form">
          <input
            type="password"
            name="password"
            placeholder="Password"
            autoComplete="new-password"
            minLength={8}
            required
          />
          <input type="text" name="inviteCode" placeholder="Invite Code" required />
          <button type="submit">Sign Up as Guest</button>
        </form>
      </section>

      <section>
        <h2>Guest Login</h2>
        <p>Sign in to your guest account with ID and password.</p>
        <form onSubmit={handleGuestLogin} className="auth-form">
          <input type="text" name="id" placeholder="Guest ID" required />
          <input
            type="password"
            name="password"
            placeholder="Password"
            autoComplete="current-password"
            minLength={8}
            required
          />
          <button type="submit">Login as Guest</button>
        </form>
      </section>

      {(os.auth.user?.user && (os.auth.user.user.email === undefined || os.auth.user.user.email === null)) && (
        <section>
          <h2>Convert Guest Account</h2>
          <p>Convert your guest account to a regular account with email.</p>
          <form onSubmit={handleGuestConversion} className="auth-form">
            <input type="email" name="email" placeholder="Email" autoComplete="email" required />
            <input type="text" name="name" placeholder="Name (optional)" autoComplete="name" />
            <input
              type="password"
              name="password"
              placeholder="New Password"
              autoComplete="new-password"
              minLength={8}
              required
            />
            <button type="submit">Convert to Email Account</button>
          </form>
        </section>
      )}

      <section>
        <h2>Get Data</h2>
        <p>Retrieve string data by key.</p>
        <form onSubmit={handleGet} className="auth-form">
          <input type="text" name="key" placeholder="Enter key" />
          <button type="submit">Get Data</button>
        </form>
      </section>

      <section>
        <h2>Put Data</h2>
        <p>Store string data with a key.</p>
        <form onSubmit={handlePut} className="auth-form">
          <input type="text" name="key" placeholder="Enter key" />
          <textarea name="value" placeholder="Enter string value" rows={4} className="json-input" />
          <button type="submit">Put Data</button>
        </form>
      </section>

      <section>
        <h2>List All</h2>
        <p>Retrieve all available keys/values in your storage.</p>
        <form onSubmit={handleList} className="auth-form">
          <button type="submit">List All</button>
        </form>
        {listData.length > 0 && (
          <div className="data-display">
            <div className="grid-container">
              <div className="grid-header">Key</div>
              <div className="grid-header">Value</div>
              <div className="grid-header">Created At</div>
              <div className="grid-header">Updated At</div>
              {listData.map((item, index) => (
                <React.Fragment key={index}>
                  <div className="grid-item">{item.key}</div>
                  <div className="grid-item value">{item.value}</div>
                  <div className="grid-item timestamp">
                    {new Date(item.created_at).toLocaleString()}
                  </div>
                  <div className="grid-item timestamp">
                    {new Date(item.updated_at).toLocaleString()}
                  </div>
                </React.Fragment>
              ))}
            </div>
          </div>
        )}
      </section>

      <section>
        <h2>Delete Data</h2>
        <p>Remove a key-value pair from storage.</p>
        <form onSubmit={handleDelete} className="auth-form">
          <input type="text" name="key" placeholder="Enter key to delete" />
          <button type="submit">Delete Data</button>
        </form>
      </section>

      <section>
        <h2>Private Key</h2>
        <p>Retrieve your private key mnemonic phrase. Keep this secure and never share it.</p>
        
        <button 
          onClick={async () => {
            try {
              const response = await os.getPrivateKey();
              alert(`Your mnemonic phrase is:\n\n${response.mnemonic}\n\nPlease store this securely and never share it with anyone.`);
            } catch (error) {
              console.error("Failed to get private key:", error);
              alert("Failed to get private key: " + (error as Error).message);
            }
          }}
          style={{ marginBottom: "1rem" }}
        >
          Show Private Key Mnemonic
        </button>

        <div>
          <h3>Private Key Bytes</h3>
          <p>Get private key bytes for a specific BIP32 derivation path.</p>
          <details>
            <summary>Common derivation paths</summary>
            <ul>
              <li>BIP44 (Legacy): m/44'/0'/0'/0/0</li>
              <li>BIP49 (SegWit): m/49'/0'/0'/0/0</li>
              <li>BIP84 (Native SegWit): m/84'/0'/0'/0/0</li>
              <li>BIP86 (Taproot): m/86'/0'/0'/0/0</li>
            </ul>
            <p><small>
              Note: Supports both absolute (starting with "m/") and relative paths.
              Supports hardened derivation using either ' or h notation.
            </small></p>
            <p><small>
              Examples:
              <ul>
                <li>Absolute path: "m/44'/0'/0'/0/0"</li>
                <li>Relative path: "0'/0'/0'/0/0"</li>
                <li>Hardened notation: "44'" or "44h"</li>
              </ul>
            </small></p>
          </details>
          <form onSubmit={async (e) => {
            e.preventDefault();
            const formData = new FormData(e.currentTarget);
            const derivationPath = formData.get("derivationPath") as string;
            
            try {
              const response = await os.getPrivateKeyBytes(derivationPath || undefined);
              alert(`Private key bytes (hex):\n\n${response.private_key}`);
            } catch (error) {
              console.error("Failed to get private key bytes:", error);
              alert("Failed to get private key bytes: " + (error as Error).message);
            }
          }} className="auth-form">
            <input 
              type="text" 
              name="derivationPath" 
              placeholder="Derivation path (e.g. m/44'/0'/0'/0/0)" 
            />
            <button type="submit">Get Private Key Bytes</button>
          </form>
        </div>
      </section>

      <section>
        <h2>Cryptographic Signing</h2>
        <p>Demonstrate message signing and verification using your key pair.</p>
        
        <div style={{ marginBottom: "1rem" }}>
          <label>
            Algorithm:
            <select 
              value={algorithm} 
              onChange={(e) => setAlgorithm(e.target.value as "schnorr" | "ecdsa")}
              style={{ marginLeft: "0.5rem" }}
            >
              <option value="schnorr">Schnorr</option>
              <option value="ecdsa">ECDSA</option>
            </select>
          </label>
        </div>

        <div style={{ marginBottom: "1rem" }}>
          <details style={{ marginBottom: "0.5rem" }}>
            <summary>About derivation paths</summary>
            <p><small>
              The derivation path determines which private key is used for signing.
              Different paths generate different key pairs from the same master key,
              useful for separating keys by purpose or application.
            </small></p>
            <p><small>Common paths and their uses:</small></p>
            <ul>
              <li><small>BIP44 (Legacy): m/44'/0'/0'/0/0 - For legacy Bitcoin addresses</small></li>
              <li><small>BIP49 (SegWit): m/49'/0'/0'/0/0 - For SegWit addresses</small></li>
              <li><small>BIP84 (Native SegWit): m/84'/0'/0'/0/0 - For Native SegWit</small></li>
              <li><small>BIP86 (Taproot): m/86'/0'/0'/0/0 - For Taproot addresses</small></li>
            </ul>
            <p><small>
              Path format examples:
              <ul>
                <li>Absolute: "m/44'/0'/0'/0/0"</li>
                <li>Relative: "0'/0'/0'/0/0"</li>
                <li>Hardened: "44'" or "44h"</li>
              </ul>
              Leave empty to use the master key.
            </small></p>
          </details>
          <input 
            type="text" 
            value={derivationPath}
            onChange={(e) => setDerivationPath(e.target.value)}
            placeholder="Derivation path (optional)"
            style={{ marginRight: "0.5rem", padding: "0.5rem" }}
          />
          <button onClick={async () => {
            try {
              const response = await os.getPublicKey(algorithm, derivationPath || undefined);
              setPublicKey(response.public_key);
              setVerificationResult(null);
            } catch (error) {
              console.error("Failed to get public key:", error);
              alert("Failed to get public key: " + (error as Error).message);
            }
          }}>Get Public Key</button>
        </div>

        {publicKey && (
          <div className="data-display" style={{ wordBreak: "break-all", marginBottom: "1rem" }}>
            <strong>Public Key:</strong> {publicKey}
          </div>
        )}

        <form onSubmit={handleSignMessage} className="auth-form">
          <details style={{ marginBottom: "0.5rem" }}>
            <summary>About message signing</summary>
            <p><small>
              Messages are converted to bytes before signing. Examples:
            </small></p>
            <pre style={{ fontSize: "small" }}>
{`// From string
const messageBytes = new TextEncoder().encode("Hello, World!");

// From hex
const messageBytes = new Uint8Array(Buffer.from("deadbeef", "hex"));`}
            </pre>
          </details>
          <textarea 
            name="message" 
            placeholder="Enter message to sign" 
            rows={4} 
            className="json-input"
            required 
          />
          <button type="submit">Sign Message</button>
        </form>

        {lastSignature && (
          <div className="data-display" style={{ marginTop: "1rem", wordBreak: "break-all" }}>
            <div><strong>Message:</strong> {lastSignature.message}</div>
            <div><strong>Message Hash:</strong> {lastSignature.messageHash}</div>
            <div><strong>Signature:</strong> {lastSignature.signature}</div>
            <button 
              onClick={handleVerifySignature}
              style={{ marginTop: "0.5rem" }}
            >
              Verify Signature
            </button>
            {verificationResult !== null && (
              <div style={{ 
                marginTop: "0.5rem",
                color: verificationResult ? "green" : "red",
                fontWeight: "bold"
              }}>
                Signature is {verificationResult ? "valid" : "invalid"}!
              </div>
            )}
          </div>
        )}
      </section>

      <section>
        <h2>AI Chat Demo</h2>
        <p>Try out the AI chat functionality with encrypted communication.</p>
        <AI />
      </section>

      <section>
        <h2>Third Party Token</h2>
        <p>Generate JWT tokens for authorized third-party services (URL-based).</p>
        <TokenGenerator />
      </section>

    </main>
  );
}

export default App;
