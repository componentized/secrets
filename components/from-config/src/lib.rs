//! `componentized:secrets`, backed by `wasi:config`, for hosts without secrets, e.g. wasmtime. Each secret
//! is the config value of the same key, a missing key is not found. The config values are read once, by the
//! first `init`.

use std::collections::HashMap;
use std::sync::OnceLock;

use exports::componentized::secrets::store::{ErrorCode, Guest as StoreGuest, Secret};
use wasi::config::store as config;

/// The config values, or the error reading them, read once by the first `init`.
static SECRETS: OnceLock<Result<HashMap<String, String>, ErrorCode>> = OnceLock::new();

struct Component;

impl StoreGuest for Component {
    async fn init() -> Result<(), ErrorCode> {
        if SECRETS.get().is_none() {
            // never resolve the first init immediately, so callers handle init as async work
            wit_bindgen::yield_async().await;
        }
        SECRETS
            .get_or_init(|| {
                let values = config::get_all().map_err(|err| match err {
                    config::Error::Upstream(message) => {
                        ErrorCode::Other(Some(format!("upstream: {message}")))
                    }
                    config::Error::Io(message) => ErrorCode::Other(Some(format!("io: {message}"))),
                })?;
                Ok(values.into_iter().collect())
            })
            .as_ref()
            .map(|_| ())
            .map_err(Clone::clone)
    }

    fn get(key: String) -> Result<Secret, ErrorCode> {
        let Some(Ok(secrets)) = SECRETS.get() else {
            return Err(ErrorCode::NotInitialized);
        };
        secrets
            .get(&key)
            .map(|value| Secret::String(value.clone()))
            .ok_or(ErrorCode::NotFound)
    }
}

wit_bindgen::generate!({
    path: "../wit",
    world: "from-config",
    generate_all,
});

export!(Component);
