/*
    Instructions with Arguments of Two Registers & One Immediate
*/

use std::cmp::{min, max};

use crate::pvm_types::{RamMemory, Gas, Registers, ExitReason, Program, RamAddress, RegSize};
use crate::isa::{skip, extend_sign, _store, _load, signed, unsigned};

fn get_imm(pc: &RegSize, program: &Program) -> RegSize {
   let start= (*pc + 2) as usize;
   let end = start + get_x_length(pc, program) as usize;
   utils::log::trace!("lx: {:?}", get_x_length(pc, program));
   extend_sign(&program.code[start..end], get_x_length(pc, program) as usize) as RegSize
}

fn get_x_length(pc: &RegSize, program: &Program) -> RegSize {
    min(4, max(0, skip(pc, &program.bitmask).saturating_sub(1)))
}

fn get_reg(pc: &RegSize, code: &[u8]) -> (u8, u8) {
    let reg_a: u8 = min(12, code[*pc as usize + 1] & 15);
    let reg_b: u8 = min(12, code[*pc as usize + 1] >> 4);
    (reg_a, reg_b)
}

fn get_data(pc: &RegSize, reg: &Registers, program: &Program) -> (u8, u8, RegSize, RegSize) {
    let (reg_a, reg_b) = get_reg(pc, &program.code);
    let value_imm = get_imm(pc, program) as RegSize;
    let value_reg_b = reg[reg_b as usize] as RegSize;
    (reg_a, reg_b, value_imm, value_reg_b)
}

#[inline(always)]
pub fn shar_r_imm_alt_32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    reg[reg_a as usize] = unsigned(signed(value_imm & (u32::MAX as u64), 4) >> (value_reg_b % 32), 8);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn cmov_iz_imm(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, value_imm, _value_reg_b) = get_data(pc, reg, program);
    if reg[reg_b as usize] as RegSize == 0 {
        reg[reg_a as usize] = value_imm;
    }
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn cmov_nz_imm(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, value_imm, _value_reg_b) = get_data(pc, reg, program);
    if reg[reg_b as usize] as RegSize != 0 {
        reg[reg_a as usize] = value_imm;
    }
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

fn load<T>(program: &Program, pc: &mut RegSize, ram: &mut RamMemory, reg: &mut Registers, signed: bool) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    let address: RamAddress = (value_reg_b as RamAddress).wrapping_add(value_imm as RamAddress);
    _load::<T>(program, pc, ram, reg, address, reg_a as u64, signed)
}

pub fn store_ind_u8(program: &Program, pc: &mut RegSize, _gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    let address: RamAddress = (value_reg_b as RamAddress).wrapping_add(value_imm as RamAddress);
    let value = reg[reg_a as usize] as u8;
    _store::<u8>(program, pc, ram, address, value as u64)
}

pub fn store_ind_u16(program: &Program, pc: &mut RegSize, _gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    let address: RamAddress = (value_reg_b as RamAddress).wrapping_add(value_imm as RamAddress);
    let value = reg[reg_a as usize] as u16;
    _store::<u16>(program, pc, ram, address, value as u64)
}

pub fn store_ind_u32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    let address: RamAddress = (value_reg_b as RamAddress).wrapping_add(value_imm as RamAddress);
    let value = reg[reg_a as usize] as u32;
    _store::<u32>(program, pc, ram, address, value as u64)
}

pub fn store_ind_u64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    let address: RamAddress = (value_reg_b as RamAddress).wrapping_add(value_imm as RamAddress);
    let value = reg[reg_a as usize] as u64;
    _store::<u64>(program, pc, ram, address, value)
}

pub fn load_ind_u8(program: &Program, pc: &mut RegSize, _gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    load::<u8>(program, pc, ram, reg, false)
}

pub fn load_ind_i8(program: &Program, pc: &mut RegSize, _gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    load::<i8>(program, pc, ram, reg, true)
}

pub fn load_ind_u16(program: &Program, pc: &mut RegSize, _gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    load::<u16>(program, pc, ram, reg, false)
}

pub fn load_ind_i16(program: &Program, pc: &mut RegSize, _gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    load::<i16>(program, pc, ram, reg, true)
}

pub fn load_ind_u32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    load::<u32>(program, pc, ram, reg, false)
}

pub fn load_ind_i32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    load::<i32>(program, pc, ram, reg, true)
}

pub fn load_ind_u64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    load::<u64>(program, pc, ram, reg, false)
}

#[inline(always)]
pub fn add_imm_32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    let result = value_reg_b.wrapping_add(value_imm) as u32;
    reg[reg_a as usize] = extend_sign(&result.to_le_bytes(), 4);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn and_imm(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, value_imm, _value_reg_b) = get_data(pc, reg, program);
    reg[reg_a as usize] = reg[reg_b as usize] & value_imm;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn xor_imm(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, value_imm, _value_reg_b) = get_data(pc, reg, program);
    reg[reg_a as usize] = reg[reg_b as usize] ^ value_imm;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn or_imm(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, value_imm, _value_reg_b) = get_data(pc, reg, program);
    reg[reg_a as usize] = reg[reg_b as usize] | value_imm;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn mul_imm_32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    let result = value_reg_b.wrapping_mul(value_imm ) as u32;
    reg[reg_a as usize] = extend_sign(&result.to_le_bytes(), 4);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn set_lt_u_imm(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);

    if (value_reg_b as RegSize) < (value_imm as RegSize) {
        reg[reg_a as usize] = 1;
        *pc += skip(pc, &program.bitmask) + 1;
        return ExitReason::Continue;
    }

    reg[reg_a as usize] = 0;
    *pc += skip(pc, &program.bitmask) + 1;
    return ExitReason::Continue;
}

#[inline(always)]
pub fn set_lt_s_imm(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);

    if signed(value_reg_b, 8) < signed(value_imm, 8) {
        reg[reg_a as usize] = 1;
        *pc += skip(pc, &program.bitmask) + 1;
        return ExitReason::Continue;
    } 

    reg[reg_a as usize] = 0;
    *pc += skip(pc, &program.bitmask) + 1;
    return ExitReason::Continue;
}

#[inline(always)]
pub fn shlo_l_imm_32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    let result = (value_reg_b << (value_imm & 31)) as u32;
    reg[reg_a as usize] = extend_sign(&result.to_le_bytes(), 4);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn shlo_r_imm_32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    let result = ((value_reg_b  as u32) >> (value_imm & 31)) as u32;
    reg[reg_a as usize] = extend_sign(&result.to_le_bytes(), 4);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn shar_r_imm_32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    reg[reg_a as usize] = unsigned(signed(value_reg_b & (u32::MAX as u64), 4) >> (value_imm % 32), 8);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn neg_add_imm_32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    let result = (value_imm.wrapping_sub(value_reg_b)) as u32;
    reg[reg_a as usize] = extend_sign(&result.to_le_bytes(), 4);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn set_gt_u_imm(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);

    if value_reg_b as RegSize > value_imm as RegSize {
        reg[reg_a as usize] = 1;
        *pc += skip(pc, &program.bitmask) + 1;
        return ExitReason::Continue;
    }

    reg[reg_a as usize] = 0;
    *pc += skip(pc, &program.bitmask) + 1;
    return ExitReason::Continue;
}

#[inline(always)]
pub fn set_gt_s_imm(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    utils::log::trace!("reg_b: {value_reg_b}, value_imm: {value_imm}");
    if signed(value_reg_b, 8) > signed(value_imm, 8) {
        reg[reg_a as usize] = 1;
        *pc += skip(pc, &program.bitmask) + 1;
        return ExitReason::Continue;
    } 
    
    reg[reg_a as usize] = 0;
    *pc += skip(pc, &program.bitmask) + 1;
    return ExitReason::Continue;
}

#[inline(always)]
pub fn shlo_l_imm_alt_32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    let result = (value_imm << (value_reg_b & 31)) as u32;
    reg[reg_a as usize] = extend_sign(&result.to_le_bytes(), 4);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn shlo_r_imm_alt_32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    let result = ((value_imm as u32) >> (value_reg_b & 31)) as u32;
    reg[reg_a as usize] = extend_sign(&result.to_le_bytes(), 4);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn add_imm_64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    reg[reg_a as usize] = value_reg_b.wrapping_add(value_imm) as u64;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn mul_imm_64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    let result = (value_reg_b as u64).wrapping_mul(value_imm as u64) as u64;
    reg[reg_a as usize] = extend_sign(&result.to_le_bytes(), 8);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn shlo_l_imm_64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    let result = (value_reg_b as u64) << (value_imm & 63);
    reg[reg_a as usize] = extend_sign(&result.to_le_bytes(), 8);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn shlo_r_imm_64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    let result = (value_reg_b >> (value_imm & 63)) as u64;
    reg[reg_a as usize] = extend_sign(&result.to_le_bytes(), 8);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn shar_r_imm_64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    reg[reg_a as usize] = unsigned(signed(value_reg_b, 8) >> (value_imm & 63), 8);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn neg_add_imm_64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    reg[reg_a as usize] = (value_imm as u128).wrapping_sub(value_reg_b as u128) as u64;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn shlo_l_imm_alt_64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    reg[reg_a as usize] = (value_imm << (value_reg_b & 63)) as u64;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn shlo_r_imm_alt_64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    reg[reg_a as usize] = (value_imm >> (value_reg_b & 63)) as u64;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn shar_r_imm_alt_64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    reg[reg_a as usize] = unsigned(signed(value_imm, 8) >> value_reg_b, 8);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn rot_r_64_imm(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    let result = value_reg_b.rotate_right(value_imm as u32);
    reg[reg_a as usize] = result;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn rot_r_64_imm_alt(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    let result = value_imm.rotate_right(value_reg_b as u32);
    reg[reg_a as usize] = result;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn rot_r_32_imm(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    let rotated_u32 = (value_reg_b as u32).rotate_right(value_imm as u32);
    reg[reg_a as usize] = extend_sign(&rotated_u32.to_le_bytes(), 4);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn rot_r_32_imm_alt(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    let rotated_u32 = (value_imm as u32).rotate_right(value_reg_b as u32);
    reg[reg_a as usize] = extend_sign(&rotated_u32.to_le_bytes(), 4);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}
