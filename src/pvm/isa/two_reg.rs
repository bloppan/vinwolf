/*
    Instructions with Arguments of Two Registers.
*/

use std::cmp::{min, max};
use crate::constants::NUM_REG;
use crate::types::{Context, Program, ExitReason};
use crate::utils::codec::{DecodeSize, BytesReader};
use crate::pvm::isa::{skip, extend_sign};

fn get_reg(pc: &u64, code: &[u8]) -> (u8, u8) {
    let reg_a: u8 = min(12, code[*pc as usize + 1] >> 4);
    let reg_d: u8 = min(12, code[*pc as usize + 1] % 16);
    (reg_a, reg_d)
}

pub fn move_reg(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_d) = get_reg(&pvm_ctx.pc, &program.code);
    pvm_ctx.reg[reg_d as usize] = pvm_ctx.reg[reg_a as usize];
    pvm_ctx.pc = skip(&pvm_ctx.pc, &program.bitmask);
    ExitReason::Continue
}


