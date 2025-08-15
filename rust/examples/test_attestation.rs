use opensecret_sdk::{OpenSecretClient, Result};
use std::env;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
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
    
    let email = env::var("VITE_TEST_EMAIL")
        .or_else(|_| env::var("OPENSECRET_EMAIL"))
        .unwrap_or_else(|_| {
            println!("VITE_TEST_EMAIL not set, skipping login");
            String::new()
        });
    
    let password = env::var("VITE_TEST_PASSWORD")
        .or_else(|_| env::var("OPENSECRET_PASSWORD"))
        .unwrap_or_else(|_| {
            println!("VITE_TEST_PASSWORD not set, skipping login");
            String::new()
        });
    
    println!("Creating OpenSecret client for: {}", base_url);
    let client = OpenSecretClient::new(base_url)?;
    
    // Test basic connection
    println!("\n1. Testing connection...");
    match client.test_connection().await {
        Ok(response) => println!("   ✓ Health check response: {}", response),
        Err(e) => println!("   ✗ Health check failed: {}", e),
    }
    
    // Login if credentials provided
    if !email.is_empty() && !password.is_empty() {
        println!("\n2. Logging in...");
        match client.login(&email, &password).await {
            Ok(_) => println!("   ✓ Login successful"),
            Err(e) => {
                println!("   ✗ Login failed: {}", e);
                println!("   Continuing without authentication...");
            }
        }
    } else {
        println!("\n2. Skipping login (no credentials provided)");
    }
    
    // Test attestation handshake
    println!("\n3. Performing attestation handshake...");
    match client.perform_attestation_handshake().await {
        Ok(_) => {
            println!("   ✓ Attestation handshake successful!");
            
            // Check if we have a session
            if let Ok(Some(session_id)) = client.get_session_id() {
                println!("   ✓ Session established: {}", session_id);
            }
        }
        Err(e) => {
            println!("   ✗ Attestation handshake failed: {}", e);
            println!("\nDebug info:");
            println!("   - Make sure the OpenSecret server is running");
            println!("   - Check that the API URL is correct: {}", 
                     env::var("OPENSECRET_API_URL").unwrap_or_else(|_| "http://localhost:3000 (default)".to_string()));
            println!("   - Error details: {:?}", e);
        }
    }
    
    Ok(())
}