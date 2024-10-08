// File: src/main.rs

use arti_client::{TorClient, TorClientConfig};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging

    // Load and print the configuration
    let config = TorClientConfig::builder().build().unwrap();
    let tor_client = TorClient::create_bootstrapped(config.clone()).await?;

    // Print the configuration
    println!("Tor Configuration: {:#?}", config);

    // Try to create and bootstrap a TorClient
    println!("Attempting to create and bootstrap TorClient...");
    match TorClient::create_bootstrapped(config).await {
        Ok(_) => println!("Successfully created and bootstrapped TorClient!"),
        Err(e) => println!("Failed to create and bootstrap TorClient: {:?}", e),
    }

    Ok(())
}
