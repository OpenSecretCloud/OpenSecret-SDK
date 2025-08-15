use opensecret_sdk::{OpenSecretClient, Result};
use std::env;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for better debugging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    
    // Load .env.local from parent directory if it exists
    let env_path = Path::new("../.env.local");
    if env_path.exists() {
        dotenv::from_path(env_path).ok();
        println!("Loaded environment from ../.env.local");
    }
    
    // Get configuration from environment (compatible with VITE_ prefixed variables)
    let base_url = env::var("VITE_OPEN_SECRET_API_URL")
        .or_else(|_| env::var("OPENSECRET_API_URL"))
        .unwrap_or_else(|_| "http://localhost:3000".to_string());
    
    println!("================================");
    println!("ATTESTATION VERIFICATION TEST");
    println!("================================");
    println!("Target URL: {}", base_url);
    println!("Mode: {}", if base_url.contains("localhost") || base_url.contains("127.0.0.1") { 
        "MOCK (localhost)" 
    } else { 
        "PRODUCTION (real attestation)" 
    });
    println!("================================\n");
    
    let client = OpenSecretClient::new(base_url.clone())?;
    
    // Test ONLY the attestation handshake
    println!("Starting attestation handshake...");
    match client.perform_attestation_handshake().await {
        Ok(_) => {
            println!("✅ ATTESTATION VERIFICATION SUCCESSFUL!");
            
            if let Ok(Some(session_id)) = client.get_session_id() {
                println!("✅ Session established with ID: {}", session_id);
                println!("\nThis means:");
                println!("  1. Attestation document was fetched");
                println!("  2. CBOR parsing succeeded");
                println!("  3. Nonce verification passed");
                if !base_url.contains("localhost") && !base_url.contains("127.0.0.1") {
                    println!("  4. Certificate chain was validated");
                    println!("  5. Signature verification PASSED");
                    println!("  6. Document is from a REAL AWS Nitro Enclave");
                } else {
                    println!("  4. Mock attestation was accepted (dev mode)");
                }
                println!("  7. Server's public key was extracted");
                println!("  8. Key exchange completed successfully");
                println!("  9. Session key was decrypted");
            }
        }
        Err(e) => {
            println!("❌ ATTESTATION VERIFICATION FAILED!");
            println!("Error: {}", e);
            
            // Provide detailed debugging info based on error type
            match e {
                opensecret_sdk::Error::AttestationVerificationFailed(msg) => {
                    println!("\n🔍 Attestation Error Details: {}", msg);
                    
                    if msg.contains("Signature verification failed") {
                        println!("\n📝 This means:");
                        println!("  - The COSE_Sign1 signature doesn't match");
                        println!("  - Either the signature algorithm is wrong");
                        println!("  - Or the signature structure encoding is incorrect");
                        println!("  - Or the public key extraction is wrong");
                    } else if msg.contains("Certificate") {
                        println!("\n📝 Certificate chain issue - check cert parsing");
                    } else if msg.contains("Nonce") {
                        println!("\n📝 Nonce mismatch - attestation may be stale");
                    }
                }
                opensecret_sdk::Error::KeyExchange(msg) => {
                    println!("\n🔑 Key Exchange Error: {}", msg);
                }
                _ => {
                    println!("\n❓ Unexpected error type: {:?}", e);
                }
            }
        }
    }
    
    Ok(())
}