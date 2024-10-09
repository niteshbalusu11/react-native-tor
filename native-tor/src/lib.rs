use anyhow::{Result, Context, Error};
use arti_client::{config::TorClientConfigBuilder, TorClient};
use futures::{AsyncReadExt, AsyncWriteExt};
use std::sync::{LazyLock, Mutex};
use tor_rtcompat::BlockOn;
use std::fs;

static TOR_CLIENT: LazyLock<Mutex<Option<TorClient<tor_rtcompat::PreferredRuntime>>>> =
    LazyLock::new(|| Mutex::new(None));
static RUNTIME: LazyLock<tor_rtcompat::PreferredRuntime> =
    LazyLock::new(|| tor_rtcompat::PreferredRuntime::create().unwrap());

pub fn run_arti(test_url: &str, cache: &str) -> Result<String> {
    // Ensure directories exist
    fs::create_dir_all(format!("{}/arti-data", cache))
        .context("Failed to create arti-data directory")?;
    fs::create_dir_all(format!("{}/arti-cache", cache))
        .context("Failed to create arti-cache directory")?;

    let config = TorClientConfigBuilder::from_directories(
        format!("{}/arti-data", cache),
        format!("{}/arti-cache", cache),
    )
    .build()
    .context("Failed to build TorClientConfig")?;

    let client = RUNTIME.block_on(async {
        TorClient::with_runtime(RUNTIME.clone())
            .config(config)
            .create_bootstrapped()
            .await
    }).context("Failed to create and bootstrap TorClient")?;

    // Test the connection
    let test_result: Result<String, Error> = RUNTIME.block_on(async {
        let mut stream = client.connect((test_url, 443)).await?;
        let request = format!(
            "GET / HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            test_url
        );

        stream.write_all(request.as_bytes()).await?;
        stream.flush().await?;

        let mut response = Vec::new();
        stream.read_to_end(&mut response).await?;

        Ok(String::from_utf8_lossy(&response).to_string())
    });

    // Store the client only if the test was successful
    match test_result {
        Ok(response) => {
            let mut tor_client = TOR_CLIENT.lock().unwrap();
            *tor_client = Some(client);
            Ok(response)
        }
        Err(e) => Err(e).context("Failed during connection test"),
    }
}

/// Expose the JNI interface for Android
#[cfg(target_os = "android")]
pub mod android;

/// Expose the native interface for iOS
#[cfg(target_os = "ios")]
pub mod ios;
