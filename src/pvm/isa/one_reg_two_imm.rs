/*
    Instructions with Arguments of One Register & Two Immediates.
*/

use std::cmp::{min, max};
use crate::constants::RAM_SIZE;
use crate::types::{Context, ExitReason, Program, RamAddress, RegSize};
use crate::pvm::isa::{skip, extend_sign, _store};

fn get_reg(pc: &RegSize, program: &Program) -> RegSize {
    min(12, program.code[*pc as usize + 1] % 16) as u64
}

fn get_x_length(pc: &RegSize, program: &Program) -> RegSize {
    min(4, max(0, (program.code[*pc as usize + 1] >> 4) % 8) as u64)
}

fn get_y_length(pc: &RegSize, program: &Program) -> RegSize {
    min(4, max(0, skip(pc, &program.bitmask) - get_x_length(pc, program) - 1))
}

fn get_x_imm(pc: &RegSize, program: &Program) -> RegSize {
    let start = *pc as usize + 2;
    let end = start + get_x_length(pc, program) as usize;
    let n = get_x_length(pc, program) as usize;
    extend_sign(&program.code[start..end], n)
}

fn get_y_imm(pc: &RegSize, program: &Program) -> RegSize {
    let start = *pc as usize + 2 + get_x_length(pc, program) as usize;
    let end = start + get_y_length(pc, program) as usize;
    let n = get_y_length(pc, program) as usize;
    extend_sign(&program.code[start..end], n)
}

fn get_address(pvm_ctx: &Context, program: &Program) -> RamAddress {
    let reg_a = get_reg(&pvm_ctx.pc, program);
    let addr_reg_a = pvm_ctx.reg[reg_a as usize];
    let vx = get_x_imm(&pvm_ctx.pc, program);
    ((addr_reg_a + vx) % RAM_SIZE) as RamAddress
}

fn get_value<T>(pc: &RegSize, program: &Program) -> RegSize {
    ((get_y_imm(pc, program) as u128) % (1 << (std::mem::size_of::<T>() * 8))) as RegSize
}

fn store_imm_ind<T>(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let address = get_address(pvm_ctx, program);
    let value = get_value::<T>(&pvm_ctx.pc, program);
    _store::<T>(pvm_ctx, program, address, value)
}

pub fn store_imm_ind_u8(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    store_imm_ind::<u8>(pvm_ctx, program)
}

pub fn store_imm_ind_u16(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    store_imm_ind::<u16>(pvm_ctx, program)
}

pub fn store_imm_ind_u32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    store_imm_ind::<u32>(pvm_ctx, program)
}

pub fn store_imm_ind_u64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    store_imm_ind::<u64>(pvm_ctx, program)
}
