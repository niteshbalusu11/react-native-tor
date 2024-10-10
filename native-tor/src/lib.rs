use anyhow::{Context, Result};
use arti_client::{
    config::{onion_service::OnionServiceConfigBuilder, TorClientConfigBuilder},
    TorClient,
};
use std::net::SocketAddr;
use std::sync::{LazyLock, Mutex};
use std::{fs, os::unix::fs::PermissionsExt};
use tokio::io::AsyncReadExt as TokioAsyncReadExt;
use tokio::net::TcpListener;
use tokio_socks::tcp::Socks5Stream;
use tor_hsservice::HsNickname;
use tor_rtcompat::BlockOn;

static TOR_CLIENT: LazyLock<Mutex<Option<TorClient<tor_rtcompat::PreferredRuntime>>>> =
    LazyLock::new(|| Mutex::new(None));
static RUNTIME: LazyLock<tor_rtcompat::PreferredRuntime> =
    LazyLock::new(|| tor_rtcompat::PreferredRuntime::create().unwrap());

pub fn run_arti_proxy(_target: &str, cache: &str) -> Result<String> {
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

    // Onion service setup (if needed)
    let hs_nickname = OnionServiceConfigBuilder::default()
        .nickname(HsNickname::new("blixt".to_string())?)
        .build()
        .context("Failed to create OnionServiceConfig")?;

    let (onion_service, _request_stream) = client.launch_onion_service(hs_nickname)?;
    println!(
        "Onion service launched. Address: {:?}",
        onion_service.onion_name()
    );

    // Run the SOCKS proxy on port 9050
    let socks_address = SocketAddr::from(([127, 0, 0, 1], 9050));
    println!("Starting SOCKS proxy on {:?}", socks_address);

    RUNTIME.block_on(async {
        run_socks5_proxy(socks_address, &client)
            .await
            .context("Failed to run SOCKS proxy")
    })?;

    // Store the client
    let mut tor_client = TOR_CLIENT.lock().unwrap();
    *tor_client = Some(client);

    let onion_name = onion_service.onion_name().unwrap().to_string();

    Ok(onion_name)
}

async fn run_socks5_proxy(
    addr: SocketAddr,
    client: &TorClient<tor_rtcompat::PreferredRuntime>, // Use reference to avoid move
) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;
    println!("SOCKS5 proxy listening on {:?}", addr);

    loop {
        let (socket, _) = listener.accept().await?;
        let client_clone = client.clone();

        tokio::spawn(async move {
            // Accept connections and forward through the SOCKS proxy
            let socks5_stream =
                match Socks5Stream::connect("127.0.0.1:9050", "example.com:80").await {
                    Ok(stream) => stream,
                    Err(e) => {
                        eprintln!("Failed to connect through SOCKS5: {:?}", e);
                        return;
                    }
                };

            // Forward the SOCKS5 stream to the Tor network
            let mut response = Vec::new();
            let mut inner_stream = socks5_stream.into_inner();
            inner_stream.read_to_end(&mut response).await.unwrap();
            println!("Response: {:?}", String::from_utf8_lossy(&response));
        });
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
