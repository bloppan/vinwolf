/*
    Instructions with Arguments of Two Registers & One Offset.
*/

use std::cmp::{min, max};

use crate::pvm_types::{RamMemory, Gas, Registers, ExitReason, Program, RegSize};
use crate::isa::{skip, signed, _branch};
use codec::BytesReader;
use codec::generic_codec::decode_integer;

fn get_reg(pc: &RegSize, program: &Program) -> (usize, usize) {
    let reg_a = min(12, program.code[*pc as usize + 1] & 15) as usize;
    let reg_b = min(12, program.code[*pc as usize + 1] >> 4) as usize;
    (reg_a, reg_b)
}

fn get_x_length(pc: &RegSize, program: &Program) -> RegSize {
    min(4, max(0, skip(pc, &program.bitmask) - 1))
}

fn get_value(pc: &RegSize, program: &Program) -> i64 {
    let start = (*pc + 2) as usize;
    let end = start + get_x_length(pc, program) as usize;
    let mut reader = BytesReader::new(&program.code[start..end]);
    let value = decode_integer(&mut reader, get_x_length(pc, program) as usize).unwrap() as u64;
    signed(value, get_x_length(pc, program) as usize) + *pc as i64
}

fn branch(
    pc: &mut RegSize,
    reg: &mut Registers,
    program: &Program,
    compare: impl Fn(RegSize, RegSize) -> bool,
) -> ExitReason {
    let (reg_a, reg_b) = get_reg(pc, program);
    let l_value = reg[reg_a];
    let r_value = reg[reg_b];
    let n = get_value(pc, program);
    if !compare(l_value, r_value) {
        *pc += skip(pc, &program.bitmask) + 1;
        return ExitReason::Continue;
    }
    _branch(pc, program, n)
}

#[inline(always)]
pub fn branch_eq(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    branch(pc, reg, program, |a, b| a as RegSize == b as RegSize)
}

#[inline(always)]
pub fn branch_ne(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    branch(pc, reg, program, |a, b| a as RegSize != b as RegSize)
}

#[inline(always)]
pub fn branch_lt_u(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    branch(pc, reg, program, |a, b| (a as RegSize) < (b as RegSize))
}

#[inline(always)]
pub fn branch_lt_s(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    branch(pc, reg, program, |a, b| signed(a, 8) < signed(b, 8))
}

#[inline(always)]
pub fn branch_ge_u(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    branch(pc, reg, program, |a, b| a as RegSize >= b as RegSize)
}

#[inline(always)]
pub fn branch_ge_s(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    branch(pc, reg, program, |a, b| signed(a, 8) >= signed(b, 8))
}

