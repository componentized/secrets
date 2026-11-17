//! `componentized:secrets`, always returning not found.

use std::sync::atomic::{AtomicBool, Ordering};

use exports::componentized::secrets::store::{ErrorCode, Guest as StoreGuest, Secret};

static INITIALIZED: AtomicBool = AtomicBool::new(false);

struct Component;

impl StoreGuest for Component {
    async fn init() -> Result<(), ErrorCode> {
        if !INITIALIZED.load(Ordering::Relaxed) {
            // never resolve the first init immediately, so callers handle init as async work
            wit_bindgen::yield_async().await;
            INITIALIZED.store(true, Ordering::Relaxed);
        }
        Ok(())
    }

    fn get(_key: String) -> Result<Secret, ErrorCode> {
        if !INITIALIZED.load(Ordering::Relaxed) {
            return Err(ErrorCode::NotInitialized);
        }
        Err(ErrorCode::NotFound)
    }
}

wit_bindgen::generate!({
    path: "../wit",
    world: "empty",
    generate_all,
});

export!(Component);
