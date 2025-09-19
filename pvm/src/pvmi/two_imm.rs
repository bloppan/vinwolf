/*
    Instructions with Arguments of Two Immediates.
*/

use std::cmp::{min, max};
use crate::pvm_types::{Gas, RamMemory, Registers, ExitReason, Program, RamAddress, RegSize};
use crate::pvmi::{skip, extend_sign};

use super::_store;

fn get_x_length(pc: &RegSize, program: &Program) -> RegSize {
    min(4, program.code[*pc as usize + 1] & 7) as RegSize
}

fn get_y_length(pc: &RegSize, program: &Program) -> RegSize {
    min(4, max(0, skip(pc, &program.bitmask) - get_x_length(pc, program) as u64 - 1)) as RegSize
}

fn get_x_imm(pc: &RegSize, program: &Program) -> RegSize {
    let start = *pc as usize + 2;
    let end = start + get_x_length(pc, program) as usize;
    extend_sign(&program.code[start..end], get_x_length(pc, program) as usize) as RegSize
}

fn get_y_imm(pc: &RegSize, program: &Program) -> RegSize {
    let start = *pc as usize + 2 + get_x_length(pc, program) as usize;
    let end = start + get_y_length(pc, program) as usize;
    extend_sign(&program.code[start..end], get_y_length(pc, program) as usize) as RegSize
}

pub fn store_imm_u8(program: &Program, pc: &mut RegSize, _gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    store_imm::<u8>(program, pc, ram, reg)
}

pub fn store_imm_u16(program: &Program, pc: &mut RegSize, _gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    store_imm::<u16>(program, pc, ram, reg)
}

pub fn store_imm_u32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    store_imm::<u32>(program, pc, ram, reg)
}

pub fn store_imm_u64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    store_imm::<u64>(program, pc, ram, reg)
}

fn store_imm<T>(program: &Program, pc: &mut RegSize, ram: &mut RamMemory, _reg: &mut Registers) -> ExitReason {
    let address = get_x_imm(pc, program) as RamAddress;
    let value = ((get_y_imm(pc, program) as u128) & (1 << (std::mem::size_of::<T>() * 8)) - 1) as RegSize;
    _store::<T>(program, pc, ram, address, value)
}