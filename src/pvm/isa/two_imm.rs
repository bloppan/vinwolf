/*
    Instructions with Arguments of Two Immediates.
*/

use std::cmp::{min, max};
use crate::types::{Context, ExitReason, Program, RamAddress, RegSize};
use crate::pvm::isa::{skip, extend_sign};

use super::_store;

fn get_x_length(pc: &RegSize, program: &Program) -> RegSize {
    min(4, program.code[*pc as usize + 1] % 8) as RegSize
}

fn get_y_length(pc: &RegSize, program: &Program) -> RegSize {
    min(4, max(0, skip(pc, &program.bitmask) - get_x_length(pc, program) as u64 - 1)) as RegSize
}

fn get_x_imm(pc: &RegSize, program: &Program) -> RegSize {
    let start = *pc as usize + 2;
    let end = start + get_x_length(pc, program) as usize;
    extend_sign(&program.code[start..end], get_x_length(pc, program) as usize) as RegSize
}

fn get_y_imm(pc: &RegSize, program: &Program) -> RegSize {
    let start = *pc as usize + 2 + get_x_length(pc, program) as usize;
    let end = start + get_y_length(pc, program) as usize;
    extend_sign(&program.code[start..end], get_y_length(pc, program) as usize) as RegSize
}

pub fn store_imm_u8(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    store_imm::<u8>(pvm_ctx, program)
}

pub fn store_imm_u16(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    store_imm::<u16>(pvm_ctx, program)
}

pub fn store_imm_u32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    store_imm::<u32>(pvm_ctx, program)
}

pub fn store_imm_u64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    store_imm::<u64>(pvm_ctx, program)
}

fn store_imm<T>(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let address = get_x_imm(&pvm_ctx.pc, program) as RamAddress;
    let value = ((get_y_imm(&pvm_ctx.pc, program) as u128) % (1 << (std::mem::size_of::<T>() * 8))) as RegSize;
    _store::<T>(pvm_ctx, program, address, value)
}