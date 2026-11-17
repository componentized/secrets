//! `componentized:secrets` latch that defers every operation, allowing all secrets.

use exports::componentized::secrets::latch::{Decision, ErrorCode, Guest as LatchGuest, Operation};

struct Component;

impl LatchGuest for Component {
    fn authorize(_operation: Operation) -> Result<Decision, ErrorCode> {
        Ok(Decision::Deferred)
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
