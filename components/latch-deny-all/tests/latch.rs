//! The tests of `latch-deny-all`.

use test_harness::{ErrorCode, Host, Latch, exported::Decision};

#[tokio::test]
async fn denies_every_key_with_not_found() {
    let mut latch = Latch::new("latch-deny-all", Host::default()).await.unwrap();
    for key in ["password", "", "db.admin"] {
        assert_eq!(latch.authorize_get(key).await.unwrap(), Ok(Decision::Denied(ErrorCode::NotFound)));
    }
}

#[tokio::test]
async fn observes_without_error() {
    let mut latch = Latch::new("latch-deny-all", Host::default()).await.unwrap();
    let decision = Decision::Denied(ErrorCode::NotFound);
    assert_eq!(latch.observe_get(&decision, "password").await.unwrap(), Ok(()));
}
