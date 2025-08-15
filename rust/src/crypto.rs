use chacha20poly1305::{
    aead::{Aead, KeyInit, Nonce},
    ChaCha20Poly1305,
};
use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret, StaticSecret};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use crate::error::{Error, Result};

pub fn generate_random_bytes<const N: usize>() -> [u8; N] {
    let mut bytes = [0u8; N];
    getrandom::getrandom(&mut bytes).expect("Failed to generate random bytes");
    bytes
}

pub fn generate_ephemeral_keypair() -> (EphemeralSecret, PublicKey) {
    let secret = EphemeralSecret::random_from_rng(rand::thread_rng());
    let public = PublicKey::from(&secret);
    (secret, public)
}

pub fn generate_static_keypair() -> (StaticSecret, PublicKey) {
    let secret = StaticSecret::random_from_rng(rand::thread_rng());
    let public = PublicKey::from(&secret);
    (secret, public)
}

pub fn perform_key_exchange(secret: EphemeralSecret, their_public: &PublicKey) -> SharedSecret {
    secret.diffie_hellman(their_public)
}

pub fn perform_static_key_exchange(secret: &StaticSecret, their_public: &PublicKey) -> SharedSecret {
    secret.diffie_hellman(their_public)
}

pub fn encrypt_data(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new_from_slice(key)
        .map_err(|e| Error::Crypto(format!("Failed to create cipher: {}", e)))?;
    
    let nonce_bytes = generate_random_bytes::<12>();
    let nonce = Nonce::<ChaCha20Poly1305>::from_slice(&nonce_bytes);
    
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| Error::Encryption(format!("Encryption failed: {}", e)))?;
    
    // Prepend nonce to ciphertext
    let mut result = Vec::with_capacity(12 + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    
    Ok(result)
}

pub fn decrypt_data(key: &[u8; 32], encrypted_data: &[u8]) -> Result<Vec<u8>> {
    if encrypted_data.len() < 12 {
        return Err(Error::Decryption("Encrypted data too short".to_string()));
    }
    
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
    let nonce = Nonce::<ChaCha20Poly1305>::from_slice(nonce_bytes);
    
    let cipher = ChaCha20Poly1305::new_from_slice(key)
        .map_err(|e| Error::Crypto(format!("Failed to create cipher: {}", e)))?;
    
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| Error::Decryption(format!("Decryption failed: {}", e)))
}

pub fn encrypt_json<T: serde::Serialize>(key: &[u8; 32], data: &T) -> Result<String> {
    let json = serde_json::to_string(data)?;
    let encrypted = encrypt_data(key, json.as_bytes())?;
    Ok(BASE64.encode(encrypted))
}

pub fn decrypt_json<T: serde::de::DeserializeOwned>(key: &[u8; 32], encrypted_b64: &str) -> Result<T> {
    let encrypted = BASE64.decode(encrypted_b64)?;
    let decrypted = decrypt_data(key, &encrypted)?;
    let json_str = String::from_utf8(decrypted)?;
    Ok(serde_json::from_str(&json_str)?)
}

pub fn decrypt_session_key(
    shared_secret: &SharedSecret,
    encrypted_data: &str,
) -> Result<[u8; 32]> {
    let encrypted = BASE64.decode(encrypted_data)?;
    
    if encrypted.len() < 12 {
        return Err(Error::Decryption("Encrypted session key too short".to_string()));
    }
    
    let (nonce_bytes, ciphertext) = encrypted.split_at(12);
    let nonce = Nonce::<ChaCha20Poly1305>::from_slice(nonce_bytes);
    
    let cipher = ChaCha20Poly1305::new_from_slice(shared_secret.as_bytes())
        .map_err(|e| Error::Crypto(format!("Failed to create cipher: {}", e)))?;
    
    let session_key_vec = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| Error::Decryption(format!("Failed to decrypt session key: {}", e)))?;
    
    if session_key_vec.len() != 32 {
        return Err(Error::Decryption("Invalid session key length".to_string()));
    }
    
    let mut session_key = [0u8; 32];
    session_key.copy_from_slice(&session_key_vec);
    Ok(session_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = generate_random_bytes::<32>();
        let plaintext = b"Hello, World!";
        
        let encrypted = encrypt_data(&key, plaintext).unwrap();
        let decrypted = decrypt_data(&key, &encrypted).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_key_exchange() {
        // Use static secrets for testing since ephemeral secrets are consumed
        let (alice_secret, alice_public) = generate_static_keypair();
        let (bob_secret, bob_public) = generate_static_keypair();
        
        let alice_shared = perform_static_key_exchange(&alice_secret, &bob_public);
        let bob_shared = perform_static_key_exchange(&bob_secret, &alice_public);
        
        assert_eq!(alice_shared.as_bytes(), bob_shared.as_bytes());
    }
}