//! `componentized:secrets`, backed by `wasi:config`, for hosts without secrets, e.g. wasmtime. Each secret
//! is the config value of the same key, a missing key is not found.

use exports::componentized::secrets::store::{
    Guest as StoreGuest, Secret, ErrorCode,
};
use wasi::config::store as config;

struct Component;

impl StoreGuest for Component {
    async fn get(key: String) -> Result<Secret, ErrorCode> {
        match config::get(&key) {
            Ok(Some(value)) => Ok(Secret::String(value)),
            Ok(None) => Err(ErrorCode::NotFound),
            Err(config::Error::Upstream(message)) => Err(ErrorCode::Other(Some(format!("upstream: {message}")))),
            Err(config::Error::Io(message)) => Err(ErrorCode::Other(Some(format!("io: {message}")))),
        }
    }
}

wit_bindgen::generate!({
    path: "../wit",
    world: "from-config",
    generate_all,
});

export!(Component);
