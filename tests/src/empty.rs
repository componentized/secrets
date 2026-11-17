//! The tests of `empty`, plugged into `test-consumer` as `test-empty`.

use crate::host::{Consumer, ErrorCode, Host};

async fn consumer() -> Consumer {
    Consumer::new("test-empty", Host::default()).await.unwrap()
}

#[tokio::test]
async fn get_before_init_is_not_initialized() {
    let mut consumer = consumer().await;
    assert!(matches!(consumer.get("key").await.unwrap(), Err(ErrorCode::NotInitialized)));
}

#[tokio::test]
async fn first_init_does_not_resolve_immediately() {
    let mut consumer = consumer().await;
    let call = consumer.init().await.unwrap();
    assert!(!call.immediate, "the first init resolved on its first poll");
    assert!(call.result.is_ok());
}

#[tokio::test]
async fn init_after_init_resolves_immediately() {
    let mut consumer = consumer().await;
    consumer.init().await.unwrap();
    let call = consumer.init().await.unwrap();
    assert!(call.immediate, "an init after the store is initialized waited");
    assert!(call.result.is_ok());
}

#[tokio::test]
async fn get_after_init_is_not_found() {
    let mut consumer = consumer().await;
    consumer.init().await.unwrap();
    assert!(matches!(consumer.get("key").await.unwrap(), Err(ErrorCode::NotFound)));
}
