#![allow(non_snake_case)]

use crate::run_arti_proxy;
use anyhow::Result;
use jni::objects::{JClass, JString};
use jni::sys::jstring;
use jni::JNIEnv;
use std::sync::Once;
use std::thread;
use tracing_subscriber::fmt::Subscriber;
use tracing_subscriber::prelude::*;

/// Create a static method nativeConnectToTorNetwork on class com.tor.TorModule
#[no_mangle]
pub extern "C" fn Java_com_tor_TorModule_nativeConnectToTorNetwork(
    env: JNIEnv,
    _: JClass,
    target: JString, // Not used, can be removed later if not needed
    cache_dir: JString,
) -> jstring {
    // Initialize the logger
    let _ = init_logger();

    // Fetch the cache directory from the passed JString
    let cache_dir_str = env
        .get_string(cache_dir)
        .expect("cache_dir is invalid")
        .to_string_lossy()
        .to_string(); // Clone to move into a new thread

    let target_str = env
        .get_string(target)
        .expect("target is invalid")
        .to_string_lossy()
        .to_string(); // Clone to move into a new thread

    // Spawn a new thread to run the Tor client in the background
    thread::spawn(move || match run_arti_proxy(&target_str, &cache_dir_str) {
        Ok(response) => {
            println!(
                "Tor client initialized successfully. Onion address: {}",
                response
            );
        }
        Err(e) => {
            eprintln!("Tor Error: {}. Cause: {:?}", e, e.root_cause());
        }
    });

    // Immediately return a success message without waiting for the Tor client to finish
    let result = "Tor client is starting in the background".to_string();
    let output = env
        .new_string(result)
        .expect("failed to create java string");

    output.into_inner()
}

static LOGGER: Once = Once::new();

fn init_logger() -> Result<()> {
    if LOGGER.is_completed() {
        let layer = tracing_android::layer("rust.arti")?;
        LOGGER.call_once(|| Subscriber::new().with(layer).init());
    }
    Ok(())
}
