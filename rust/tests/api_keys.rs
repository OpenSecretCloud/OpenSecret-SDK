use opensecret::{OpenSecretClient, Result};
use std::env;
use uuid::Uuid;

/// Helper function to load environment variables from .env.local
fn load_env_vars() {
    let env_path = std::path::Path::new("../.env.local");
    if env_path.exists() {
        dotenv::from_path(env_path).ok();
    } else {
        // Fallback to standard .env
        dotenv::dotenv().ok();
    }
}

/// Helper function to get test environment configuration
fn get_test_config() -> (String, String, String, Uuid) {
    load_env_vars();

    let api_url = env::var("VITE_OPEN_SECRET_API_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());
    let email = env::var("VITE_TEST_EMAIL").expect("VITE_TEST_EMAIL must be set");
    let password = env::var("VITE_TEST_PASSWORD").expect("VITE_TEST_PASSWORD must be set");
    let client_id = env::var("VITE_TEST_CLIENT_ID")
        .expect("VITE_TEST_CLIENT_ID must be set")
        .parse::<Uuid>()
        .expect("VITE_TEST_CLIENT_ID must be a valid UUID");

    (api_url, email, password, client_id)
}

async fn setup_test_client() -> Result<OpenSecretClient> {
    let (api_url, email, password, client_id) = get_test_config();

    let client = OpenSecretClient::new(&api_url)?;
    client.perform_attestation_handshake().await?;
    client.login(email, password, client_id).await?;

    Ok(client)
}

#[tokio::test]
async fn test_create_list_delete_api_key() -> Result<()> {
    let client = setup_test_client().await?;

    // Create a new API key
    let key_name = format!("Test Key {}", chrono::Utc::now().timestamp());
    let created_key = client.create_api_key(key_name.clone()).await?;

    assert_eq!(created_key.name, key_name);
    assert!(!created_key.key.is_empty());

    // Verify the key format is a UUID with dashes
    let uuid_result = Uuid::parse_str(&created_key.key);
    assert!(uuid_result.is_ok(), "API key should be a valid UUID");

    // List API keys and verify the new key is present
    let keys = client.list_api_keys().await?;
    let found_key = keys.iter().find(|k| k.name == created_key.name);

    assert!(found_key.is_some());
    assert_eq!(found_key.unwrap().name, key_name);

    // Delete the API key
    client.delete_api_key(&created_key.name).await?;

    // Verify the key is deleted
    let keys_after_delete = client.list_api_keys().await?;
    let deleted_key = keys_after_delete
        .iter()
        .find(|k| k.name == created_key.name);
    assert!(deleted_key.is_none());

    Ok(())
}

#[tokio::test]
async fn test_api_key_authentication() -> Result<()> {
    let (api_url, email, password, client_id) = get_test_config();

    // First create an API key using regular auth
    let client = OpenSecretClient::new(&api_url)?;
    client.perform_attestation_handshake().await?;
    client.login(email, password, client_id).await?;

    let key_name = format!("Test API Key {}", chrono::Utc::now().timestamp());
    let created_key = client.create_api_key(key_name.clone()).await?;
    let api_key = created_key.key.clone();
    let api_key_name = created_key.name.clone();

    // Now create a new client with just the API key
    let api_client = OpenSecretClient::new_with_api_key(&api_url, api_key.clone())?;
    api_client.perform_attestation_handshake().await?;

    // Test fetching models with API key authentication
    let models = api_client.get_models().await?;
    assert!(!models.data.is_empty());

    // Test that the models endpoint works
    let model_exists = models
        .data
        .iter()
        .any(|m| m.id.contains("llama") || m.id.contains("gpt"));
    assert!(model_exists, "Should have at least one model available");

    // Clean up: delete the API key
    client.delete_api_key(&api_key_name).await?;

    Ok(())
}

// Skip non-streaming test since server only supports streaming
// The streaming test below covers the API key functionality

#[tokio::test]
async fn test_streaming_chat_with_api_key() -> Result<()> {
    use futures::StreamExt;
    use opensecret::{ChatCompletionRequest, ChatMessage};

    let (api_url, email, password, client_id) = get_test_config();

    // First create an API key using regular auth
    let client = OpenSecretClient::new(&api_url)?;
    client.perform_attestation_handshake().await?;
    client.login(email, password, client_id).await?;

    let key_name = format!("Test Stream Key {}", chrono::Utc::now().timestamp());
    let created_key = client.create_api_key(key_name.clone()).await?;
    let api_key = created_key.key.clone();
    let api_key_name = created_key.name.clone();

    // Create a new client with the API key
    let api_client = OpenSecretClient::new_with_api_key(&api_url, api_key)?;
    api_client.perform_attestation_handshake().await?;

    // Test streaming chat completion
    let request = ChatCompletionRequest {
        model: "ibnzterrell/Meta-Llama-3.3-70B-Instruct-AWQ-INT4".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: serde_json::json!("Please reply with exactly and only the word 'echo'"),
            tool_calls: None,
        }],
        temperature: Some(0.1),
        max_tokens: Some(10),
        stream: Some(true),
        stream_options: None,
        tools: None,
        tool_choice: None,
    };

    let mut stream = api_client.create_chat_completion_stream(request).await?;
    let mut full_response = String::new();

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                if !chunk.choices.is_empty() {
                    if let Some(serde_json::Value::String(s)) = &chunk.choices[0].delta.content {
                        full_response.push_str(s);
                    }
                }
            }
            Err(e) => {
                eprintln!("Stream error: {}", e);
                break;
            }
        }
    }

    assert_eq!(full_response.trim().to_lowercase(), "echo");

    // Clean up
    client.delete_api_key(&api_key_name).await?;

    Ok(())
}

#[tokio::test]
async fn test_multiple_api_keys() -> Result<()> {
    let client = setup_test_client().await?;
    let mut key_names = Vec::new();

    // Create multiple keys
    for i in 0..3 {
        let key_name = format!("Test Key {} - {}", i, chrono::Utc::now().timestamp());
        let created_key = client.create_api_key(key_name.clone()).await?;
        key_names.push(created_key.name);
    }

    // List keys and verify all are present
    let keys = client.list_api_keys().await?;
    for name in &key_names {
        assert!(keys.iter().any(|k| k.name == *name));
    }

    // Clean up: delete all created keys
    for name in key_names {
        client.delete_api_key(&name).await?;
    }

    Ok(())
}

#[tokio::test]
async fn test_invalid_api_key_fails() -> Result<()> {
    let (api_url, _, _, _) = get_test_config();

    // Create a client with an invalid API key
    let invalid_key = "550e8400-e29b-41d4-a716-000000000000".to_string();
    let api_client = OpenSecretClient::new_with_api_key(&api_url, invalid_key)?;
    api_client.perform_attestation_handshake().await?;

    // This should fail with authentication error
    let result = api_client.get_models().await;
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("401") || error_msg.contains("Unauthorized"));

    Ok(())
}
