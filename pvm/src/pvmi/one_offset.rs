/*
    Instructions with Arguments of One Offset.
*/

use std::cmp::min;
use crate::pvm_types::{Gas, RegSize, Registers, RamMemory, ExitReason, Program};
use codec::generic_codec::decode_integer;
use codec::BytesReader;
use crate::pvmi::{skip, _branch, signed};

fn get_lx_length(pc: &u64, bitmask: &[u8]) -> u64 {
    min(4, skip(pc, bitmask)) as u64
}

fn get_lx_imm(pc: &u64, program: &Program) -> i64 {
    let start = *pc as usize + 1;
    let lx = get_lx_length(pc, &program.bitmask) as usize;
    let end = start + lx;
    let mut reader = BytesReader::new(&program.code[start..end]);
    let value = decode_integer(&mut reader, lx).unwrap() as u64;
    signed(value, lx) + *pc as i64
}

#[inline(always)]
pub fn jump(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, _reg: &mut Registers) -> ExitReason {
    let value_x = get_lx_imm(pc, program);
    _branch(pc, program, value_x)
}
