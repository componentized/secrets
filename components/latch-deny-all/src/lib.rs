//! `componentized:secrets` latch that denies every operation with `not-found`, allowing no secrets.

use exports::componentized::secrets::latch::{
    Decision, ErrorCode, Guest as LatchGuest, Operation, StoreErrorCode,
};

struct Component;

impl LatchGuest for Component {
    fn authorize(_operation: Operation) -> Result<Decision, ErrorCode> {
        Ok(Decision::Denied(StoreErrorCode::NotFound))
    }

    fn observe_decision(_final_decision: Decision, _operation: Operation) -> Result<(), ErrorCode> {
        // no side effects
        Ok(())
    }
}

wit_bindgen::generate!({
    path: "../wit",
    world: "latch",
    generate_all,
});

export!(Component);
