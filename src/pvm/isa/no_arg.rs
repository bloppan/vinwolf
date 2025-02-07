/*
    Instructions with Arguments of One Offset.
*/

use crate::types::ExitReason;

pub fn trap() -> ExitReason {
    ExitReason::panic
}

pub fn fallthrough() -> ExitReason {
    ExitReason::Continue
}