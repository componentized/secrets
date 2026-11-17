//! The tests of `from-config`, plugged into `test-consumer` as `test-from-config`.

use std::collections::HashMap;

use crate::host::{ConfigError, Consumer, ErrorCode, Host, Secret};

async fn consumer(config: Result<HashMap<String, String>, ConfigError>) -> Consumer {
    Consumer::new("test-from-config", Host { config, ..Host::default() }).await.unwrap()
}

fn config() -> Result<HashMap<String, String>, ConfigError> {
    Ok(HashMap::from([("password".to_string(), "hunter2".to_string())]))
}

#[tokio::test]
async fn get_before_init_is_not_initialized() {
    let mut consumer = consumer(config()).await;
    assert!(matches!(consumer.get("password").await.unwrap(), Err(ErrorCode::NotInitialized)));
}

#[tokio::test]
async fn first_init_does_not_resolve_immediately() {
    let mut consumer = consumer(config()).await;
    let call = consumer.init().await.unwrap();
    assert!(!call.immediate, "the first init resolved on its first poll");
    assert!(call.result.is_ok());
}

#[tokio::test]
async fn init_after_init_resolves_immediately() {
    let mut consumer = consumer(config()).await;
    consumer.init().await.unwrap();
    let call = consumer.init().await.unwrap();
    assert!(call.immediate, "an init after the store is initialized waited");
    assert!(call.result.is_ok());
    assert_eq!(consumer.store.data().get_all_calls, 1);
}

#[tokio::test]
async fn concurrent_inits_read_the_config_once() {
    let mut consumer = consumer(config()).await;
    let (first, second) = consumer.init_concurrently().await.unwrap();
    assert!(first.result.is_ok());
    assert!(second.result.is_ok());
    assert_eq!(consumer.store.data().get_all_calls, 1);
}

#[tokio::test]
async fn get_is_the_config_value() {
    let mut consumer = consumer(config()).await;
    consumer.init().await.unwrap();
    assert!(matches!(
        consumer.get("password").await.unwrap(),
        Ok(Secret::String(value)) if value == "hunter2"
    ));
}

#[tokio::test]
async fn get_missing_key_is_not_found() {
    let mut consumer = consumer(config()).await;
    consumer.init().await.unwrap();
    assert!(matches!(consumer.get("missing").await.unwrap(), Err(ErrorCode::NotFound)));
}

#[tokio::test]
async fn config_changed_after_init_is_not_seen() {
    let mut consumer = consumer(config()).await;
    consumer.init().await.unwrap();
    consumer.store.data_mut().config = Ok(HashMap::from([("password".to_string(), "changed".to_string())]));
    assert!(matches!(
        consumer.get("password").await.unwrap(),
        Ok(Secret::String(value)) if value == "hunter2"
    ));
}

#[tokio::test]
async fn config_error_fails_init() {
    let mut consumer = consumer(Err(ConfigError::Upstream("unavailable".to_string()))).await;
    let call = consumer.init().await.unwrap();
    assert!(matches!(
        call.result,
        Err(ErrorCode::Other(Some(message))) if message == "upstream: unavailable"
    ));
    assert!(matches!(consumer.get("password").await.unwrap(), Err(ErrorCode::NotInitialized)));
}

#[tokio::test]
async fn init_after_failed_init_is_the_same_error() {
    let mut consumer = consumer(Err(ConfigError::Io("unreadable".to_string()))).await;
    consumer.init().await.unwrap();
    consumer.store.data_mut().config = config();
    let call = consumer.init().await.unwrap();
    assert!(call.immediate, "an init after a failed init waited");
    assert!(matches!(
        call.result,
        Err(ErrorCode::Other(Some(message))) if message == "io: unreadable"
    ));
    assert_eq!(consumer.store.data().get_all_calls, 1);
}
