use opensecret_sdk::{OpenSecretClient, Result};
use std::env;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

#[tokio::test]
async fn test_client_initialization() -> Result<()> {
    let client = OpenSecretClient::new("http://localhost:3000".to_string())?;
    assert!(client.get_session_id()?.is_none());
    Ok(())
}

#[tokio::test]
async fn test_health_check_mock() -> Result<()> {
    // Start a mock server
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/health-check"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "healthy",
            "timestamp": "2024-01-01T00:00:00Z"
        })))
        .mount(&mock_server)
        .await;
    
    let client = OpenSecretClient::new(mock_server.uri())?;
    let response = client.test_connection().await?;
    
    assert!(response.contains("healthy"));
    
    Ok(())
}

#[tokio::test]
async fn test_full_flow_with_real_server() -> Result<()> {
    // Load .env.local from OpenSecret-SDK directory
    dotenv::from_path("../.env.local").ok();
    
    // This test requires environment variables to be set
    let base_url = env::var("VITE_OPEN_SECRET_API_URL")
        .or_else(|_| env::var("OPENSECRET_TEST_URL"))
        .unwrap_or_else(|_| "http://localhost:3000".to_string());
    
    println!("Testing against: {}", base_url);
    
    let client = OpenSecretClient::new(base_url)?;
    
    // 1. Test connection
    let health = client.test_connection().await?;
    println!("Health check: {}", health);
    
    // 2. Perform attestation
    client.perform_attestation_handshake().await?;
    println!("Attestation successful");
    
    // 3. Verify session
    let session_id = client.get_session_id()?.expect("Should have session");
    println!("Session established: {}", session_id);
    
    Ok(())
}