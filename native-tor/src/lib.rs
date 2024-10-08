use arti_client::{TorClient, TorClientConfig};
use jni::objects::JClass;
use jni::sys::jstring;
use jni::JNIEnv;
use tokio::runtime::Builder;
use anyhow::Result;

#[no_mangle]
pub extern "C" fn Java_com_tor_TorModule_nativeMyTorMethod(env: JNIEnv, _: JClass) -> jstring {
    // Create a new Tokio runtime
    let runtime = match Builder::new_current_thread().enable_all().build() {
        Ok(rt) => rt,
        Err(e) => {
            let error_msg = format!("Failed to create Tokio runtime: {}", e);
            return env.new_string(error_msg).expect("Couldn't create java string!").into_inner();
        }
    };

    // Run the async block in the runtime
    let result = runtime.block_on(async {
        bootstrap_tor().await
    });

    // Convert the result to a Java string
    match result {
        Ok(message) => env.new_string(message).expect("Couldn't create java string!").into_inner(),
        Err(e) => {
            let error_msg = format!("Error: {}", e);
            env.new_string(error_msg).expect("Couldn't create java string!").into_inner()
        }
    }
}

async fn bootstrap_tor() -> Result<String> {
    let config = TorClientConfig::builder().build().unwrap();

    println!("tor config {:?}", config);
    let tor_client = TorClient::create_bootstrapped(config).await?;

    // You can add more Tor-related operations here if needed

    Ok("Successfully connected to Tor network!".to_string())
}
