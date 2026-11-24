//! The tests of `to-wasmcloud`, plugged with `from-config` as `test-to-wasmcloud`.

use std::collections::HashMap;

use test_harness::wasmcloud::{SecretValue, SecretsError};
use test_harness::{ConfigError, Host, Wasmcloud};

async fn wasmcloud(config: Result<HashMap<String, String>, ConfigError>) -> Wasmcloud {
    Wasmcloud::new(
        "test-to-wasmcloud",
        Host {
            config,
            ..Host::default()
        },
    )
    .await
    .unwrap()
}

fn config() -> Result<HashMap<String, String>, ConfigError> {
    Ok(HashMap::from([
        ("password".to_string(), "hunter2".to_string()),
        ("token".to_string(), "s3cr3t".to_string()),
    ]))
}

#[tokio::test]
async fn get_reveals_the_secret() {
    let mut wasmcloud = wasmcloud(config()).await;
    assert!(matches!(
        wasmcloud.get("password").await.unwrap(),
        Ok(SecretValue::String(value)) if value == "hunter2"
    ));
}

#[tokio::test]
async fn get_missing_key_is_not_found() {
    let mut wasmcloud = wasmcloud(config()).await;
    assert!(matches!(
        wasmcloud.get("missing").await.unwrap(),
        Err(SecretsError::NotFound)
    ));
}

#[tokio::test]
async fn gets_initialize_the_store_once() {
    let mut wasmcloud = wasmcloud(config()).await;
    wasmcloud.get("password").await.unwrap().unwrap();
    assert!(matches!(
        wasmcloud.get("token").await.unwrap(),
        Ok(SecretValue::String(value)) if value == "s3cr3t"
    ));
    assert_eq!(wasmcloud.store.data().get_all_calls, 1);
}

#[tokio::test]
async fn concurrent_gets_initialize_the_store_once() {
    let mut wasmcloud = wasmcloud(config()).await;
    let (first, second) = wasmcloud
        .get_concurrently("password", "token")
        .await
        .unwrap();
    assert!(matches!(first, Ok(SecretValue::String(value)) if value == "hunter2"));
    assert!(matches!(second, Ok(SecretValue::String(value)) if value == "s3cr3t"));
    assert_eq!(wasmcloud.store.data().get_all_calls, 1);
}

#[tokio::test]
async fn store_error_is_upstream() {
    let mut wasmcloud = wasmcloud(Err(ConfigError::Upstream("unavailable".to_string()))).await;
    assert!(matches!(
        wasmcloud.get("password").await.unwrap(),
        Err(SecretsError::Upstream(message)) if message == "upstream: unavailable"
    ));
}
