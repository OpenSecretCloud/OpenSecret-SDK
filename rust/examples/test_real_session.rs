use opensecret_sdk::{OpenSecretClient, Result};
use std::env;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env.local from parent directory if it exists
    let env_path = Path::new("../.env.local");
    if env_path.exists() {
        dotenv::from_path(env_path).ok();
    }
    
    let base_url = env::var("VITE_OPEN_SECRET_API_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());
    
    println!("Testing real session key decryption with {}", base_url);
    let client = OpenSecretClient::new(base_url)?;
    
    // Perform attestation handshake
    println!("\nPerforming attestation handshake...");
    client.perform_attestation_handshake().await?;
    
    if let Ok(Some(session_id)) = client.get_session_id() {
        println!("✓ Session established: {}", session_id);
        
        // Test making an encrypted request
        println!("\nTesting encrypted communication...");
        
        // Try to make a simple encrypted request to verify the session works
        match client.make_encrypted_request::<(), serde_json::Value>(
            "GET",
            "/health-check",
            None,
        ).await {
            Ok(response) => {
                println!("✓ Encrypted request successful!");
                println!("  Response: {:?}", response);
            },
            Err(e) => {
                println!("✗ Encrypted request failed: {}", e);
            }
        }
    }
    
    Ok(())
}