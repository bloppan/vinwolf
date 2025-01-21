/*
    Instructions with Arguments of Two Registers & One Immediate
*/

use std::cmp::{min, max};
use crate::constants::NUM_REG;
/*use crate::types::{Context, ProgramSequence, ExitReason};
use crate::utils::codec::{DecodeSize, BytesReader};
use crate::utils::codec::generic::{decode_unsigned};
use crate::pvm::isa::{skip, extend_sign};

fn get_reg(pvm_ctx: &mut Context, program: &ProgramSequence) -> u8 {
    min(12, program.data[pvm_ctx.pc as usize + 1] % 16)
}

fn get_imm(pc: &u64, program: &ProgramSequence) -> u64 {
    let start = *pc as usize + 2;
    let end = start + 8;
    let mut array = [0u8; std::mem::size_of::<usize>()];
    array.copy_from_slice(&program.data[start..end]);
    u64::from_le_bytes(array)
}

pub fn load_imm_64(pvm_ctx: &mut Context, program: &ProgramSequence) -> ExitReason {
    let reg_a = get_reg(pvm_ctx, program);
    let imm = get_imm(&pvm_ctx.pc, program);
    pvm_ctx.reg[reg_a as usize] = imm;
    pvm_ctx.pc = skip(&pvm_ctx.pc, &program.bitmask);
    ExitReason::Continue
}*/