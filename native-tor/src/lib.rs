use anyhow::{anyhow, Context, Result};
use arti_client::config::TorClientConfigBuilder;
use arti_client::TorClient;
use futures::future::try_join;
use log::{error, info};
use std::{fs, os::unix::fs::PermissionsExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream}; // Import logging macros

pub async fn run_arti_proxy(_target: &str, cache: &str) -> Result<String> {
    let data_dir = format!("{}/arti-data", cache);
    let cache_dir = format!("{}/arti-cache", cache);
    create_and_set_permissions(&data_dir)?;
    create_and_set_permissions(&cache_dir)?;

    // Create a TorClientConfig using TorClientConfigBuilder
    let config = TorClientConfigBuilder::from_directories(data_dir, cache_dir).build()?;

    let tor_client = TorClient::create_bootstrapped(config).await?;

    // Start the SOCKS proxy in the background
    let socks_port = 9050; // Change this to your desired port
    let listen_addr = format!("127.0.0.1:{}", socks_port);
    let tor_client_clone = tor_client.clone();
    tokio::spawn(async move {
        if let Err(e) = run_socks_proxy(tor_client_clone, &listen_addr).await {
            error!("SOCKS proxy exited with error: {}", e);
        }
    });

    Ok("Arti Tor proxy started successfully".to_string())
}

async fn run_socks_proxy<R>(tor_client: TorClient<R>, listen_addr: &str) -> Result<()>
where
    R: tor_rtcompat::Runtime,
{
    let listener = TcpListener::bind(&listen_addr)
        .await
        .context(format!("Failed to bind to {}", listen_addr))?;
    info!("SOCKS proxy listening on {}", listen_addr);

    loop {
        let (stream, addr) = listener
            .accept()
            .await
            .context("Failed to accept incoming connection")?;
        let tor_client_clone = tor_client.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_socks5_client(stream, tor_client_clone).await {
                error!("Failed to handle SOCKS connection from {}: {}", addr, e);
            }
        });
    }
}

async fn handle_socks5_client<R>(
    mut client_stream: TcpStream,
    tor_client: TorClient<R>,
) -> Result<()>
where
    R: tor_rtcompat::Runtime,
{
    // Implement minimal SOCKS5 handshake
    let mut buf = [0u8; 262];

    // Read the SOCKS5 greeting
    let n = client_stream
        .read(&mut buf)
        .await
        .context("Failed to read SOCKS5 greeting")?;
    if n < 2 {
        return Err(anyhow!("Invalid SOCKS5 greeting"));
    }

    if buf[0] != 0x05 {
        return Err(anyhow!("Only SOCKS5 is supported"));
    }

    // Send the authentication method response (no authentication)
    client_stream
        .write_all(&[0x05, 0x00])
        .await
        .context("Failed to write SOCKS5 method selection")?;

    // Read the SOCKS5 request
    let n = client_stream
        .read(&mut buf)
        .await
        .context("Failed to read SOCKS5 request")?;
    if n < 5 {
        return Err(anyhow!("Invalid SOCKS5 request"));
    }

    if buf[0] != 0x05 {
        return Err(anyhow!("Invalid SOCKS5 version in request"));
    }

    if buf[1] != 0x01 {
        return Err(anyhow!("Only CONNECT command is supported"));
    }

    let addr = match buf[3] {
        0x01 => {
            // IPv4
            if n < 10 {
                return Err(anyhow!("Invalid IPv4 address in SOCKS5 request"));
            }
            let ip = std::net::Ipv4Addr::new(buf[4], buf[5], buf[6], buf[7]);
            let port = u16::from_be_bytes([buf[8], buf[9]]);
            (ip.to_string(), port)
        }
        0x03 => {
            // Domain name
            let domain_len = buf[4] as usize;
            if n < 5 + domain_len + 2 {
                return Err(anyhow!("Invalid domain name in SOCKS5 request"));
            }
            let domain = std::str::from_utf8(&buf[5..5 + domain_len])
                .context("Failed to parse domain name")?;
            let port = u16::from_be_bytes([buf[5 + domain_len], buf[6 + domain_len]]);
            (domain.to_string(), port)
        }
        0x04 => {
            // IPv6
            if n < 22 {
                return Err(anyhow!("Invalid IPv6 address in SOCKS5 request"));
            }
            let ip = std::net::Ipv6Addr::from([
                buf[4], buf[5], buf[6], buf[7], buf[8], buf[9], buf[10], buf[11], buf[12], buf[13],
                buf[14], buf[15], buf[16], buf[17], buf[18], buf[19],
            ]);
            let port = u16::from_be_bytes([buf[20], buf[21]]);
            (ip.to_string(), port)
        }
        _ => return Err(anyhow!("Unsupported address type in SOCKS5 request")),
    };

    info!("Connecting to {}:{}", addr.0, addr.1);

    // Send the SOCKS5 response (success)
    let response = [0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0, 0, 0]; // BND.ADDR and BND.PORT are set to zero
    client_stream
        .write_all(&response)
        .await
        .context("Failed to write SOCKS5 response")?;

    // Now, create a connection through Tor
    let tor_stream = tor_client
        .connect(addr)
        .await
        .context("Failed to connect through Tor")?;

    // Relay data between client_stream and tor_stream
    let (mut ri, mut wi) = tokio::io::split(client_stream);
    let (mut ro, mut wo) = tokio::io::split(tor_stream);

    let client_to_tor = tokio::io::copy(&mut ri, &mut wo);
    let tor_to_client = tokio::io::copy(&mut ro, &mut wi);

    try_join(client_to_tor, tor_to_client).await?;

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
