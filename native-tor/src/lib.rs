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
        let connect_string = format!("GET / HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n", to);

        stream
            .write_all(connect_string.as_bytes())
            .await?;

        // IMPORTANT: Make sure the request was written.
        // Arti buffers data, so flushing the buffer is usually required.
        stream.flush().await?;

        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await?;

        stream.close().await?;

        Ok(String::from_utf8_lossy(&buf).to_string())
    })
}

/// Expose the JNI interface for Android
#[cfg(target_os = "android")]
pub mod android;

/// Expose the native interface for iOS
#[cfg(target_os = "ios")]
pub mod ios;
