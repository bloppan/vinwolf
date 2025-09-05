/*
    Instructions with Arguments of One Register, One Immediate and One Offset.
*/

use std::cmp::{min, max};
use crate::pvm_types::{RamMemory, Gas, ExitReason, Program, RegSigned, RegSize, Registers};
use crate::isa::{skip, extend_sign, signed, _branch};
use codec::BytesReader;
use codec::generic_codec::decode_integer;

fn get_reg(pc: &RegSize, program: &Program) -> u8 {
    min(12, program.code[*pc as usize + 1] % 16)
}

fn get_x_length(pc: &RegSize, program: &Program) -> RegSize {
    (min(4, program.code[*pc as usize + 1] >> 4) % 8) as RegSize
}

fn get_y_length(pc: &RegSize, program: &Program) -> RegSize {
    min(4, max(0, skip(pc, &program.bitmask) - 1 - get_x_length(pc, program))) as RegSize
}

fn get_x_value(pc: &RegSize, program: &Program) -> RegSize {
    let start = *pc as usize + 2;
    let end = start + get_x_length(pc, program) as usize;
    extend_sign(&program.code[start..end], get_x_length(pc, program) as usize) as RegSize
}

fn get_y_value(pc: &RegSize, program: &Program) -> i64 {
    let ly = get_y_length(pc, program) as usize;
    let start = *pc as usize + 2 + get_x_length(pc, program) as usize;
    let end = start + ly;
    let mut reader = BytesReader::new(&program.code[start..end]);
    let value = decode_integer(&mut reader, ly).unwrap() as u64;
    signed(value, ly) + *pc as i64
}

fn branch(
    pc: &mut RegSize,
    reg: &mut Registers,
    program: &Program,
    compare: impl Fn(u64, u64) -> bool,
) -> ExitReason {
    let reg_target = get_reg(pc, program);
    let l_value = reg[reg_target as usize];
    let r_value = get_x_value(pc, program);
    let n = get_y_value(pc, program);
    if !compare(l_value, r_value) {
        *pc += skip(pc, &program.bitmask) + 1;
        return ExitReason::Continue;
    }
    _branch(pc, program, n as RegSigned)
}

#[inline(always)]
pub fn load_imm_jump(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let reg_a = get_reg(pc, program);
    let vx = get_x_value(pc, program);
    let vy = get_y_value(pc, program);
    let exit_reason = _branch(pc, program, vy);
    reg[reg_a as usize] = vx as RegSize;
    return exit_reason;
}

#[inline(always)]
pub fn branch_eq_imm(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    branch(pc, reg, program, |a, b| a as RegSize == b as RegSize)
}

#[inline(always)]
pub fn branch_ne_imm(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    branch(pc, reg, program, |a, b| a as RegSize != b as RegSize)
}

#[inline(always)]
pub fn branch_lt_u_imm(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    branch(pc, reg, program, |a, b| (a as RegSize) < (b as RegSize))
}

#[inline(always)]
pub fn branch_le_u_imm(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    branch(pc, reg, program, |a, b| a as RegSize <= b as RegSize)
}

#[inline(always)]
pub fn branch_ge_u_imm(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    branch(pc, reg, program, |a, b| a as RegSize >= b as RegSize) 
}

#[inline(always)]
pub fn branch_gt_u_imm(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    branch(pc, reg, program, |a, b| a as RegSize > b as RegSize)
}

#[inline(always)]
pub fn branch_lt_s_imm(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    branch(pc, reg, program, |a, b| signed(a, 8) < signed(b, 8))
}

#[inline(always)]
pub fn branch_le_s_imm(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    branch(pc, reg, program, |a, b| signed(a, 8) <= signed(b, 8))
}

#[inline(always)]
pub fn branch_ge_s_imm(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    branch(pc, reg, program, |a, b| signed(a, 8) >= signed(b, 8))
}

#[inline(always)]
pub fn branch_gt_s_imm(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    branch(pc, reg, program, |a, b| signed(a, 8) > signed(b, 8))
}

