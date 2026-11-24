//! Adapts a `componentized:secrets/store` to `wasmcloud:secrets`, for components that use the secrets
//! of a wasmCloud host.
//!
//! `wasmcloud:secrets` has no `init`, the store is initialized by the first `get`, again by the next
//! `get` when it fails. A secret holds its value, the store's secrets are immutable once initialized,
//! so revealing a secret can't fail.

use std::sync::atomic::{AtomicBool, Ordering};

use componentized::secrets::store;
use componentized::secrets::types::{ErrorCode, Secret as StoreSecret};
use exports::wasmcloud::secrets::reveal::Guest as RevealGuest;
use exports::wasmcloud::secrets::store::{
    Guest as StoreGuest, GuestSecret, Secret, SecretBorrow, SecretValue, SecretsError,
};

/// Whether the store is initialized, it isn't initialized again.
static INITIALIZED: AtomicBool = AtomicBool::new(false);

struct Component;

/// A secret of the store, holding its value.
pub struct AdaptedSecret {
    value: SecretValue,
}

impl GuestSecret for AdaptedSecret {}

impl StoreGuest for Component {
    type Secret = AdaptedSecret;

    async fn get(key: String) -> Result<Secret, SecretsError> {
        if !INITIALIZED.load(Ordering::Acquire) {
            store::init().await?;
            INITIALIZED.store(true, Ordering::Release);
        }
        let value = match store::get(&key)? {
            StoreSecret::String(value) => SecretValue::String(value),
            StoreSecret::Bytes(value) => SecretValue::Bytes(value),
        };
        Ok(Secret::new(AdaptedSecret { value }))
    }
}

impl RevealGuest for Component {
    async fn reveal(s: SecretBorrow<'_>) -> SecretValue {
        s.get::<AdaptedSecret>().value.clone()
    }
}

/// The `wasmcloud:secrets` error for an error of the store. Any error but `not-found` is an upstream
/// error, with the store's message.
impl From<ErrorCode> for SecretsError {
    fn from(code: ErrorCode) -> Self {
        match code {
            ErrorCode::NotFound => SecretsError::NotFound,
            ErrorCode::NotInitialized => SecretsError::Upstream("not-initialized".to_string()),
            ErrorCode::Other(Some(message)) => SecretsError::Upstream(message),
            ErrorCode::Other(None) => SecretsError::Upstream("other".to_string()),
        }
    }
}

wit_bindgen::generate!({
    path: "../wit",
    world: "to-wasmcloud",
    generate_all,
});

export!(Component);
