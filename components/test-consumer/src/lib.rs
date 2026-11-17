//! A consumer of `componentized:secrets`, for the tests of each component. Records whether `init`
//! resolves on its first poll.

use std::future::{Future, poll_fn};
use std::pin::pin;
use std::task::Poll;

use componentized::secrets::store::{self, ErrorCode, Secret};
use exports::componentized::secrets_components::probe::{Guest as ProbeGuest, InitCall};

struct Component;

impl ProbeGuest for Component {
    async fn init() -> InitCall {
        let mut call = pin!(store::init());
        let first = poll_fn(|cx| Poll::Ready(call.as_mut().poll(cx))).await;
        match first {
            Poll::Ready(result) => InitCall { immediate: true, result },
            Poll::Pending => InitCall { immediate: false, result: call.await },
        }
    }

    fn get(key: String) -> Result<Secret, ErrorCode> {
        store::get(&key)
    }
}

wit_bindgen::generate!({
    path: "../wit",
    world: "test-consumer",
    generate_all,
});

export!(Component);
