//! The tests of `gate`, called by `test-consumer` and gating `from-config`, as `test-gate`. The
//! latch is the host's [`FakeLatch`](test_harness::FakeLatch).

use std::collections::HashMap;

use test_harness::{get, Consumer, Decision, ErrorCode, Host, LatchError, Level, Log, Secret};

async fn consumer(host: Host) -> Consumer {
    Consumer::new("test-gate", host).await.unwrap()
}

fn host() -> Host {
    Host {
        config: Ok(HashMap::from([("password".to_string(), "hunter2".to_string())])),
        ..Host::default()
    }
}

fn denied(reason: ErrorCode) -> Host {
    let mut host = host();
    host.latch.authorize = Ok(Decision::Denied(reason));
    host
}

fn gate_logs(consumer: &Consumer) -> Vec<Log> {
    consumer.store.data().logs.iter().filter(|log| log.context == "componentized-gate").cloned().collect()
}

#[tokio::test]
async fn deferred_get_is_the_secret() {
    let mut consumer = consumer(host()).await;
    consumer.init().await.unwrap();
    assert_eq!(consumer.get("password").await.unwrap(), Ok(Secret::String("hunter2".to_string())));
    let latch = &consumer.store.data().latch;
    assert_eq!(latch.authorized, vec![get("password")]);
    assert_eq!(latch.observed, vec![(Decision::Deferred, get("password"))]);
    assert_eq!(gate_logs(&consumer), vec![]);
}

#[tokio::test]
async fn deferred_get_of_a_missing_key_is_not_found() {
    let mut consumer = consumer(host()).await;
    consumer.init().await.unwrap();
    assert_eq!(consumer.get("missing").await.unwrap(), Err(ErrorCode::NotFound));
}

#[tokio::test]
async fn deferred_get_before_init_is_not_initialized() {
    let mut consumer = consumer(host()).await;
    assert_eq!(consumer.get("password").await.unwrap(), Err(ErrorCode::NotInitialized));
}

#[tokio::test]
async fn denied_get_fails_with_the_reason() {
    let mut consumer = consumer(denied(ErrorCode::NotFound)).await;
    consumer.init().await.unwrap();
    assert_eq!(consumer.get("password").await.unwrap(), Err(ErrorCode::NotFound));
    let latch = &consumer.store.data().latch;
    assert_eq!(latch.authorized, vec![get("password")]);
    assert_eq!(latch.observed, vec![(Decision::Denied(ErrorCode::NotFound), get("password"))]);
    assert_eq!(
        gate_logs(&consumer),
        vec![Log {
            level: Level::Warn,
            context: "componentized-gate".to_string(),
            message: "Denied REASON=not-found OPERATION=componentized:secrets/store#get KEY=password".to_string(),
        }]
    );
}

#[tokio::test]
async fn denied_get_does_not_call_the_store() {
    // the store is not initialized, it would fail with `not-initialized`
    let mut consumer = consumer(denied(ErrorCode::Other(Some("denied".to_string())))).await;
    assert_eq!(consumer.get("password").await.unwrap(), Err(ErrorCode::Other(Some("denied".to_string()))));
    assert_eq!(
        gate_logs(&consumer)[0].message,
        "Denied REASON=other<denied> OPERATION=componentized:secrets/store#get KEY=password"
    );
}

#[tokio::test]
async fn latch_error_fails_the_get() {
    let mut host = host();
    host.latch.authorize = Err(LatchError::InvalidConfig("latch-key".to_string()));
    let mut consumer = consumer(host).await;
    consumer.init().await.unwrap();
    assert_eq!(
        consumer.get("password").await.unwrap(),
        Err(ErrorCode::Other(Some("latch-error: invalid-config<latch-key>".to_string())))
    );
    // a decision that wasn't made is not observed
    assert_eq!(consumer.store.data().latch.observed, vec![]);
    assert_eq!(
        gate_logs(&consumer),
        vec![Log {
            level: Level::Error,
            context: "componentized-gate".to_string(),
            message: "Latch error CODE=invalid-config<latch-key> OPERATION=componentized:secrets/store#get KEY=password"
                .to_string(),
        }]
    );
}

#[tokio::test]
async fn failed_observation_fails_a_deferred_get() {
    let mut host = host();
    host.latch.observe = Err(LatchError::ObservationFailed("latch-audit".to_string()));
    let mut consumer = consumer(host).await;
    consumer.init().await.unwrap();
    assert_eq!(
        consumer.get("password").await.unwrap(),
        Err(ErrorCode::Other(Some("latch-error: observation-failed<latch-audit>".to_string())))
    );
    assert_eq!(gate_logs(&consumer)[0].level, Level::Error);
}

#[tokio::test]
async fn init_is_not_authorized() {
    let mut consumer = consumer(denied(ErrorCode::NotFound)).await;
    let call = consumer.init().await.unwrap();
    assert_eq!(call.result, Ok(()));
    assert_eq!(consumer.store.data().latch.authorized, vec![]);
    assert_eq!(consumer.store.data().latch.observed, vec![]);
}

#[tokio::test]
async fn first_init_does_not_resolve_immediately() {
    let mut consumer = consumer(host()).await;
    let call = consumer.init().await.unwrap();
    assert!(!call.immediate, "the first init resolved on its first poll");
    let call = consumer.init().await.unwrap();
    assert!(call.immediate, "an init after the store is initialized waited");
}
