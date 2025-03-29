/*
    Instructions with Arguments of Two Registers.
*/

use std::cmp::min;
use crate::types::{RegSize, Context, Program, ExitReason};
use crate::pvm::isa::skip;
use crate::pvm::isa::{signed, unsigned};

fn get_reg(pc: &u64, code: &[u8]) -> (u8, u8) {
    let reg_a: u8 = min(12, code[*pc as usize + 1] >> 4);
    let reg_d: u8 = min(12, code[*pc as usize + 1] % 16);
    (reg_a, reg_d)
}

pub fn move_reg(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_d) = get_reg(&pvm_ctx.pc, &program.code);
    pvm_ctx.reg[reg_d as usize] = pvm_ctx.reg[reg_a as usize];
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn count_set_bits_64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_d) = get_reg(&pvm_ctx.pc, &program.code);
    pvm_ctx.reg[reg_d as usize] = pvm_ctx.reg[reg_a as usize].count_ones() as RegSize;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn count_set_bits_32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_d) = get_reg(&pvm_ctx.pc, &program.code);
    pvm_ctx.reg[reg_d as usize] = (pvm_ctx.reg[reg_a as usize] as u32).count_ones() as RegSize;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn leading_zero_bits_64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_d) = get_reg(&pvm_ctx.pc, &program.code);
    pvm_ctx.reg[reg_d as usize] = pvm_ctx.reg[reg_a as usize].leading_zeros() as RegSize;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn leading_zero_bits_32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_d) = get_reg(&pvm_ctx.pc, &program.code);
    pvm_ctx.reg[reg_d as usize] = (pvm_ctx.reg[reg_a as usize] as u32).leading_zeros() as RegSize;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn trailing_zero_bits_64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_d) = get_reg(&pvm_ctx.pc, &program.code);
    pvm_ctx.reg[reg_d as usize] = pvm_ctx.reg[reg_a as usize].trailing_zeros() as RegSize;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn trailing_zero_bits_32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_d) = get_reg(&pvm_ctx.pc, &program.code);
    pvm_ctx.reg[reg_d as usize] = (pvm_ctx.reg[reg_a as usize] as u32).trailing_zeros() as RegSize;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn sign_extend_8(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_d) = get_reg(&pvm_ctx.pc, &program.code);
    pvm_ctx.reg[reg_d as usize] = unsigned(signed((pvm_ctx.reg[reg_a as usize] as u8) as u64, 1), 8) as RegSize;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn sign_extend_16(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_d) = get_reg(&pvm_ctx.pc, &program.code);
    pvm_ctx.reg[reg_d as usize] = unsigned(signed((pvm_ctx.reg[reg_a as usize] as u16) as u64, 2), 8) as RegSize;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn zero_extend_16(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_d) = get_reg(&pvm_ctx.pc, &program.code);
    pvm_ctx.reg[reg_d as usize] = (pvm_ctx.reg[reg_a as usize] as u16) as RegSize;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn reverse_bytes(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_d) = get_reg(&pvm_ctx.pc, &program.code);
    pvm_ctx.reg[reg_d as usize] = pvm_ctx.reg[reg_a as usize].swap_bytes();
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}