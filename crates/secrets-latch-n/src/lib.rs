//! Bindings and helpers for the `latch-n` components, which aggregate several latches into one.
//!
//! The bindings are generated once here for the `latch-n` world and shared by every `latch-n`
//! component. Each uses the first of the imported latches, the imports it doesn't use are dropped
//! from the built component.

pub use crate::bindings::exports::componentized::secrets::latch::{
    Decision, ErrorCode, Guest as Latch, Operation,
};
pub use crate::bindings::{latch0, latch1, latch2, latch3, latch4};

/// Ask each latch in turn, the first denial is the decision. Latches after a denial are not asked.
pub fn authorize(
    operation: Operation,
    authorizers: &[fn(&Operation) -> Result<Decision, ErrorCode>],
) -> Result<Decision, ErrorCode> {
    for authorize in authorizers {
        if let Decision::Denied(reason) = authorize(&operation)? {
            return Ok(Decision::Denied(reason));
        }
    }
    Ok(Decision::Deferred)
}

/// Pass the final decision to every latch, including latches that were not asked to authorize
/// the operation because an earlier latch denied it.
///
/// Every observer is called even if an earlier one fails, the result is the first error.
pub fn observe_decision(
    final_decision: Decision,
    operation: Operation,
    observers: &[fn(&Decision, &Operation) -> Result<(), ErrorCode>],
) -> Result<(), ErrorCode> {
    let mut result = Ok(());
    for observe in observers {
        if let Err(err) = observe(&final_decision, &operation) {
            if result.is_ok() {
                result = Err(err);
            }
        }
    }
    result
}

pub mod bindings {
    wit_bindgen::generate!({
        path: "../../components/wit",
        world: "latch-n",
        pub_export_macro: true,
        merge_structurally_equal_types: true,
        generate_all
    });
}

#[macro_export]
macro_rules! export {
    ($($t:tt)*) => {
        $crate::bindings::export!($($t)*);
    };
}
