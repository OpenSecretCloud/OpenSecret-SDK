use opensecret_sdk::{OpenSecretClient, Result};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the client
    let client = OpenSecretClient::new("http://localhost:3000")?;

    // Your client ID (get this from your OpenSecret dashboard)
    let client_id = Uuid::parse_str("your-client-id-here").unwrap_or_else(|_| Uuid::new_v4());

    // Step 1: Perform attestation handshake (required before any encrypted calls)
    println!("Performing attestation handshake...");
    client.perform_attestation_handshake().await?;
    println!("✓ Secure session established");

    // Example 1: Register a new user with email
    println!("\n--- User Registration ---");
    let email = "user@example.com".to_string();
    let password = "secure_password_123".to_string();
    let name = Some("John Doe".to_string());

    match client
        .register(email.clone(), password.clone(), client_id, name)
        .await
    {
        Ok(response) => {
            println!("✓ User registered successfully!");
            println!("  User ID: {}", response.id);
            println!("  Email: {:?}", response.email);
            println!("  Access Token: {}...", &response.access_token[..20]);
        }
        Err(e) => {
            println!("Registration failed (user might already exist): {}", e);

            // Try logging in instead
            println!("\n--- User Login ---");
            match client.login(email.clone(), password, client_id).await {
                Ok(response) => {
                    println!("✓ Login successful!");
                    println!("  User ID: {}", response.id);
                    println!("  Access Token: {}...", &response.access_token[..20]);
                }
                Err(e) => println!("Login failed: {}", e),
            }
        }
    }

    // Example 2: Guest registration (no email required)
    println!("\n--- Guest Registration ---");

    // Logout first if logged in
    if client.get_access_token()?.is_some() {
        client.logout().await?;
        // Need new session after logout
        client.perform_attestation_handshake().await?;
    }

    let guest_password = "guest_password_456".to_string();
    match client
        .register_guest(guest_password.clone(), client_id)
        .await
    {
        Ok(response) => {
            println!("✓ Guest registered successfully!");
            println!("  Guest ID: {}", response.id);
            println!("  No email: {:?}", response.email);

            // Save the guest ID for future logins
            let guest_id = response.id;

            // Logout and login again as guest
            client.logout().await?;
            client.perform_attestation_handshake().await?;

            println!("\n--- Guest Login ---");
            match client
                .login_with_id(guest_id, guest_password, client_id)
                .await
            {
                Ok(_) => println!("✓ Guest login successful!"),
                Err(e) => println!("Guest login failed: {}", e),
            }
        }
        Err(e) => println!("Guest registration failed: {}", e),
    }

    // Example 3: Token refresh
    if client.get_access_token()?.is_some() {
        println!("\n--- Token Refresh ---");
        let old_token = client.get_access_token()?.unwrap();
        println!("Old access token: {}...", &old_token[..20]);

        client.refresh_token().await?;

        let new_token = client.get_access_token()?.unwrap();
        println!("New access token: {}...", &new_token[..20]);
        println!("✓ Token refreshed successfully!");
    }

    // Example 4: Logout
    if client.get_access_token()?.is_some() {
        println!("\n--- Logout ---");
        client.logout().await?;
        println!("✓ Logged out successfully");
        println!(
            "  Access token cleared: {}",
            client.get_access_token()?.is_none()
        );
        println!("  Session cleared: {}", client.get_session_id()?.is_none());
    }

    Ok(())
}
