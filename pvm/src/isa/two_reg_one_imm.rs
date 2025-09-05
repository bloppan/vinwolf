/*
    Instructions with Arguments of Two Registers & One Immediate
*/

use std::cmp::{min, max};

use crate::pvm_types::{RamMemory, Gas, Registers, ExitReason, Program, RamAddress, RegSize};
use crate::isa::{skip, extend_sign, _store, _load, signed, unsigned};

fn get_imm(pc: &RegSize, program: &Program) -> RegSize {
   let start= (*pc + 2) as usize;
   let end = start + get_x_length(pc, program) as usize;
   extend_sign(&program.code[start..end], get_x_length(pc, program) as usize) as RegSize
}

fn get_x_length(pc: &RegSize, program: &Program) -> RegSize {
    min(4, max(0, skip(pc, &program.bitmask).saturating_sub(1)))
}

fn get_reg(pc: &RegSize, code: &[u8]) -> (u8, u8) {
    let reg_a: u8 = min(12, code[*pc as usize + 1] % 16);
    let reg_b: u8 = min(12, code[*pc as usize + 1] / 16);
    (reg_a, reg_b)
}

fn get_data(pc: &RegSize, reg: &Registers, program: &Program) -> (u8, u8, RegSize, RegSize) {
    let (reg_a, reg_b) = get_reg(pc, &program.code);
    let value_imm = get_imm(pc, program) as RegSize;
    let value_reg_b = reg[reg_b as usize] as RegSize;
    (reg_a, reg_b, value_imm, value_reg_b)
}

pub fn shar_r_imm_alt_32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    reg[reg_a as usize] = unsigned(signed(value_imm % (1 << 32), 4) >> (value_reg_b % 32), 8);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn cmov_iz_imm(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, value_imm, _value_reg_b) = get_data(pc, reg, program);
    if reg[reg_b as usize] as RegSize == 0 {
        reg[reg_a as usize] = value_imm;
    }
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

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
    let value = (reg[reg_a as usize] % (1 << 8)) as u8;
    _store::<u8>(program, pc, ram, address, value as u64)
}

pub fn store_ind_u16(program: &Program, pc: &mut RegSize, _gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    let address: RamAddress = (value_reg_b as RamAddress).wrapping_add(value_imm as RamAddress);
    let value = (reg[reg_a as usize] % (1 << 16)) as u16;
    _store::<u16>(program, pc, ram, address, value as u64)
}

pub fn store_ind_u32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    let address: RamAddress = (value_reg_b as RamAddress).wrapping_add(value_imm as RamAddress);
    let value = (reg[reg_a as usize] % (1 << 32)) as u32;
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

pub fn add_imm_32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    let result = (value_reg_b.wrapping_add(value_imm) % (1 << 32)) as u32;
    reg[reg_a as usize] = extend_sign(&result.to_le_bytes(), 4);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn and_imm(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, value_imm, _value_reg_b) = get_data(pc, reg, program);
    reg[reg_a as usize] = reg[reg_b as usize] & value_imm;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn xor_imm(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, value_imm, _value_reg_b) = get_data(pc, reg, program);
    reg[reg_a as usize] = reg[reg_b as usize] ^ value_imm;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn or_imm(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, value_imm, _value_reg_b) = get_data(pc, reg, program);
    reg[reg_a as usize] = reg[reg_b as usize] | value_imm;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn mul_imm_32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    let result = (value_reg_b.wrapping_mul(value_imm ) % (1 << 32)) as u32;
    reg[reg_a as usize] = extend_sign(&result.to_le_bytes(), 4);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

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

pub fn shlo_l_imm_32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    let result = ((value_reg_b << (value_imm % 32)) % (1 << 32)) as u32;
    reg[reg_a as usize] = extend_sign(&result.to_le_bytes(), 4);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn shlo_r_imm_32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    let result = (((value_reg_b % (1 << 32)) as u32) >> (value_imm % 32)) as u32;
    reg[reg_a as usize] = extend_sign(&result.to_le_bytes(), 4);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn shar_r_imm_32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    reg[reg_a as usize] = unsigned(signed(value_reg_b % (1 << 32), 4) >> (value_imm % 32), 8);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn neg_add_imm_32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    let result = (value_imm.wrapping_add(1 << 32).wrapping_sub(value_reg_b) % (1 << 32)) as u32;
    reg[reg_a as usize] = extend_sign(&result.to_le_bytes(), 4);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

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

pub fn set_gt_s_imm(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);

    if signed(value_reg_b, 8) > signed(value_imm, 8) {
        reg[reg_a as usize] = 1;
        *pc += skip(pc, &program.bitmask) + 1;
        return ExitReason::Continue;
    } 
    
    reg[reg_a as usize] = 0;
    *pc += skip(pc, &program.bitmask) + 1;
    return ExitReason::Continue;
}

pub fn shlo_l_imm_alt_32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    let result = ((value_imm << (value_reg_b % 32)) % (1 << 32)) as u32;
    reg[reg_a as usize] = extend_sign(&result.to_le_bytes(), 4);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn shlo_r_imm_alt_32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    let result = ((value_imm % (1 << 32)) >> (value_reg_b % 32)) as u32;
    reg[reg_a as usize] = extend_sign(&result.to_le_bytes(), 4);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn add_imm_64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    reg[reg_a as usize] = value_reg_b.wrapping_add(value_imm) as u64;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn mul_imm_64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    let result = (value_reg_b as u64).wrapping_mul(value_imm as u64) as u64;
    reg[reg_a as usize] = extend_sign(&result.to_le_bytes(), 8);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn shlo_l_imm_64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    let result = (value_reg_b as u64) << (value_imm % 64);
    reg[reg_a as usize] = extend_sign(&result.to_le_bytes(), 8);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn shlo_r_imm_64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    let result = (value_reg_b >> (value_imm % 64)) as u64;
    reg[reg_a as usize] = extend_sign(&result.to_le_bytes(), 8);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn shar_r_imm_64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    reg[reg_a as usize] = unsigned(signed(value_reg_b, 8) >> (value_imm % 64), 8);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn neg_add_imm_64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    reg[reg_a as usize] = ((value_imm as u128).wrapping_add(1 << 64).wrapping_sub(value_reg_b as u128) % (1 << 64)) as u64;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn shlo_l_imm_alt_64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    reg[reg_a as usize] = ((value_imm << (value_reg_b % 64))) as u64;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn shlo_r_imm_alt_64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    reg[reg_a as usize] = (value_imm >> (value_reg_b % 64)) as u64;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn shar_r_imm_alt_64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    reg[reg_a as usize] = unsigned(signed(value_imm, 8) >> (value_reg_b % 64), 8);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn rot_r_64_imm(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    let mut result: u64 = 0;
    for i in 0..64 {
        let bit_b = (value_reg_b >> ((i as u64).wrapping_add(value_imm) % 64)) & 1;
        result |= bit_b << i;
    }
    reg[reg_a as usize] = result;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn rot_r_64_imm_alt(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    let mut result: u64 = 0;
    for i in 0..64 {
        let bit_b = (value_imm >> (((i as u64).wrapping_add(value_reg_b)) % 64)) & 1;
        result |= bit_b << i;
    }
    reg[reg_a as usize] = result;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn rot_r_32_imm(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    let mut result: u32 = 0;
    for i in 0..32 {
        let bit_b = ((value_reg_b as u32) >> ((i as u64).wrapping_add(value_imm) % 32)) & 1;
        result |= bit_b << i;
    }
    reg[reg_a as usize] = extend_sign(&result.to_le_bytes(), 4);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn rot_r_32_imm_alt(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pc, reg, program);
    let mut result: u32 = 0;
    for i in 0..32 {
        let bit_b = ((value_imm as u32) >> (((i as u64).wrapping_add(value_reg_b)) % 32)) & 1;
        result |= bit_b << i;
    }
    reg[reg_a as usize] = extend_sign(&result.to_le_bytes(), 4);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}
