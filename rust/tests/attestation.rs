use opensecret_sdk::{OpenSecretClient, Result};
use std::env;

#[tokio::test]
async fn test_attestation_handshake_localhost() -> Result<()> {
    // Skip if not running against localhost
    let base_url = env::var("VITE_OPEN_SECRET_API_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());

    if !base_url.contains("localhost") && !base_url.contains("127.0.0.1") {
        eprintln!("Skipping localhost test - not running against localhost");
        return Ok(());
    }

    let client = OpenSecretClient::new(base_url)?;

    // Perform attestation handshake with mock attestation
    client.perform_attestation_handshake().await?;

    // Verify session was established
    let session_id = client
        .get_session_id()?
        .expect("Session ID should be set after successful handshake");

    assert!(!session_id.to_string().is_empty());

    Ok(())
}

#[tokio::test]
async fn test_attestation_handshake_production() -> Result<()> {
    // Try to load .env.local if it exists (for local testing)
    if std::path::Path::new("../.env.local").exists() {
        dotenv::from_path("../.env.local").ok();
    }

    // This test requires VITE_OPEN_SECRET_API_URL to be set to a production endpoint
    let base_url = env::var("VITE_OPEN_SECRET_API_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());

    if base_url.contains("localhost") || base_url.contains("127.0.0.1") {
        println!("Skipping production attestation test - running against localhost");
        return Ok(());
    }

    let client = OpenSecretClient::new(base_url.clone())?;

    // Perform attestation handshake with real AWS Nitro attestation
    client.perform_attestation_handshake().await?;

    // Verify session was established
    let session_id = client
        .get_session_id()?
        .expect("Session ID should be set after successful handshake");

    assert!(!session_id.to_string().is_empty());
    println!("✅ Production attestation successful against {}", base_url);
    println!("   Session ID: {}", session_id);

    Ok(())
}

#[tokio::test]
async fn test_attestation_nonce_verification() -> Result<()> {
    let base_url = env::var("VITE_OPEN_SECRET_API_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());

    let client = OpenSecretClient::new(base_url.clone())?;

    // The handshake should generate a unique nonce internally
    client.perform_attestation_handshake().await?;
    let first_session = client.get_session_id()?.expect("Should have session");

    // Create a new client and do another handshake - should get different session
    let client2 = OpenSecretClient::new(base_url)?;
    client2.perform_attestation_handshake().await?;
    let second_session = client2.get_session_id()?.expect("Should have session");

    // Sessions should be different (different nonces)
    assert_ne!(
        first_session, second_session,
        "Each handshake should create a unique session"
    );

    Ok(())
}

#[tokio::test]
async fn test_connection_health_check() -> Result<()> {
    let base_url = env::var("VITE_OPEN_SECRET_API_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());

    let client = OpenSecretClient::new(base_url)?;

    // Test basic connection without attestation
    let response = client.test_connection().await?;
    assert!(!response.is_empty());

    Ok(())
}
