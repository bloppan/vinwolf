/*
    Instructions with Arguments of Two Registers & One Immediate
*/

use std::cmp::min;
use crate::pvm_types::{Gas, RamMemory, Registers, ExitReason, Program, RegSize};
use codec::generic_codec::decode;
use crate::isa::skip;

fn get_reg(pc: &RegSize, program: &Program) -> usize {
    min(12, program.code[*pc as usize + 1] % 16) as usize
}

fn get_imm(pc: &RegSize, program: &Program) -> RegSize {
    let start = *pc as usize + 2;
    let end = start + 8;
    decode::<RegSize>(&program.code[start..end], 8)
}

#[inline(always)]
pub fn load_imm_64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let reg_a = get_reg(pc, program);
    let value = get_imm(pc, program);
    reg[reg_a] = value;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}
