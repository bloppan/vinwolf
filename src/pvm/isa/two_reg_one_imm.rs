/*
    Instructions with Arguments of Two Registers & One Immediate
*/

use std::cmp::{min, max};

use crate::types::{Context, ExitReason, Program, RamAddress, RegSize};
use crate::pvm::isa::{skip, extend_sign, _store, _load, signed, unsigned};

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

fn get_data(pvm_ctx: &mut Context, program: &Program) -> (u8, u8, RegSize, RegSize) {
    let (reg_a, reg_b) = get_reg(&pvm_ctx.pc, &program.code);
    let value_imm = get_imm(&pvm_ctx.pc, program) as RegSize;
    let value_reg_b = pvm_ctx.reg[reg_b as usize] as RegSize;
    (reg_a, reg_b, value_imm, value_reg_b)
}

pub fn shar_r_imm_alt_32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pvm_ctx, program);
    pvm_ctx.reg[reg_a as usize] = unsigned(signed(value_imm % (1 << 32), 4) >> (value_reg_b % 32), 8);
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn cmov_iz_imm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, value_imm, _value_reg_b) = get_data(pvm_ctx, program);
    if pvm_ctx.reg[reg_b as usize] as RegSize == 0 {
        pvm_ctx.reg[reg_a as usize] = value_imm;
    }
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

fn load<T>(pvm_ctx: &mut Context, program: &Program, signed: bool) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pvm_ctx, program);
    let address: RamAddress = (value_reg_b as RamAddress).wrapping_add(value_imm as RamAddress);
    _load::<T>(pvm_ctx, program, address, reg_a as u64, signed)
}

pub fn store_ind_u8(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pvm_ctx, program);
    let address: RamAddress = (value_reg_b as RamAddress).wrapping_add(value_imm as RamAddress);
    let value = (pvm_ctx.reg[reg_a as usize] % (1 << 8)) as u8;
    _store::<u8>(pvm_ctx, program, address, value as u64)
}

pub fn store_ind_u16(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pvm_ctx, program);
    let address: RamAddress = (value_reg_b as RamAddress).wrapping_add(value_imm as RamAddress);
    let value = (pvm_ctx.reg[reg_a as usize] % (1 << 16)) as u16;
    _store::<u16>(pvm_ctx, program, address, value as u64)
}

pub fn store_ind_u32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pvm_ctx, program);
    let address: RamAddress = (value_reg_b as RamAddress).wrapping_add(value_imm as RamAddress);
    let value = (pvm_ctx.reg[reg_a as usize] % (1 << 32)) as u32;
    _store::<u32>(pvm_ctx, program, address, value as u64)
}

pub fn store_ind_u64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pvm_ctx, program);
    let address: RamAddress = (value_reg_b as RamAddress).wrapping_add(value_imm as RamAddress);
    let value = pvm_ctx.reg[reg_a as usize] as u64;
    _store::<u64>(pvm_ctx, program, address, value)
}

pub fn load_ind_u8(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    load::<u8>(pvm_ctx, program, false)
}

pub fn load_ind_i8(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    load::<i8>(pvm_ctx, program, true)
}

pub fn load_ind_u16(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    load::<u16>(pvm_ctx, program, false)
}

pub fn load_ind_i16(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    load::<i16>(pvm_ctx, program, true)
}

pub fn load_ind_u32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    load::<u32>(pvm_ctx, program, false)
}

pub fn load_ind_i32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    load::<i32>(pvm_ctx, program, true)
}

pub fn load_ind_u64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    load::<u64>(pvm_ctx, program, false)
}

pub fn add_imm_32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pvm_ctx, program);
    let result = (value_reg_b.wrapping_add(value_imm) % (1 << 32)) as u32;
    pvm_ctx.reg[reg_a as usize] = extend_sign(&result.to_le_bytes(), 4);
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn and_imm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, value_imm, _value_reg_b) = get_data(pvm_ctx, program);
    pvm_ctx.reg[reg_a as usize] = pvm_ctx.reg[reg_b as usize] & value_imm;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn xor_imm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, value_imm, _value_reg_b) = get_data(pvm_ctx, program);
    pvm_ctx.reg[reg_a as usize] = pvm_ctx.reg[reg_b as usize] ^ value_imm;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn or_imm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, value_imm, _value_reg_b) = get_data(pvm_ctx, program);
    pvm_ctx.reg[reg_a as usize] = pvm_ctx.reg[reg_b as usize] | value_imm;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn mul_imm_32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pvm_ctx, program);
    let result = (value_reg_b.wrapping_mul(value_imm ) % (1 << 32)) as u32;
    pvm_ctx.reg[reg_a as usize] = extend_sign(&result.to_le_bytes(), 4);
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn set_lt_u_imm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pvm_ctx, program);

    if (value_reg_b as RegSize) < (value_imm as RegSize) {
        pvm_ctx.reg[reg_a as usize] = 1;
        pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
        return ExitReason::Continue;
    }

    pvm_ctx.reg[reg_a as usize] = 0;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    return ExitReason::Continue;
}

pub fn set_lt_s_imm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pvm_ctx, program);

    if signed(value_reg_b, 8) < signed(value_imm, 8) {
        pvm_ctx.reg[reg_a as usize] = 1;
        pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
        return ExitReason::Continue;
    } 

    pvm_ctx.reg[reg_a as usize] = 0;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    return ExitReason::Continue;
}

pub fn shlo_l_imm_32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pvm_ctx, program);
    let result = ((value_reg_b << (value_imm % 32)) % (1 << 32)) as u32;
    pvm_ctx.reg[reg_a as usize] = extend_sign(&result.to_le_bytes(), 4);
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn shlo_r_imm_32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pvm_ctx, program);
    let result = (((value_reg_b % (1 << 32)) as u32) >> (value_imm % 32)) as u32;
    pvm_ctx.reg[reg_a as usize] = extend_sign(&result.to_le_bytes(), 4);
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn shar_r_imm_32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pvm_ctx, program);
    pvm_ctx.reg[reg_a as usize] = unsigned(signed(value_reg_b % (1 << 32), 4) >> (value_imm % 32), 8);
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn neg_add_imm_32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pvm_ctx, program);
    let result = (value_imm.wrapping_add(1 << 32).wrapping_sub(value_reg_b) % (1 << 32)) as u32;
    pvm_ctx.reg[reg_a as usize] = extend_sign(&result.to_le_bytes(), 4);
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn set_gt_u_imm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pvm_ctx, program);

    if value_reg_b as RegSize > value_imm as RegSize {
        pvm_ctx.reg[reg_a as usize] = 1;
        pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
        return ExitReason::Continue;
    }

    pvm_ctx.reg[reg_a as usize] = 0;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    return ExitReason::Continue;
}

pub fn set_gt_s_imm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pvm_ctx, program);

    if signed(value_reg_b, 8) > signed(value_imm, 8) {
        pvm_ctx.reg[reg_a as usize] = 1;
        pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
        return ExitReason::Continue;
    } 
    
    pvm_ctx.reg[reg_a as usize] = 0;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    return ExitReason::Continue;
}

pub fn shlo_l_imm_alt_32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pvm_ctx, program);
    let result = ((value_imm << (value_reg_b % 32)) % (1 << 32)) as u32;
    pvm_ctx.reg[reg_a as usize] = extend_sign(&result.to_le_bytes(), 4);
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn shlo_r_imm_alt_32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pvm_ctx, program);
    let result = ((value_imm % (1 << 32)) >> (value_reg_b % 32)) as u32;
    pvm_ctx.reg[reg_a as usize] = extend_sign(&result.to_le_bytes(), 4);
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn add_imm_64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pvm_ctx, program);
    pvm_ctx.reg[reg_a as usize] = value_reg_b.wrapping_add(value_imm) as u64;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn mul_imm_64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pvm_ctx, program);
    let result = (value_reg_b as u64).wrapping_mul(value_imm as u64) as u64;
    pvm_ctx.reg[reg_a as usize] = extend_sign(&result.to_le_bytes(), 4);
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn shlo_l_imm_64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pvm_ctx, program);
    let result = (value_reg_b as u64) << (value_imm % 64);
    pvm_ctx.reg[reg_a as usize] = extend_sign(&result.to_le_bytes(), 8);
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn shlo_r_imm_64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pvm_ctx, program);
    let result = (value_reg_b >> (value_imm % 64)) as u64;
    pvm_ctx.reg[reg_a as usize] = extend_sign(&result.to_le_bytes(), 8);
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn shar_r_imm_64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pvm_ctx, program);
    pvm_ctx.reg[reg_a as usize] = unsigned(signed(value_reg_b, 8) >> (value_imm % 64), 8);
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn neg_add_imm_64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pvm_ctx, program);
    pvm_ctx.reg[reg_a as usize] = ((value_imm as u128).wrapping_add(1 << 64).wrapping_sub(value_reg_b as u128) % (1 << 64)) as u64;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn shlo_l_imm_alt_64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pvm_ctx, program);
    pvm_ctx.reg[reg_a as usize] = ((value_imm << (value_reg_b % 64))) as u64;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn shlo_r_imm_alt_64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pvm_ctx, program);
    pvm_ctx.reg[reg_a as usize] = (value_imm >> (value_reg_b % 64)) as u64;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn shar_r_imm_alt_64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pvm_ctx, program);
    pvm_ctx.reg[reg_a as usize] = unsigned(signed(value_imm, 8) >> (value_reg_b % 64), 8);
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn rot_r_64_imm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pvm_ctx, program);
    let mut result: u64 = 0;
    for i in 0..64 {
        let bit_b = (value_reg_b >> ((i as u64).wrapping_add(value_imm) % 64)) & 1;
        result |= bit_b << i;
    }
    pvm_ctx.reg[reg_a as usize] = result;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn rot_r_64_imm_alt(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pvm_ctx, program);
    let mut result: u64 = 0;
    for i in 0..64 {
        let bit_b = (value_imm >> (((i as u64).wrapping_add(value_reg_b)) % 64)) & 1;
        result |= bit_b << i;
    }
    pvm_ctx.reg[reg_a as usize] = result;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn rot_r_32_imm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pvm_ctx, program);
    let mut result: u32 = 0;
    for i in 0..32 {
        let bit_b = ((value_reg_b as u32) >> ((i as u64).wrapping_add(value_imm) % 32)) & 1;
        result |= bit_b << i;
    }
    pvm_ctx.reg[reg_a as usize] = extend_sign(&result.to_le_bytes(), 4);
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn rot_r_32_imm_alt(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, _reg_b, value_imm, value_reg_b) = get_data(pvm_ctx, program);
    let mut result: u32 = 0;
    for i in 0..32 {
        let bit_b = ((value_imm as u32) >> (((i as u64).wrapping_add(value_reg_b)) % 32)) & 1;
        result |= bit_b << i;
    }
    pvm_ctx.reg[reg_a as usize] = extend_sign(&result.to_le_bytes(), 4);
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}
