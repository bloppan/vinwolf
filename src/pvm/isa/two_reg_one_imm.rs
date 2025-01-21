/*
    Instructions with Arguments of Two Registers & One Immediate
*/

use std::cmp::{min, max};
use crate::constants::NUM_REG;
use crate::types::{Context, Program, ExitReason};
use crate::utils::codec::{DecodeSize, BytesReader};
use crate::pvm::isa::{skip, extend_sign};

fn get_imm(pc: &u64, program: &Program) -> u64 {
   let i = *pc + 2;
   let l_x = min(4, max(0, skip(&i, &program.bitmask).saturating_sub(1))) as usize;
   let number = program.code[i as usize..i as usize + l_x as usize].to_vec();
   return extend_sign(&number, l_x);
}

fn get_reg(pc: &u64, code: &[u8]) -> (u8, u8) {
    let reg_a: u8 = min(12, code[*pc as usize + 1] % 16);
    let reg_b: u8 = min(12, code[*pc as usize + 1] >> 4);
    (reg_a, reg_b)
}

pub fn add_imm_32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b) = get_reg(&pvm_ctx.pc, &program.code);
    let vx: u64 = get_imm(&pvm_ctx.pc, program);
    let result = (pvm_ctx.reg[reg_b as usize].wrapping_add(vx) % (1 << 32)).to_le_bytes();
    pvm_ctx.reg[reg_a as usize] = extend_sign(&result, 4);
    pvm_ctx.pc = skip(&pvm_ctx.pc, &program.bitmask);
    ExitReason::Continue
}

pub fn and_imm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b) = get_reg(&pvm_ctx.pc, &program.code);   
    let vx = get_imm(&pvm_ctx.pc, program);
    pvm_ctx.reg[reg_a as usize] = pvm_ctx.reg[reg_b as usize] & vx;
    pvm_ctx.pc = skip(&pvm_ctx.pc, &program.bitmask);
    ExitReason::Continue
}

pub fn xor_imm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b) = get_reg(&pvm_ctx.pc, &program.code);
    let vx = get_imm(&pvm_ctx.pc, program);
    pvm_ctx.reg[reg_a as usize] = pvm_ctx.reg[reg_b as usize] ^ vx;
    pvm_ctx.pc = skip(&pvm_ctx.pc, &program.bitmask);
    ExitReason::Continue
}

