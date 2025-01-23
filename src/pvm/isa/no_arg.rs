/*
    Instructions with Arguments of One Offset.
*/

use std::cmp::{min, max};
use crate::constants::{NUM_REG, PAGE_SIZE, RAM_SIZE};
use crate::types::{Context, ExitReason, MemoryChunk, Program};
use crate::utils::codec::{EncodeSize, DecodeSize, BytesReader};
use crate::pvm::isa::{skip, extend_sign};

pub fn trap() -> ExitReason {
    ExitReason::trap // TODO esto es un panic
}

pub fn fallthrough() -> ExitReason {
    ExitReason::Continue
}