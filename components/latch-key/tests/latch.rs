//! The tests of `latch-key`.

use std::collections::HashMap;

use test_harness::{
    ConfigError, ErrorCode, Host, Latch, Level, Log,
    exported::{Decision, LatchError},
};

async fn latch(config: &[(&str, &str)]) -> Latch {
    let config = config.iter().map(|(key, value)| (key.to_string(), value.to_string())).collect();
    Latch::new("latch-key", Host { config: Ok(config), ..Host::default() }).await.unwrap()
}

const DENIED: Decision = Decision::Denied(ErrorCode::NotFound);

#[tokio::test]
async fn no_config_defers_every_key() {
    let mut latch = latch(&[]).await;
    assert_eq!(latch.authorize_get("password").await.unwrap(), Ok(Decision::Deferred));
}

#[tokio::test]
async fn denied_key_is_not_found() {
    let mut latch = latch(&[("password", "denied")]).await;
    assert_eq!(latch.authorize_get("password").await.unwrap(), Ok(DENIED));
    assert_eq!(latch.authorize_get("other").await.unwrap(), Ok(Decision::Deferred));
}

#[tokio::test]
async fn allowlist() {
    // the example of the README
    let mut latch = latch(&[("db.*", "deferred"), ("db.admin", "denied"), ("*", "denied")]).await;
    assert_eq!(latch.authorize_get("db.password").await.unwrap(), Ok(Decision::Deferred));
    assert_eq!(latch.authorize_get("db.admin").await.unwrap(), Ok(DENIED));
    assert_eq!(latch.authorize_get("api.token").await.unwrap(), Ok(DENIED));
}

#[tokio::test]
async fn empty_value_is_deferred() {
    let mut latch = latch(&[("*", "denied"), ("password", "")]).await;
    assert_eq!(latch.authorize_get("password").await.unwrap(), Ok(Decision::Deferred));
}

#[tokio::test]
async fn equally_specific_patterns_deny() {
    let mut latch = latch(&[("a*", "deferred"), ("*a", "denied")]).await;
    assert_eq!(latch.authorize_get("aa").await.unwrap(), Ok(DENIED));
    assert_eq!(latch.authorize_get("ab").await.unwrap(), Ok(Decision::Deferred));
}

#[tokio::test]
async fn invalid_value_is_invalid_config() {
    let mut latch = latch(&[("db.*", "allowed")]).await;
    assert_eq!(
        latch.authorize_get("db.password").await.unwrap(),
        Err(LatchError::InvalidConfig("latch-key".to_string()))
    );
    assert_eq!(
        latch.store.data().logs,
        vec![Log {
            level: Level::Critical,
            context: "componentized-latch".to_string(),
            message: "Invalid config LATCH=latch-key KEY=db.* VALUE=allowed ERROR=expected one of: 'deferred', 'denied'"
                .to_string(),
        }]
    );
}

#[tokio::test]
async fn config_error_is_invalid_config() {
    let host = Host { config: Err(ConfigError::Upstream("unavailable".to_string())), ..Host::default() };
    let mut latch = Latch::new("latch-key", host).await.unwrap();
    assert_eq!(
        latch.authorize_get("password").await.unwrap(),
        Err(LatchError::InvalidConfig("latch-key".to_string()))
    );
    assert_eq!(
        latch.store.data().logs[0].message,
        "Invalid config LATCH=latch-key ERROR=upstream: unavailable"
    );
}

#[tokio::test]
async fn config_is_read_once() {
    let mut latch = latch(&[]).await;
    assert_eq!(latch.authorize_get("password").await.unwrap(), Ok(Decision::Deferred));
    latch.store.data_mut().config = Ok(HashMap::from([("password".to_string(), "denied".to_string())]));
    assert_eq!(latch.authorize_get("password").await.unwrap(), Ok(Decision::Deferred));
    assert_eq!(latch.store.data().get_all_calls, 1);
}

#[tokio::test]
async fn invalid_config_fails_every_operation_and_is_logged_once() {
    let mut latch = latch(&[("db.*", "allowed")]).await;
    let invalid = Err(LatchError::InvalidConfig("latch-key".to_string()));
    assert_eq!(latch.authorize_get("db.password").await.unwrap(), invalid);
    // fixing the config after it was read changes nothing
    latch.store.data_mut().config = Ok(HashMap::new());
    assert_eq!(latch.authorize_get("other").await.unwrap(), invalid);
    assert_eq!(latch.store.data().get_all_calls, 1);
    assert_eq!(latch.store.data().logs.len(), 1);
}

#[tokio::test]
async fn observes_without_error() {
    let mut latch = latch(&[]).await;
    assert_eq!(latch.observe_get(&DENIED, "password").await.unwrap(), Ok(()));
    assert_eq!(latch.store.data().get_all_calls, 0);
}
