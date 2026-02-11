use futures::StreamExt;
use opensecret::{
    AgentSseEvent, MemorySearchRequest, OpenSecretClient, Result, UpdateAgentConfigRequest,
    UpdateMemoryBlockRequest,
};
use serde_json::json;
use std::env;
use uuid::Uuid;

async fn setup_authenticated_client() -> Result<OpenSecretClient> {
    let env_path = std::path::Path::new("../.env.local");
    if env_path.exists() {
        dotenv::from_path(env_path).ok();
    } else {
        dotenv::dotenv().ok();
    }

    let base_url = env::var("VITE_OPEN_SECRET_API_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());
    let client = OpenSecretClient::new(base_url)?;
    client.perform_attestation_handshake().await?;

    let email = env::var("VITE_TEST_EMAIL").expect("VITE_TEST_EMAIL must be set");
    let password = env::var("VITE_TEST_PASSWORD").expect("VITE_TEST_PASSWORD must be set");
    let name = env::var("VITE_TEST_NAME").ok();
    let client_id = env::var("VITE_TEST_CLIENT_ID")
        .expect("VITE_TEST_CLIENT_ID must be set")
        .parse::<Uuid>()
        .expect("Invalid client_id format");

    match client
        .login(email.clone(), password.clone(), client_id)
        .await
    {
        Ok(_) => {}
        Err(_) => {
            client.register(email, password, client_id, name).await?;
        }
    }

    Ok(client)
}

#[tokio::test]
async fn test_get_agent_config() {
    let client = setup_authenticated_client()
        .await
        .expect("Failed to setup client");

    let config = client
        .get_agent_config()
        .await
        .expect("Failed to get agent config");

    assert!(!config.model.is_empty(), "Model should not be empty");
    assert!(
        config.max_context_tokens > 0,
        "Max context tokens should be positive"
    );
    assert!(
        config.compaction_threshold > 0.0 && config.compaction_threshold <= 1.0,
        "Compaction threshold should be between 0 and 1"
    );

    println!(
        "Agent config: model={}, enabled={}",
        config.model, config.enabled
    );
}

#[tokio::test]
async fn test_update_agent_config() {
    let client = setup_authenticated_client()
        .await
        .expect("Failed to setup client");

    let original = client
        .get_agent_config()
        .await
        .expect("Failed to get original config");

    let update = UpdateAgentConfigRequest {
        enabled: Some(true),
        model: None,
        max_context_tokens: Some(80_000),
        compaction_threshold: None,
        system_prompt: Some("You are a test assistant.".to_string()),
    };

    let updated = client
        .update_agent_config(update)
        .await
        .expect("Failed to update agent config");

    assert!(updated.enabled);
    assert_eq!(updated.max_context_tokens, 80_000);
    assert_eq!(
        updated.system_prompt.as_deref(),
        Some("You are a test assistant.")
    );

    // Restore original
    let restore = UpdateAgentConfigRequest {
        enabled: Some(original.enabled),
        model: Some(original.model),
        max_context_tokens: Some(original.max_context_tokens),
        compaction_threshold: Some(original.compaction_threshold),
        system_prompt: original.system_prompt,
    };
    client
        .update_agent_config(restore)
        .await
        .expect("Failed to restore config");
}

#[tokio::test]
async fn test_list_memory_blocks() {
    let client = setup_authenticated_client()
        .await
        .expect("Failed to setup client");

    // Trigger agent config init (which creates default blocks)
    let _ = client.get_agent_config().await;

    let blocks = client
        .list_memory_blocks()
        .await
        .expect("Failed to list memory blocks");

    assert!(
        blocks.len() >= 2,
        "Should have at least persona and human blocks"
    );

    let labels: Vec<&str> = blocks.iter().map(|b| b.label.as_str()).collect();
    assert!(labels.contains(&"persona"), "Should have persona block");
    assert!(labels.contains(&"human"), "Should have human block");

    for block in &blocks {
        assert!(!block.label.is_empty());
        assert!(block.char_limit > 0);
    }

    println!("Found {} memory blocks", blocks.len());
}

#[tokio::test]
async fn test_get_memory_block() {
    let client = setup_authenticated_client()
        .await
        .expect("Failed to setup client");

    let _ = client.get_agent_config().await;

    let block = client
        .get_memory_block("persona")
        .await
        .expect("Failed to get persona block");

    assert_eq!(block.label, "persona");
    assert!(
        !block.value.is_empty(),
        "Persona block should have a default value"
    );
    assert!(block.char_limit > 0);

    println!("Persona block: {}", block.value);
}

#[tokio::test]
async fn test_update_memory_block() {
    let client = setup_authenticated_client()
        .await
        .expect("Failed to setup client");

    let _ = client.get_agent_config().await;

    let original = client
        .get_memory_block("human")
        .await
        .expect("Failed to get human block");

    let update = UpdateMemoryBlockRequest {
        description: None,
        value: Some("Test user info from Rust SDK integration test.".to_string()),
        char_limit: None,
        read_only: None,
    };

    let updated = client
        .update_memory_block("human", update)
        .await
        .expect("Failed to update human block");

    assert_eq!(updated.label, "human");
    assert_eq!(
        updated.value,
        "Test user info from Rust SDK integration test."
    );

    // Restore original
    let restore = UpdateMemoryBlockRequest {
        description: None,
        value: Some(original.value),
        char_limit: None,
        read_only: None,
    };
    client
        .update_memory_block("human", restore)
        .await
        .expect("Failed to restore human block");
}

#[tokio::test]
async fn test_archival_memory_insert_and_delete() {
    let client = setup_authenticated_client()
        .await
        .expect("Failed to setup client");

    let inserted = client
        .insert_archival_memory(
            "Rust SDK test: The capital of France is Paris.",
            Some(json!({"tags": ["test", "geography"]})),
        )
        .await
        .expect("Failed to insert archival memory");

    assert_eq!(inserted.source_type, "archival");
    assert!(inserted.token_count > 0);

    println!(
        "Inserted archival memory: id={}, model={}",
        inserted.id, inserted.embedding_model
    );

    let deleted = client
        .delete_archival_memory(inserted.id)
        .await
        .expect("Failed to delete archival memory");

    assert!(deleted.deleted);
    assert_eq!(deleted.id, inserted.id);
}

#[tokio::test]
async fn test_memory_search() {
    let client = setup_authenticated_client()
        .await
        .expect("Failed to setup client");

    // Insert something searchable first
    let inserted = client
        .insert_archival_memory("Rust SDK search test: quantum computing uses qubits.", None)
        .await
        .expect("Failed to insert archival memory");

    // Search for it
    let search = MemorySearchRequest {
        query: "quantum computing qubits".to_string(),
        top_k: Some(5),
        max_tokens: None,
        source_types: Some(vec!["archival".to_string()]),
    };

    let results = client
        .search_agent_memory(search)
        .await
        .expect("Failed to search agent memory");

    println!("Search returned {} results", results.results.len());

    // Clean up
    let _ = client.delete_archival_memory(inserted.id).await;
}

#[tokio::test]
async fn test_agent_conversations() {
    let client = setup_authenticated_client()
        .await
        .expect("Failed to setup client");

    let conversations = client
        .list_agent_conversations()
        .await
        .expect("Failed to list agent conversations");

    assert_eq!(conversations.object, "list");

    println!("Agent has {} conversations", conversations.data.len());

    if let Some(conv) = conversations.data.first() {
        let items = client
            .list_agent_conversation_items(&conv.id, Some(10), None, None)
            .await
            .expect("Failed to list conversation items");

        assert_eq!(items.object, "list");
        println!("First conversation has {} items (page)", items.data.len());
    }
}

#[tokio::test]
async fn test_agent_chat_sse() {
    let client = setup_authenticated_client()
        .await
        .expect("Failed to setup client");

    // Ensure agent is enabled
    let _ = client
        .update_agent_config(UpdateAgentConfigRequest {
            enabled: Some(true),
            model: None,
            max_context_tokens: None,
            compaction_threshold: None,
            system_prompt: None,
        })
        .await;

    let mut stream = client
        .agent_chat("Hello, please respond with just the word 'pong'.")
        .await
        .expect("Failed to start agent chat");

    let mut got_message = false;
    let mut got_done = false;
    let mut all_messages: Vec<String> = Vec::new();

    while let Some(result) = stream.next().await {
        match result {
            Ok(event) => match event {
                AgentSseEvent::Message(msg) => {
                    got_message = true;
                    all_messages.extend(msg.messages);
                    println!("Agent message at step {}: {:?}", msg.step, all_messages);
                }
                AgentSseEvent::Done(done) => {
                    got_done = true;
                    println!(
                        "Agent done: {} steps, {} messages",
                        done.total_steps, done.total_messages
                    );
                }
                AgentSseEvent::Error(err) => {
                    panic!("Agent error: {}", err.error);
                }
            },
            Err(e) => {
                panic!("Stream error: {:?}", e);
            }
        }
    }

    assert!(
        got_message,
        "Should have received at least one message event"
    );
    assert!(got_done, "Should have received a done event");
    assert!(!all_messages.is_empty(), "Should have at least one message");
}
