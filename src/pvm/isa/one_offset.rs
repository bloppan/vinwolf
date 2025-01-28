/*
    Instructions with Arguments of One Offset.
*/

use std::cmp::{min, max};
use crate::constants::{NUM_REG, PAGE_SIZE, RAM_SIZE};
use crate::types::{Context, ExitReason, MemoryChunk, Program};
use crate::utils::codec::{EncodeSize, DecodeSize, BytesReader};
use crate::pvm::isa::{skip, extend_sign};

fn get_lx_length(pc: &u64, bitmask: &[bool]) -> u64 {
    min(4, skip(pc, bitmask)) as u64
}

fn get_lx_imm(pc: &u64, program: &Program) -> u64 {
    let start = *pc as usize + 1;
    let end = start + get_lx_length(pc, &program.bitmask) as usize;
    let n = get_lx_length(pc, &program.bitmask) as usize;
    extend_sign(&program.code[start..end], n) as u64
}