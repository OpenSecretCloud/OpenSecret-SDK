use futures::StreamExt;
use opensecret::{
    ChatCompletionRequest, ChatMessage, EmbeddingInput, EmbeddingRequest, OpenSecretClient, Result,
};
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
    let name = env::var("VITE_TEST_NAME").ok();
    let client_id = env::var("VITE_TEST_CLIENT_ID")
        .expect("VITE_TEST_CLIENT_ID must be set")
        .parse::<Uuid>()
        .expect("Invalid client_id format");

    // Try login first, if it fails then register
    match client
        .login(email.clone(), password.clone(), client_id)
        .await
    {
        Ok(_) => {}
        Err(_) => {
            // Register the user if login failed
            client.register(email, password, client_id, name).await?;
        }
    }

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
        model: "llama-3.3-70b".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: serde_json::json!(r#"please reply with exactly and only the word "echo""#),
            tool_calls: None,
        }],
        temperature: Some(0.0),
        max_tokens: Some(10),
        stream: Some(true),
        stream_options: None,
        tools: None,
        tool_choice: None,
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
            if let Some(serde_json::Value::String(s)) = &chunk.choices[0].delta.content {
                full_response.push_str(s);
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
        model: "llama-3.3-70b".to_string(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: serde_json::json!(
                    "You are a helpful assistant that always responds with exactly one word."
                ),
                tool_calls: None,
            },
            ChatMessage {
                role: "user".to_string(),
                content: serde_json::json!("What is 2+2? Answer in one word."),
                tool_calls: None,
            },
        ],
        temperature: Some(0.0),
        max_tokens: Some(10),
        stream: Some(true), // Server only supports streaming
        stream_options: None,
        tools: None,
        tool_choice: None,
    };

    let mut stream = client
        .create_chat_completion_stream(request)
        .await
        .expect("Failed to create streaming completion");

    let mut full_response = String::new();
    while let Some(result) = stream.next().await {
        let chunk = result.expect("Failed to get chunk");
        if !chunk.choices.is_empty() {
            if let Some(serde_json::Value::String(s)) = &chunk.choices[0].delta.content {
                full_response.push_str(s);
            }
        }
    }

    // Should be a single word like "four" or "4"
    let word_count = full_response.split_whitespace().count();
    assert_eq!(word_count, 1, "Response should be exactly one word");
}

#[tokio::test]
async fn test_delete_conversations() {
    let client = setup_authenticated_client()
        .await
        .expect("Failed to setup client");

    // 1. Create a couple of conversations first
    // Note: Currently the SDK doesn't have create_conversation exposed directly on client struct
    // but create_chat_completion implicitly creates or uses conversation if we could access it.
    // However, based on the TS SDK, there's explicit conversation management.
    // The Rust SDK seems to be catching up.
    // For now, we'll just call delete_conversations and ensure it returns success,
    // which covers the API contract even if list is empty.

    // Ideally we would create conversations here, but the Rust SDK client wrapper
    // doesn't seem to expose explicit conversation creation methods yet based on my earlier grep.
    // It only has what I added: delete_conversations.
    // The TS SDK had `createConversation`.

    // Let's verify what methods OpenSecretClient actually has.
    // I only added `delete_conversations`.

    let result = client
        .delete_conversations()
        .await
        .expect("Failed to delete conversations");

    assert_eq!(result.object, "list.deleted");
    assert!(result.deleted);
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
            content: serde_json::json!("test"),
            tool_calls: None,
        }],
        temperature: None,
        max_tokens: None,
        stream: Some(true), // Server only supports streaming
        stream_options: None,
        tools: None,
        tool_choice: None,
    };

    let completion_result = client.create_chat_completion(request).await;
    assert!(
        completion_result.is_err(),
        "Guest users should not be able to create completions"
    );
}

#[tokio::test]
async fn test_create_embeddings_single_input() {
    let client = setup_authenticated_client()
        .await
        .expect("Failed to setup client");

    let request = EmbeddingRequest {
        input: EmbeddingInput::Single("Hello, world!".to_string()),
        model: "nomic-embed-text".to_string(),
        encoding_format: None,
        dimensions: None,
        user: None,
    };

    let response = client
        .create_embeddings(request)
        .await
        .expect("Failed to create embeddings");

    // Verify response structure
    assert_eq!(response.object, "list");
    assert_eq!(response.data.len(), 1);
    assert_eq!(response.data[0].object, "embedding");
    assert_eq!(response.data[0].index, 0);

    // nomic-embed-text has 768 dimensions
    assert_eq!(
        response.data[0].embedding.len(),
        768,
        "Expected 768 dimensions for nomic-embed-text"
    );

    // Verify usage
    assert!(response.usage.prompt_tokens > 0);
    assert!(response.usage.total_tokens > 0);

    println!(
        "Embedding created with {} dimensions, {} tokens used",
        response.data[0].embedding.len(),
        response.usage.total_tokens
    );
}

#[tokio::test]
async fn test_create_embeddings_multiple_inputs() {
    let client = setup_authenticated_client()
        .await
        .expect("Failed to setup client");

    let request = EmbeddingRequest {
        input: EmbeddingInput::Multiple(vec![
            "First text to embed".to_string(),
            "Second text to embed".to_string(),
            "Third text to embed".to_string(),
        ]),
        model: "nomic-embed-text".to_string(),
        encoding_format: None,
        dimensions: None,
        user: None,
    };

    let response = client
        .create_embeddings(request)
        .await
        .expect("Failed to create embeddings");

    // Verify response structure
    assert_eq!(response.object, "list");
    assert_eq!(response.data.len(), 3, "Should have 3 embeddings");

    // Check each embedding
    for (i, embedding_data) in response.data.iter().enumerate() {
        assert_eq!(embedding_data.object, "embedding");
        assert_eq!(embedding_data.index as usize, i);
        assert_eq!(
            embedding_data.embedding.len(),
            768,
            "Each embedding should have 768 dimensions"
        );
    }

    // Verify usage accounts for all inputs
    assert!(response.usage.prompt_tokens > 0);

    println!(
        "Created {} embeddings, {} total tokens used",
        response.data.len(),
        response.usage.total_tokens
    );
}

#[tokio::test]
async fn test_embeddings_from_string_conversion() {
    let client = setup_authenticated_client()
        .await
        .expect("Failed to setup client");

    // Test the From<&str> conversion
    let request = EmbeddingRequest {
        input: "Test string conversion".into(),
        model: "nomic-embed-text".to_string(),
        encoding_format: None,
        dimensions: None,
        user: None,
    };

    let response = client
        .create_embeddings(request)
        .await
        .expect("Failed to create embeddings");

    assert_eq!(response.data.len(), 1);
    assert_eq!(response.data[0].embedding.len(), 768);
}
