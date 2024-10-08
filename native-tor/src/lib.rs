use anyhow::Result;
use futures::{AsyncReadExt, AsyncWriteExt};

use arti_client::{config::TorClientConfigBuilder, TorClient};
use tor_rtcompat::BlockOn;

pub fn run_arti(to: &str, cache: &str) -> Result<String> {
    let runtime = tor_rtcompat::PreferredRuntime::create()?;
    let rt_copy = runtime.clone();

    let config = TorClientConfigBuilder::from_directories(
        format!("{}/arti-data", cache),
        format!("{}/arti-cache", cache),
    )
    .build()?;
    rt_copy.block_on(async {
        let client = TorClient::with_runtime(runtime)
            .config(config)
            .create_bootstrapped().await?;

        let mut stream = client.connect((to, 443)).await?;
        stream.write_all(b"GET / HTTP/1.1\r\nHost: ").await?;
        stream.write_all(to.as_bytes()).await?;
        stream.write_all(b"\r\nConnection: close\r\n\r\n").await?;
        stream.close().await?;

        let mut res = Vec::new();
        stream.read_to_end(&mut res).await?;
        let message = std::str::from_utf8(&res)?;
        let start_index = message.find("\r\n\r\n").unwrap_or(0);

        Ok(message[start_index..].to_owned())
    })
}

/// Expose the JNI interface for Android
#[cfg(target_os = "android")]
pub mod android;

/// Expose the native interface for iOS
#[cfg(target_os = "ios")]
pub mod ios;
