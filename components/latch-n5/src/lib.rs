//! `componentized:secrets` latch that aggregates 5 latches, see README.md.

use secrets_latch_n::{Decision, ErrorCode, Latch, Operation};
use secrets_latch_n::{latch0, latch1, latch2, latch3, latch4};

struct LatchN5;

impl Latch for LatchN5 {
    fn authorize(operation: Operation) -> Result<Decision, ErrorCode> {
        secrets_latch_n::authorize(
            operation,
            &[
                latch0::authorize,
                latch1::authorize,
                latch2::authorize,
                latch3::authorize,
                latch4::authorize,
            ],
        )
    }

    fn observe_decision(final_decision: Decision, operation: Operation) -> Result<(), ErrorCode> {
        secrets_latch_n::observe_decision(
            final_decision,
            operation,
            &[
                latch0::observe_decision,
                latch1::observe_decision,
                latch2::observe_decision,
                latch3::observe_decision,
                latch4::observe_decision,
            ],
        )
    }
}

secrets_latch_n::export!(LatchN5 with_types_in secrets_latch_n::bindings);
