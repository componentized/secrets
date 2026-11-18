//! The tests of `latch-defer-all`.

use test_harness::{Host, Latch, exported::Decision};

#[tokio::test]
async fn defers_every_key() {
    let mut latch = Latch::new("latch-defer-all", Host::default())
        .await
        .unwrap();
    for key in ["password", "", "db.admin"] {
        assert_eq!(
            latch.authorize_get(key).await.unwrap(),
            Ok(Decision::Deferred)
        );
    }
}

#[tokio::test]
async fn observes_without_error() {
    let mut latch = Latch::new("latch-defer-all", Host::default())
        .await
        .unwrap();
    assert_eq!(
        latch
            .observe_get(&Decision::Deferred, "password")
            .await
            .unwrap(),
        Ok(())
    );
}
