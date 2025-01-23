/*
    Instructions with Arguments of Two Registers & One Immediate
*/

use std::cmp::{min, max};
use crate::constants::NUM_REG;
use crate::types::{Context, ExitReason, Program, RegSize};
use crate::utils::codec::{DecodeSize, BytesReader};
use crate::utils::codec::generic::{decode_unsigned, decode};
use crate::pvm::isa::{skip, extend_sign};

fn get_reg(pvm_ctx: &mut Context, program: &Program) -> u8 {
    min(12, program.code[pvm_ctx.pc as usize + 1] % 16)
}

fn get_imm(pc: &RegSize, program: &Program) -> RegSize {
    let start = *pc as usize + 2;
    let end = start + 8;
    decode::<RegSize>(&program.code[start..end])
}

pub fn load_imm_64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let reg_a = get_reg(pvm_ctx, program);
    let value = get_imm(&pvm_ctx.pc, program);
    pvm_ctx.reg[reg_a as usize] = value;
    ExitReason::Continue
}