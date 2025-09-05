/*
    Instructions with Arguments of One Register & One Immediate.
*/

use std::cmp::{min, max};

use crate::pvm_types::{Gas, RamMemory, Registers, ExitReason, Program, RamAddress, RegSize};
use crate::isa::{skip, extend_sign, _load, _store, djump};

fn get_reg(pc: &RegSize, program: &Program) -> RegSize {
    min(12, program.code[*pc as usize + 1] % 16) as RegSize
}

fn get_x_length(pc: &RegSize, program: &Program) -> RegSize {
    min(4, max(0, skip(pc, &program.bitmask) - 1))
}

fn get_x_imm(pc: &RegSize, program: &Program) -> RegSize {
    let start = *pc as usize + 2;
    let end = start + get_x_length(pc, program) as usize;
    let n = get_x_length(pc, program) as usize;
    extend_sign(&program.code[start..end], n) as RegSize
}

#[inline(always)]
pub fn jump_ind(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let reg_a = get_reg(pc, program);
    let value_imm = get_x_imm(pc, program);
    let value_reg_a = reg[reg_a as usize];
    let a = (value_reg_a.wrapping_add(value_imm) % (1 << 32)) as RegSize;
    djump(&a, pc, program)
}

#[inline(always)]
pub fn load_imm(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers)-> ExitReason {
    let reg_a = get_reg(pc, program);
    let value = get_x_imm(pc, program);
    reg[reg_a as usize] = value;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn load<T>(program: &Program, pc: &mut RegSize, ram: &mut RamMemory, reg: &mut Registers, signed: bool) -> ExitReason {
    let to_reg = get_reg(pc, program) as RegSize;
    let address = get_x_imm(pc, program) as RamAddress;
    _load::<T>(program, pc, ram, reg, address as RamAddress, to_reg, signed)
}

pub fn load_u8(program: &Program, pc: &mut RegSize, _gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers)-> ExitReason {
    load::<u8>(program, pc, ram, reg, false)
}

pub fn load_u16(program: &Program, pc: &mut RegSize, _gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    load::<u16>(program, pc, ram, reg, false)
}

pub fn load_u32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    load::<u32>(program, pc, ram, reg, false)
}

pub fn load_u64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    load::<u64>(program, pc, ram, reg, false)
}

pub fn load_i8(program: &Program, pc: &mut RegSize, _gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    load::<i8>(program, pc, ram, reg, true)
}

pub fn load_i16(program: &Program, pc: &mut RegSize, _gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    load::<i16>(program, pc, ram, reg, true)
}

pub fn load_i32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    load::<i32>(program, pc, ram, reg, true)
}

pub fn store<T>(program: &Program, pc: &mut RegSize, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let reg_a = get_reg(pc, program);
    let address = get_x_imm(pc, program) as RamAddress;
    let value = ((reg[reg_a as usize] as u128) % (1 << (std::mem::size_of::<T>() * 8))) as RegSize;
    _store::<T>( program, pc, ram, address, value)
}

pub fn store_u8(program: &Program, pc: &mut RegSize, _gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    store::<u8>(program, pc, ram, reg)
}

pub fn store_u16(program: &Program, pc: &mut RegSize, _gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    store::<u16>(program, pc, ram, reg)
}

pub fn store_u32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    store::<u32>(program, pc, ram, reg)
}

pub fn store_u64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    store::<u64>(program, pc, ram, reg)
}

