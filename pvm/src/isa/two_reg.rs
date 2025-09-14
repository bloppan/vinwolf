/*
    Instructions with Arguments of Two Registers.
*/

use std::cmp::min;
use constants::pvm::PAGE_SIZE;
use crate::mm::program_init;
use crate::pvm_types::{ExitReason, RamMemory, Registers, Gas, Program, RamAddress, RegSize};
use crate::isa::skip;
use crate::isa::{signed, unsigned};

fn get_reg(pc: &u64, code: &[u8]) -> (u8, u8) {
    let reg_a: u8 = min(12, code[*pc as usize + 1] >> 4);
    let reg_d: u8 = min(12, code[*pc as usize + 1] % 16);
    (reg_a, reg_d)
}

#[inline(always)]
pub fn move_reg(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_d) = get_reg(pc, &program.code);
    reg[reg_d as usize] = reg[reg_a as usize];
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn sbrk(program: &Program, pc: &mut RegSize, _gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_d) = get_reg(pc, &program.code);
    let value_a = reg[reg_a as usize];
    //println!("curr_heap_pointer: {:?}", ram.curr_heap_pointer);
    if value_a == 0 {
        //println!("value 0");
        reg[reg_d as usize] = ram.curr_heap_pointer as RegSize;
        *pc += skip(pc, &program.bitmask) + 1;
        return ExitReason::Continue;
    }
    //let result = ram.curr_heap_pointer as RegSize;

    let next_page_boundary = program_init::page(ram.curr_heap_pointer as usize) as RamAddress;
    let new_heap_pointer = ram.curr_heap_pointer + value_a as RamAddress;

    if new_heap_pointer > next_page_boundary {
        let final_boundary = program_init::page(new_heap_pointer as usize) as RamAddress;
        let idx_start = next_page_boundary / PAGE_SIZE;
        let idx_end = final_boundary / PAGE_SIZE;
        let page_count = idx_end - idx_start;    
        ram.allocate_pages(idx_start, page_count);
    }
    
    ram.curr_heap_pointer = new_heap_pointer;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn count_set_bits_64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_d) = get_reg(pc, &program.code);
    reg[reg_d as usize] = reg[reg_a as usize].count_ones() as RegSize;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn count_set_bits_32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_d) = get_reg(pc, &program.code);
    reg[reg_d as usize] = (reg[reg_a as usize] as u32).count_ones() as RegSize;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn leading_zero_bits_64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_d) = get_reg(pc, &program.code);
    reg[reg_d as usize] = reg[reg_a as usize].leading_zeros() as RegSize;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn leading_zero_bits_32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_d) = get_reg(pc, &program.code);
    reg[reg_d as usize] = (reg[reg_a as usize] as u32).leading_zeros() as RegSize;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn trailing_zero_bits_64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_d) = get_reg(pc, &program.code);
    reg[reg_d as usize] = reg[reg_a as usize].trailing_zeros() as RegSize;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn trailing_zero_bits_32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_d) = get_reg(pc, &program.code);
    reg[reg_d as usize] = (reg[reg_a as usize] as u32).trailing_zeros() as RegSize;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn sign_extend_8(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_d) = get_reg(pc, &program.code);
    reg[reg_d as usize] = unsigned(signed((reg[reg_a as usize] as u8) as u64, 1), 8) as RegSize;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn sign_extend_16(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_d) = get_reg(pc, &program.code);
    reg[reg_d as usize] = unsigned(signed((reg[reg_a as usize] as u16) as u64, 2), 8) as RegSize;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn zero_extend_16(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_d) = get_reg(pc, &program.code);
    reg[reg_d as usize] = (reg[reg_a as usize] as u16) as RegSize;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn reverse_bytes(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_d) = get_reg(pc, &program.code);
    reg[reg_d as usize] = reg[reg_a as usize].swap_bytes();
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}