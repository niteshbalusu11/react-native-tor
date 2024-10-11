use crate::run_arti_proxy;
use jni::objects::{JClass, JString};
use jni::sys::jstring;
use jni::JNIEnv;
use std::thread;
use tokio::runtime::Runtime;

// Import android_logger
use android_logger::Config as AndroidLogConfig;

#[no_mangle]
pub extern "C" fn Java_com_tor_TorModule_nativeConnectToTorNetwork(
    env: JNIEnv,
    _: JClass,
    target: JString,
    cache_dir: JString,
) -> jstring {
    let cache_dir_str = env
        .get_string(cache_dir)
        .expect("cache_dir is invalid")
        .to_string_lossy()
        .to_string();

    let target_str = env
        .get_string(target)
        .expect("target is invalid")
        .to_string_lossy()
        .to_string();

    // Initialize the Android logger
    android_logger::init_once(
        AndroidLogConfig::default()
            .with_min_level(log::Level::Info) // Set the minimum log level
            .with_tag("TorModule"), // Set a custom tag
    );

    // Set a panic hook to log panics
    std::panic::set_hook(Box::new(|panic_info| {
        log::error!("Panic occurred: {:?}", panic_info);
    }));

    thread::spawn(move || {
        let rt = Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async {
            match run_arti_proxy(&target_str, &cache_dir_str).await {
                Ok(response) => {
                    log::info!(
                        "Tor client initialized successfully. Response: {}",
                        response
                    );
                }
                Err(e) => {
                    log::error!("Tor Error: {}. Cause: {:?}", e, e.root_cause());
                }
            }
        });
    });

    let result = "Tor client is starting in the background".to_string();
    let output = env
        .new_string(result)
        .expect("failed to create java string");

    output.into_inner()
}
