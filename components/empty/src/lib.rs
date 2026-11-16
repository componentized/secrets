//! `componentized:secrets`, always returning not found.

use exports::componentized::secrets::store::{
    Guest as StoreGuest, Secret, ErrorCode,
};

struct Component;

impl StoreGuest for Component {
    async fn get(_key: String) -> Result<Secret, ErrorCode> {
        Err(ErrorCode::NotFound)
    }
}

wit_bindgen::generate!({
    path: "../wit",
    world: "empty",
    generate_all,
});

export!(Component);
