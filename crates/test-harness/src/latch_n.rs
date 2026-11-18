//! The tests of a `latch-n` component, e.g. `latch-n2`, aggregating the host's fake latches
//! `latch0` to `latch4`. Each `latch-n` runs them with [`latch_n_tests!`](crate::latch_n_tests).

use crate::{Decision, ErrorCode, Host, Latch, LatchError, exported, get};

/// The calls to the first `n` latches, e.g. `latch0.authorize`, `latch1.authorize`.
fn calls(call: &str, n: usize) -> Vec<String> {
    (0..n).map(|i| format!("latch{i}.{call}")).collect()
}

fn denied(i: usize) -> ErrorCode {
    ErrorCode::Other(Some(format!("denied by latch{i}")))
}

pub async fn deferred_when_every_latch_defers(component: &str, n: usize) {
    let mut latch = Latch::new(component, Host::default()).await.unwrap();
    assert_eq!(
        latch.authorize_get("password").await.unwrap(),
        Ok(exported::Decision::Deferred),
        "{component}"
    );
    let host = latch.store.data();
    assert_eq!(host.latch_calls, calls("authorize", n), "{component}");
    for fake in &host.latches[..n] {
        assert_eq!(fake.authorized, vec![get("password")], "{component}");
    }
}

pub async fn first_denial_is_the_decision(component: &str, n: usize) {
    for denying in 0..n {
        let mut host = Host::default();
        // every latch from the denying one on denies, with its own reason
        for i in denying..n {
            host.latches[i].authorize = Ok(Decision::Denied(denied(i)));
        }
        let mut latch = Latch::new(component, host).await.unwrap();
        assert_eq!(
            latch.authorize_get("password").await.unwrap(),
            Ok(exported::Decision::Denied(denied(denying))),
            "{component} latch{denying}"
        );
        // the latches after the denial are not asked
        assert_eq!(
            latch.store.data().latch_calls,
            calls("authorize", denying + 1),
            "{component} latch{denying}"
        );
    }
}

pub async fn authorize_error_is_returned(component: &str, n: usize) {
    for failing in 0..n {
        let mut host = Host::default();
        host.latches[failing].authorize = Err(LatchError::InvalidConfig(format!("latch{failing}")));
        let mut latch = Latch::new(component, host).await.unwrap();
        assert_eq!(
            latch.authorize_get("password").await.unwrap(),
            Err(exported::LatchError::InvalidConfig(format!(
                "latch{failing}"
            ))),
            "{component} latch{failing}"
        );
        assert_eq!(
            latch.store.data().latch_calls,
            calls("authorize", failing + 1),
            "{component} latch{failing}"
        );
    }
}

pub async fn every_latch_observes_the_final_decision(component: &str, n: usize) {
    for decision in [
        exported::Decision::Deferred,
        exported::Decision::Denied(ErrorCode::NotFound),
    ] {
        let mut latch = Latch::new(component, Host::default()).await.unwrap();
        assert_eq!(
            latch.observe_get(&decision, "password").await.unwrap(),
            Ok(()),
            "{component}"
        );
        let host = latch.store.data();
        assert_eq!(
            host.latch_calls,
            calls("observe-decision", n),
            "{component}"
        );
        let expected = match &decision {
            exported::Decision::Deferred => Decision::Deferred,
            exported::Decision::Denied(reason) => Decision::Denied(reason.clone()),
        };
        for fake in &host.latches[..n] {
            assert_eq!(
                fake.observed,
                vec![(expected.clone(), get("password"))],
                "{component}"
            );
        }
    }
}

pub async fn first_observation_failure_is_returned_after_every_latch_observes(
    component: &str,
    n: usize,
) {
    let mut host = Host::default();
    for i in 0..n {
        host.latches[i].observe = Err(LatchError::ObservationFailed(format!("latch{i}")));
    }
    let mut latch = Latch::new(component, host).await.unwrap();
    assert_eq!(
        latch
            .observe_get(&exported::Decision::Deferred, "password")
            .await
            .unwrap(),
        Err(exported::LatchError::ObservationFailed(
            "latch0".to_string()
        )),
        "{component}"
    );
    assert_eq!(
        latch.store.data().latch_calls,
        calls("observe-decision", n),
        "{component}"
    );
}

/// The tests of a `latch-n` component that aggregates `n` latches, e.g.
/// `test_harness::latch_n_tests!("latch-n2", 2);` in its tests.
#[macro_export]
macro_rules! latch_n_tests {
    ($component:literal, $n:literal) => {
        #[tokio::test]
        async fn deferred_when_every_latch_defers() {
            $crate::latch_n::deferred_when_every_latch_defers($component, $n).await;
        }
        #[tokio::test]
        async fn first_denial_is_the_decision() {
            $crate::latch_n::first_denial_is_the_decision($component, $n).await;
        }
        #[tokio::test]
        async fn authorize_error_is_returned() {
            $crate::latch_n::authorize_error_is_returned($component, $n).await;
        }
        #[tokio::test]
        async fn every_latch_observes_the_final_decision() {
            $crate::latch_n::every_latch_observes_the_final_decision($component, $n).await;
        }
        #[tokio::test]
        async fn first_observation_failure_is_returned_after_every_latch_observes() {
            $crate::latch_n::first_observation_failure_is_returned_after_every_latch_observes(
                $component, $n,
            )
            .await;
        }
    };
}
