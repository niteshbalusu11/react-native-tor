use android_logger::Config as AndroidLogConfig;
use anyhow::{Context, Result};
use arti::{run, ArtiConfigBuilder};
use arti_client::config::onion_service::OnionServiceConfigBuilder;
use arti_client::config::TorClientConfigBuilder;
use arti_client::TorClient;
use futures::StreamExt;
use log::{error, info};
use std::{fs, os::unix::fs::PermissionsExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::oneshot::Sender;
use tor_cell::relaycell::msg::Connected;
use tor_config::{ConfigurationSources, Listen};
use tor_hsservice::{HsNickname, RendRequest};
use tor_rtcompat::tokio::TokioRustlsRuntime;

// Modify the function signature to accept the Sender
pub async fn run_arti_proxy(_target: &str, cache: &str, ready_tx: Sender<()>) -> Result<String> {
    // Initialize the Android logger
    android_logger::init_once(
        AndroidLogConfig::default()
            .with_min_level(log::Level::Info) // Set the minimum log level
            .with_tag("TorModule"), // Set a custom tag
    );

    log::info!("Tor Socks.. Inside run_arti_proxy");

    let data_dir = format!("{}/arti-data", cache);
    let cache_dir = format!("{}/arti-cache", cache);
    create_and_set_permissions(&data_dir)?;
    create_and_set_permissions(&cache_dir)?;

    let arti_config = ArtiConfigBuilder::default().build()?;

    log::info!("Tor Socks.. ArtiConfigBuilder setup complete");

    let client_config_builder = TorClientConfigBuilder::from_directories(&data_dir, &cache_dir);
    let client_config = client_config_builder.build()?;

    log::info!("Tor Socks.. TorClientConfigBuilder setup complete");

    let runtime = TokioRustlsRuntime::create()?;

    log::info!("Tor Socks.. TokioRustlsRuntime setup complete");

    let socks_listen = Listen::new_localhost(9050);
    let dns_listen = Listen::new_none();
    let config_sources = ConfigurationSources::default();

    log::info!("Tor Socks.. Setting up socks proxy");

    run(
        runtime,
        socks_listen,
        dns_listen,
        config_sources,
        arti_config,
        client_config,
    )
    .await?;

    log::info!("Tor Socks.. Socks proxy running on 9050");

    // Create a TorClientConfig using TorClientConfigBuilder
    let config_builder = TorClientConfigBuilder::from_directories(&data_dir, &cache_dir);

    // Set up the onion service configuration
    let hs_config = OnionServiceConfigBuilder::default()
        .nickname(HsNickname::new("blixt".to_string())?)
        .build()
        .context("Failed to build OnionServiceConfig")?;

    let config = config_builder.build()?;

    let tor_client = TorClient::create_bootstrapped(config).await?;

    // Launch the onion service
    let (onion_service, onion_service_request_stream) = tor_client
        .launch_onion_service(hs_config)
        .context("Failed to launch onion service")?;

    let onion_address = onion_service.onion_name();
    info!("Onion service launched at: {:?}", onion_address);

    // Handle incoming onion service connections
    tokio::spawn(async move {
        if let Err(e) = handle_onion_service_connections(onion_service_request_stream).await {
            error!("Onion service error: {}", e);
        }
    });

    let _ = ready_tx.send(());

    Ok(format!(
        "Arti Tor proxy and onion service started successfully. Onion address: {:?}",
        onion_address
    ))
}

async fn handle_onion_service_connections(
    mut rend_request_stream: impl StreamExt<Item = RendRequest> + Unpin,
) -> Result<()> {
    while let Some(rend_request) = rend_request_stream.next().await {
        // Accept the rendezvous request
        let mut stream_request_stream = rend_request.accept().await?;

        // Handle the stream requests
        while let Some(stream_request) = stream_request_stream.next().await {
            // Accept the stream request to get a DataStream

            let mut data_stream = stream_request.accept(Connected::new_empty()).await?;

            // Spawn a task to handle the data stream
            tokio::spawn(async move {
                if let Err(e) = process_onion_stream(&mut data_stream).await {
                    error!("Error processing onion stream: {}", e);
                }
            });
        }
    }
    Ok(())
}

async fn process_onion_stream(
    stream: &mut (impl AsyncReadExt + AsyncWriteExt + Unpin),
) -> Result<()> {
    // Implement your service logic here.
    // For example, read data from the stream and respond.
    let mut buf = [0u8; 1024];
    let n = stream.read(&mut buf).await?;
    info!("Received data: {:?}", &buf[..n]);

    // Echo back the data
    stream.write_all(&buf[..n]).await?;
    Ok(())
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
