use futures::StreamExt;
use opensecret_sdk::{ChatCompletionRequest, ChatMessage, OpenSecretClient, Result};
use std::env;
use uuid::Uuid;

async fn setup_authenticated_client() -> Result<OpenSecretClient> {
    // Load .env.local from OpenSecret-SDK directory
    let env_path = std::path::Path::new("../.env.local");
    if env_path.exists() {
        dotenv::from_path(env_path).ok();
    } else {
        // Fallback to standard .env
        dotenv::dotenv().ok();
    }

    let base_url = env::var("VITE_OPEN_SECRET_API_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());
    let client = OpenSecretClient::new(base_url)?;
    client.perform_attestation_handshake().await?;

    // Login with test credentials
    let email = env::var("VITE_TEST_EMAIL").expect("VITE_TEST_EMAIL must be set");
    let password = env::var("VITE_TEST_PASSWORD").expect("VITE_TEST_PASSWORD must be set");
    let client_id = env::var("VITE_TEST_CLIENT_ID")
        .expect("VITE_TEST_CLIENT_ID must be set")
        .parse::<Uuid>()
        .expect("Invalid client_id format");

    client.login(email, password, client_id).await?;
    Ok(client)
}

#[tokio::test]
async fn test_get_models() {
    let client = setup_authenticated_client()
        .await
        .expect("Failed to setup client");

    let models = client.get_models().await.expect("Failed to get models");

    // Verify response structure
    assert_eq!(models.object, "list");
    assert!(!models.data.is_empty(), "Should have at least one model");

    // Check that each model has required fields
    for model in &models.data {
        assert!(!model.id.is_empty(), "Model ID should not be empty");
        assert_eq!(model.object, "model");
        // owned_by is optional in the server response
        if let Some(ref owned_by) = model.owned_by {
            assert!(
                !owned_by.is_empty(),
                "Model owner should not be empty if present"
            );
        }
    }

    println!("Found {} models", models.data.len());
    println!("First model: {}", models.data[0].id);
}

#[tokio::test]
#[ignore = "Server currently only supports streaming completions"]
async fn test_chat_completion_non_streaming() {
    // The server's /v1/chat/completions endpoint only returns SSE streams
    // Non-streaming is not currently implemented on the server side
    // This test is kept for future implementation
}

#[tokio::test]
async fn test_chat_completion_streaming() {
    let client = setup_authenticated_client()
        .await
        .expect("Failed to setup client");

    let request = ChatCompletionRequest {
        model: "ibnzterrell/Meta-Llama-3.3-70B-Instruct-AWQ-INT4".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: r#"please reply with exactly and only the word "echo""#.to_string(),
        }],
        temperature: Some(0.0),
        max_tokens: Some(10),
        stream: Some(true),
        stream_options: None,
    };

    let mut stream = client
        .create_chat_completion_stream(request)
        .await
        .expect("Failed to create streaming completion");

    let mut full_response = String::new();
    let mut chunk_count = 0;
    let mut saw_usage = false;

    while let Some(result) = stream.next().await {
        let chunk = result.expect("Failed to get chunk");
        chunk_count += 1;

        // Check chunk structure
        assert!(!chunk.id.is_empty());
        assert_eq!(chunk.object, "chat.completion.chunk");

        if !chunk.choices.is_empty() {
            if let Some(content) = &chunk.choices[0].delta.content {
                full_response.push_str(content);
            }
        }

        // Check if we got usage in the final chunk
        if chunk.usage.is_some() {
            saw_usage = true;
            let usage = chunk.usage.unwrap();
            assert!(usage.prompt_tokens > 0);
            assert!(usage.completion_tokens > 0);
        }
    }

    assert!(chunk_count > 0, "Should have received at least one chunk");
    assert_eq!(full_response.trim().to_lowercase(), "echo");
    assert!(saw_usage, "Should have received usage information");
}

#[tokio::test]
async fn test_chat_completion_with_system_message() {
    let client = setup_authenticated_client()
        .await
        .expect("Failed to setup client");

    let request = ChatCompletionRequest {
        model: "ibnzterrell/Meta-Llama-3.3-70B-Instruct-AWQ-INT4".to_string(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a helpful assistant that always responds with exactly one word."
                    .to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "What is 2+2? Answer in one word.".to_string(),
            },
        ],
        temperature: Some(0.0),
        max_tokens: Some(10),
        stream: Some(true), // Server only supports streaming
        stream_options: None,
    };

    let mut stream = client
        .create_chat_completion_stream(request)
        .await
        .expect("Failed to create streaming completion");

    let mut full_response = String::new();
    while let Some(result) = stream.next().await {
        let chunk = result.expect("Failed to get chunk");
        if !chunk.choices.is_empty() {
            if let Some(content) = &chunk.choices[0].delta.content {
                full_response.push_str(content);
            }
        }
    }

    // Should be a single word like "four" or "4"
    let word_count = full_response.split_whitespace().count();
    assert_eq!(word_count, 1, "Response should be exactly one word");
}

#[tokio::test]
async fn test_guest_user_cannot_use_ai() {
    // Load .env.local from OpenSecret-SDK directory
    let env_path = std::path::Path::new("../.env.local");
    if env_path.exists() {
        dotenv::from_path(env_path).ok();
    }

    let base_url = env::var("VITE_OPEN_SECRET_API_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());

    // Use the client_id from env for consistency with other tests
    let client_id = env::var("VITE_TEST_CLIENT_ID")
        .ok()
        .and_then(|id| Uuid::parse_str(&id).ok())
        .unwrap_or_else(Uuid::new_v4);

    let client = OpenSecretClient::new(base_url).expect("Failed to create client");
    client
        .perform_attestation_handshake()
        .await
        .expect("Failed to perform handshake");

    // Register as guest (no email)
    let password = format!("TestGuestPassword_{}", Uuid::new_v4());
    client
        .register_guest(password.to_string(), client_id)
        .await
        .expect("Failed to register guest");

    // Try to get models - should fail
    let models_result = client.get_models().await;
    assert!(
        models_result.is_err(),
        "Guest users should not be able to access models"
    );

    // Try to create completion - should fail
    let request = ChatCompletionRequest {
        model: "some-model".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: "test".to_string(),
        }],
        temperature: None,
        max_tokens: None,
        stream: Some(true), // Server only supports streaming
        stream_options: None,
    };

    let completion_result = client.create_chat_completion(request).await;
    assert!(
        completion_result.is_err(),
        "Guest users should not be able to create completions"
    );
}
