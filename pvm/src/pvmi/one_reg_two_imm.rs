/*
    Instructions with Arguments of One Register & Two Immediates.
*/

use std::cmp::{min, max};
use crate::pvm_types::{RamMemory, Gas, Registers, ExitReason, Program, RamAddress, RegSize};
use crate::pvmi::{skip, extend_sign, _store};

fn get_reg(pc: &RegSize, program: &Program) -> RegSize {
    min(12, program.code[*pc as usize + 1] & 15) as u64
}

fn get_x_length(pc: &RegSize, program: &Program) -> RegSize {
    min(4, max(0, (program.code[*pc as usize + 1] >> 4) & 7) as u64)
}

fn get_y_length(pc: &RegSize, program: &Program) -> RegSize {
    min(4, max(0, skip(pc, &program.bitmask) - get_x_length(pc, program) - 1))
}

fn get_x_imm(pc: &RegSize, program: &Program) -> RegSize {
    let start = *pc as usize + 2;
    let end = start + get_x_length(pc, program) as usize;
    let n = get_x_length(pc, program) as usize;
    extend_sign(&program.code[start..end], n)
}

fn get_y_imm(pc: &RegSize, program: &Program) -> RegSize {
    let start = *pc as usize + 2 + get_x_length(pc, program) as usize;
    let end = start + get_y_length(pc, program) as usize;
    let n = get_y_length(pc, program) as usize;
    extend_sign(&program.code[start..end], n)
}

fn get_address(pc: &RegSize, reg: &Registers, program: &Program) -> RamAddress {
    let reg_a = get_reg(pc, program);
    let addr_reg_a = reg[reg_a as usize];
    let vx = get_x_imm(pc, program);
    ((addr_reg_a.wrapping_add(vx)) & RamAddress::MAX as u64) as RamAddress
}

fn get_value<T>(pc: &RegSize, program: &Program) -> RegSize {
    ((get_y_imm(pc, program) as u128) & (1 << (std::mem::size_of::<T>() * 8)) - 1) as RegSize
}

fn store_imm_ind<T>(program: &Program, pc: &mut RegSize, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let address = get_address(pc, reg, program);
    let value = get_value::<T>(pc, program);
    _store::<T>(program, pc, ram, address, value)
}

pub fn store_imm_ind_u8(program: &Program, pc: &mut RegSize, _gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    store_imm_ind::<u8>(program, pc, ram, reg)
}

pub fn store_imm_ind_u16(program: &Program, pc: &mut RegSize, _gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    store_imm_ind::<u16>(program, pc, ram, reg)
}

pub fn store_imm_ind_u32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    store_imm_ind::<u32>(program, pc, ram, reg)
}

pub fn store_imm_ind_u64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    store_imm_ind::<u64>(program, pc, ram, reg)
}
