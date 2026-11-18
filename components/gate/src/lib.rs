//! `componentized:secrets` gate, access control for the secrets of a store.
//!
//! The latch is asked to authorize each `get`, then told the final decision with
//! `observe-decision`. `init` is not authorized, it gets no secret.

use std::fmt;

use componentized::secrets::latch::{
    self, Decision, ErrorCode as LatchErrorCode, GetArgs, Operation, StoreOperation,
};
use componentized::secrets::store;
use componentized::secrets::types::{ErrorCode, Secret};
use exports::componentized::secrets::store::Guest as StoreGuest;
use wasi::logging::logging::{Level, log};

const CONTEXT: &str = "componentized-gate";

macro_rules! warn {
    ($dst:expr, $($arg:tt)*) => {
        log(Level::Warn, CONTEXT, &format!($dst, $($arg)*));
    };
    ($dst:expr) => {
        log(Level::Warn, CONTEXT, &format!($dst));
    };
}

macro_rules! error {
    ($dst:expr, $($arg:tt)*) => {
        log(Level::Error, CONTEXT, &format!($dst, $($arg)*));
    };
    ($dst:expr) => {
        log(Level::Error, CONTEXT, &format!($dst));
    };
}

/// Authorize the operation with the latch, then report the decision back to the latch. Latches act
/// on the final decision in `observe-decision`, never while authorizing. Failing to observe the
/// decision fails the operation, the same as a latch error.
fn authorize(operation: &Operation) -> Result<Decision, LatchErrorCode> {
    let decision = latch::authorize(operation)?;
    latch::observe_decision(&decision, operation)?;
    Ok(decision)
}

struct Component;

impl StoreGuest for Component {
    async fn init() -> Result<(), ErrorCode> {
        store::init().await
    }

    fn get(key: String) -> Result<Secret, ErrorCode> {
        let call_summary = || format!("OPERATION=componentized:secrets/store#get KEY={key}");

        match authorize(&Operation::Store(StoreOperation::Get(GetArgs {
            key: key.clone(),
        }))) {
            Ok(Decision::Denied(reason)) => {
                warn!(
                    "Denied REASON={} {}",
                    DisplayReason(&reason),
                    call_summary()
                );
                Err(reason)?
            }
            Ok(Decision::Deferred) => store::get(&key),
            Err(code) => {
                error!(
                    "Latch error CODE={code} {summary}",
                    code = DisplayLatchError(&code),
                    summary = call_summary()
                );
                Err(code)?
            }
        }
    }
}

/// Displays a denial reason, the name of the error code, e.g. `not-found`.
struct DisplayReason<'a>(&'a store::ErrorCode);
impl fmt::Display for DisplayReason<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            store::ErrorCode::NotInitialized => f.write_str("not-initialized"),
            store::ErrorCode::NotFound => f.write_str("not-found"),
            store::ErrorCode::Other(Some(message)) => write!(f, "other<{message}>"),
            store::ErrorCode::Other(None) => f.write_str("other"),
        }
    }
}

/// Displays a latch error, e.g. `invalid-config<latch-key>`.
struct DisplayLatchError<'a>(&'a LatchErrorCode);
impl fmt::Display for DisplayLatchError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            LatchErrorCode::InvalidConfig(latch) => write!(f, "invalid-config<{latch}>"),
            LatchErrorCode::ObservationFailed(latch) => write!(f, "observation-failed<{latch}>"),
            LatchErrorCode::Other(Some(message)) => f.write_str(message),
            LatchErrorCode::Other(None) => f.write_str("other"),
        }
    }
}

impl From<LatchErrorCode> for ErrorCode {
    fn from(value: LatchErrorCode) -> Self {
        match value {
            LatchErrorCode::InvalidConfig(latch) => {
                Self::Other(Some(format!("latch-error: invalid-config<{latch}>")))
            }
            LatchErrorCode::ObservationFailed(latch) => {
                Self::Other(Some(format!("latch-error: observation-failed<{latch}>")))
            }
            LatchErrorCode::Other(Some(message)) => {
                Self::Other(Some(format!("latch-error: {message}")))
            }
            LatchErrorCode::Other(None) => Self::Other(Some("latch-error".to_string())),
        }
    }
}

wit_bindgen::generate!({
    path: "../wit",
    world: "gate",
    merge_structurally_equal_types: true,
    generate_all,
});

export!(Component);
