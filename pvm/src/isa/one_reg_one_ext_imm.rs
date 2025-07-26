/*
    Instructions with Arguments of Two Registers & One Immediate
*/

use std::cmp::min;
use crate::pvm_types::{Context, ExitReason, Program, RegSize};
use codec::generic_codec::decode;
use crate::isa::skip;

fn get_reg(pvm_ctx: &mut Context, program: &Program) -> usize {
    min(12, program.code[pvm_ctx.pc as usize + 1] % 16) as usize
}

fn get_imm(pc: &RegSize, program: &Program) -> RegSize {
    let start = *pc as usize + 2;
    let end = start + 8;
    decode::<RegSize>(&program.code[start..end], 8)
}

pub fn load_imm_64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let reg_a = get_reg(pvm_ctx, program);
    let value = get_imm(&pvm_ctx.pc, program);
    pvm_ctx.reg[reg_a] = value;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}
