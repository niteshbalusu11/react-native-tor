use anyhow::{Context, Error, Result};
use arti_client::{
    config::{onion_service::OnionServiceConfigBuilder, TorClientConfigBuilder},
    TorClient,
};
use futures::{AsyncReadExt, AsyncWriteExt};
use std::sync::{LazyLock, Mutex};
use std::{fs, os::unix::fs::PermissionsExt};
use tor_hsservice::HsNickname;
use tor_rtcompat::BlockOn;

static TOR_CLIENT: LazyLock<Mutex<Option<TorClient<tor_rtcompat::PreferredRuntime>>>> =
    LazyLock::new(|| Mutex::new(None));
static RUNTIME: LazyLock<tor_rtcompat::PreferredRuntime> =
    LazyLock::new(|| tor_rtcompat::PreferredRuntime::create().unwrap());

pub fn run_arti(test_url: &str, cache: &str) -> Result<String> {
    // Ensure directories exist with correct permissions
    create_and_set_permissions(&format!("{}/arti-data", cache))?;
    create_and_set_permissions(&format!("{}/arti-cache", cache))?;

    let config = TorClientConfigBuilder::from_directories(
        format!("{}/arti-data", cache),
        format!("{}/arti-cache", cache),
    )
    .build()
    .context("Failed to build TorClientConfig")?;

    let client = RUNTIME
        .block_on(async {
            TorClient::with_runtime(RUNTIME.clone())
                .config(config)
                .create_bootstrapped()
                .await
        })
        .context("Failed to create and bootstrap TorClient")?;

    // Test the connection
    let test_result: Result<String, Error> = RUNTIME.block_on(async {
        println!("inside test conncection");
        let mut stream = client.connect((test_url, 80)).await?;
        let request = format!(
            "GET / HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            test_url
        );

        stream.write_all(request.as_bytes()).await?;
        stream.flush().await?;

        let mut response = Vec::new();
        stream.read_to_end(&mut response).await?;
        println!("{:?}", String::from_utf8_lossy(&response).to_string());

        Ok(String::from_utf8_lossy(&response).to_string())
    });

    println!("test result is {:?}", test_result);

    let hs_nickname =
        HsNickname::new("blixt".to_string()).context("Failed to create HsNickname")?;

    let onion_config = OnionServiceConfigBuilder::default()
        .nickname(hs_nickname)
        .build()?;

    let (onion_service, mut _request_stream) = client.launch_onion_service(onion_config)?;

    println!(
        "Onion service launched. Address: {:?}",
        onion_service.onion_name()
    );

    // RUNTIME.block_on(async {
    //     while let Some(request) = request_stream.next().await {
    //         tokio::spawn(async move {
    //             // Handle the request here
    //             println!("Received request: {:?}", request);
    //             // You would typically process the request and send a response here
    //         });
    //     }
    // });

    // Store the client only if the test was successful
    match test_result {
        Ok(response) => {
            let mut tor_client = TOR_CLIENT.lock().unwrap();
            *tor_client = Some(client);
            let res = format!("{}{:?}", response, onion_service.onion_name());
            Ok(res)
        }
        Err(e) => Err(e).context("Failed during connection test"),
    }
}

fn create_and_set_permissions(path: &str) -> Result<()> {
    fs::create_dir_all(path).context(format!("Failed to create directory: {}", path))?;
    let metadata = fs::metadata(path).context(format!("Failed to get metadata for: {}", path))?;
    let mut permissions = metadata.permissions();
    permissions.set_mode(0o700); // This sets permissions to rwx------
    fs::set_permissions(path, permissions)
        .context(format!("Failed to set permissions for: {}", path))?;
    Ok(())
}

/// Expose the JNI interface for Android
#[cfg(target_os = "android")]
pub mod android;

/// Expose the native interface for iOS
#[cfg(target_os = "ios")]
pub mod ios;
