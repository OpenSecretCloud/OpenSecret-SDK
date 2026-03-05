use futures::StreamExt;
use opensecret::{AgentSseEvent, CreateSubagentRequest, OpenSecretClient, Result};
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

async fn create_test_subagent(client: &OpenSecretClient) -> Result<opensecret::SubagentResponse> {
    let suffix = Uuid::new_v4();

    client
        .create_subagent(CreateSubagentRequest {
            display_name: Some(format!("Rust SDK Test {}", suffix)),
            purpose: format!("Rust SDK integration test subagent {}", suffix),
        })
        .await
}

#[tokio::test]
#[ignore = "Requires agent API on server"]
async fn test_create_and_delete_subagent() {
    let client = setup_authenticated_client()
        .await
        .expect("Failed to setup client");

    let subagent = create_test_subagent(&client)
        .await
        .expect("Failed to create subagent");

    assert_eq!(subagent.object, "agent.subagent");
    assert!(!subagent.display_name.is_empty());
    assert!(!subagent.purpose.is_empty());

    let deleted = client
        .delete_subagent(subagent.id)
        .await
        .expect("Failed to delete subagent");

    assert!(deleted.deleted);
    assert_eq!(deleted.id, subagent.id);
    assert_eq!(deleted.object, "agent.subagent.deleted");
}

#[tokio::test]
#[ignore = "Requires agent API on server"]
async fn test_agent_chat_sse() {
    let client = setup_authenticated_client()
        .await
        .expect("Failed to setup client");

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

#[tokio::test]
#[ignore = "Requires agent API on server"]
async fn test_subagent_chat_sse() {
    let client = setup_authenticated_client()
        .await
        .expect("Failed to setup client");

    let subagent = create_test_subagent(&client)
        .await
        .expect("Failed to create subagent");

    let mut stream = client
        .subagent_chat(subagent.id, "Please reply with the word 'subpong'.")
        .await
        .expect("Failed to start subagent chat");

    let mut got_message = false;
    let mut got_done = false;
    let mut all_messages: Vec<String> = Vec::new();

    while let Some(result) = stream.next().await {
        match result {
            Ok(event) => match event {
                AgentSseEvent::Message(msg) => {
                    got_message = true;
                    all_messages.extend(msg.messages);
                    println!("Subagent message at step {}: {:?}", msg.step, all_messages);
                }
                AgentSseEvent::Done(done) => {
                    got_done = true;
                    println!(
                        "Subagent done: {} steps, {} messages",
                        done.total_steps, done.total_messages
                    );
                }
                AgentSseEvent::Error(err) => {
                    panic!("Subagent error: {}", err.error);
                }
            },
            Err(e) => {
                panic!("Stream error: {:?}", e);
            }
        }
    }

    let deleted = client
        .delete_subagent(subagent.id)
        .await
        .expect("Failed to delete subagent after chat");

    assert!(deleted.deleted);
    assert!(
        got_message,
        "Should have received at least one subagent message"
    );
    assert!(got_done, "Should have received a subagent done event");
    assert!(
        !all_messages.is_empty(),
        "Should have at least one subagent message"
    );
}
