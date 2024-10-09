#![allow(non_snake_case)]

use crate::run_arti;

use anyhow::Result;

use std::sync::Once;
use tracing_subscriber::fmt::Subscriber;
use tracing_subscriber::prelude::*;

use jni::objects::{JClass, JString};
use jni::sys::jstring;
use jni::JNIEnv;

/// Create a static method myMethod on class net.example.MyClass
#[no_mangle]
pub extern "C" fn Java_com_tor_TorModule_nativeConnectToTorNetwork(
    env: JNIEnv,
    _: JClass,
    target: JString,
    cache_dir: JString,
) -> jstring {
    // if logger initialization failed, there isn't much we can do, not even log it.
    // it shouldn't stop Arti from functionning however!
    let _ = init_logger();

    let result = match run_arti(
        &env.get_string(target)
            .expect("target is invalid")
            .to_string_lossy(),
        &env.get_string(cache_dir)
            .expect("cache_dir is invalid")
            .to_string_lossy(),
    ) {
        Ok(response) => format!(
            "Tor client initialized successfully. Test response: {}",
            response
        ),
        Err(e) => format!("Tor Error: {}. Cause: {:?}", e, e.root_cause()),
    };

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
