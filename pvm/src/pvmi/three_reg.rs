/*
    Instructions with Arguments of Three Registers.
*/

use crate::pvm_types::{RamMemory, Registers, Gas, ExitReason, Program, RegSize};
use crate::pvmi::{extend_sign, signed, unsigned};
use crate::pvmi::skip;

#[inline(always)]
fn get_reg(pc: &u64, program: &Program) -> (usize, usize, usize) {
    let reg_a = std::cmp::min(12, program.code[*pc as usize + 1] & 15) as usize;
    let reg_b = std::cmp::min(12, program.code[*pc as usize + 1] >> 4) as usize;
    let reg_d = std::cmp::min(12, program.code[*pc as usize + 2]) as usize;
    (reg_a, reg_b, reg_d)
}

#[inline(always)]
pub fn add_32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);
    let result = reg[reg_a].wrapping_add(reg[reg_b]) as u32;
    reg[reg_d] = extend_sign(&result.to_le_bytes(), 4);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn sub_32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);
    let result = reg[reg_a as usize].wrapping_sub(reg[reg_b as usize]) as u32;
    reg[reg_d as usize] = extend_sign(&result.to_le_bytes(), 4);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn mul_32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);
    let result = reg[reg_a as usize].wrapping_mul(reg[reg_b as usize]) as u32;
    reg[reg_d as usize] = extend_sign(&result.to_le_bytes(), 4);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn div_u_32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);

    if reg[reg_b as usize] & u32::MAX as u64 == 0 {
        reg[reg_d as usize] = u64::MAX;
        *pc += skip(pc, &program.bitmask) + 1;
        return ExitReason::Continue;
    }

    let result = reg[reg_a as usize] as u32 / reg[reg_b as usize] as u32;
    reg[reg_d as usize] = extend_sign(&result.to_le_bytes(), 4);
    *pc += skip(pc, &program.bitmask) + 1;
    return ExitReason::Continue;
}

#[inline(always)]
pub fn div_s_32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);
    let a = signed(reg[reg_a as usize] & u32::MAX as u64, 4);
    let b = signed(reg[reg_b as usize] & u32::MAX as u64, 4);
    if reg[reg_b as usize] == 0 {
        reg[reg_d as usize] = u64::MAX;
    } else if a == i32::MIN as i64 && b == -1 {
        reg[reg_d as usize] = a as RegSize;
    } else {
        reg[reg_d as usize] = unsigned(a / b, 8) as RegSize;
    }
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn rem_u_32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);

    if reg[reg_b as usize] & u32::MAX as u64 == 0 {
        reg[reg_d as usize] = extend_sign(&(reg[reg_a as usize] as u32).to_le_bytes(), 4);
        *pc += skip(pc, &program.bitmask) + 1;
        return ExitReason::Continue;
    }

    let value_reg_a = reg[reg_a as usize] as u32;
    let value_reg_b = reg[reg_b as usize] as u32;
    reg[reg_d as usize] = extend_sign(&(value_reg_a % value_reg_b).to_le_bytes(), 4) as RegSize;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn rem_s_32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);
    let a = signed(reg[reg_a as usize] & u32::MAX as u64, 4);
    let b = signed(reg[reg_b as usize] & u32::MAX as u64, 4);
    if b == 0 {
        reg[reg_d as usize] = unsigned(a, 8) as RegSize;
    } else if a == i32::MIN as i64 && b == -1 { // TODO revisar esto a ver si esta bien
        reg[reg_d as usize] = 0;
    } else {
        reg[reg_d as usize] = unsigned(a % b, 8) as RegSize;
    }
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn shlo_l_32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);
    let value_reg_a = (reg[reg_a as usize] & u32::MAX as u64) as u64;
    let reg_b_mod32 = (reg[reg_b as usize] % 32) as u32;
    let result = (value_reg_a << reg_b_mod32) as u32;
    reg[reg_d as usize] = extend_sign(&result.to_le_bytes(), 4);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn shlo_r_32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);
    let value_reg_a = reg[reg_a as usize] & u32::MAX as u64;
    let reg_b_mod32 = (reg[reg_b as usize] % 32) as u32;
    reg[reg_d as usize] = extend_sign(&((value_reg_a >> reg_b_mod32) as u32).to_le_bytes(), 4);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn shar_r_32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);
    let value_reg_a = reg[reg_a as usize] as u32;
    let reg_b_mod32 = (reg[reg_b as usize] % 32) as u32;
    reg[reg_d as usize] = unsigned(signed(value_reg_a as u64, 4) >> reg_b_mod32, 8) as RegSize;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn add_64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);
    reg[reg_d] = reg[reg_a].wrapping_add(reg[reg_b]) as u64;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn sub_64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);
    reg[reg_d as usize] = (reg[reg_a as usize] as u128).wrapping_sub(reg[reg_b as usize] as u128) as u64;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn and(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);
    reg[reg_d as usize] = reg[reg_a as usize] & reg[reg_b as usize];
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn xor(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);
    reg[reg_d as usize] = reg[reg_a as usize] ^ reg[reg_b as usize];
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn or(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);
    reg[reg_d as usize] = reg[reg_a as usize] | reg[reg_b as usize];
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn mul_upper_s_s(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);
    let a = signed(reg[reg_a as usize], 8) as i128;
    let b = signed(reg[reg_b as usize], 8) as i128;
    let result = a * b;
    reg[reg_d as usize] = unsigned((result >> 64) as i64, 8) as RegSize;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn mul_upper_u_u(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);
    let a = reg[reg_a as usize] as u128;
    let b = reg[reg_b as usize] as u128;
    let result = a * b;
    reg[reg_d as usize] = (result >> 64) as RegSize;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn mul_upper_s_u(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);
    let a = signed(reg[reg_a as usize], 8) as i128;
    let b = reg[reg_b as usize] as i128;
    let result = a * b;
    reg[reg_d as usize] = unsigned((result >> 64) as i64, 8) as RegSize;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn mul_64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);
    reg[reg_d as usize] = reg[reg_a as usize].wrapping_mul(reg[reg_b as usize]) as u64;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn div_u_64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);
    if reg[reg_b as usize] == 0 {
        reg[reg_d as usize] = u64::MAX;
    } else {
        reg[reg_d as usize] = reg[reg_a as usize] / reg[reg_b as usize];
    }
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn div_s_64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);
    let a = signed(reg[reg_a as usize], 8);
    let b = signed(reg[reg_b as usize], 8);
    if reg[reg_b as usize] == 0 {
        reg[reg_d as usize] = u64::MAX;
    } else if a == i64::MIN && b == -1 {
        reg[reg_d as usize] = reg[reg_a as usize];
    } else {
        reg[reg_d as usize] = unsigned(a / b, 8) as RegSize;
    }
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn rem_u_64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);

    if reg[reg_b as usize] == 0 {
        reg[reg_d as usize] = reg[reg_a as usize];
        *pc += skip(pc, &program.bitmask) + 1;
        return ExitReason::Continue;
    }

    let value_reg_a = reg[reg_a as usize] as u64;
    let value_reg_b = reg[reg_b as usize] as u64;
    reg[reg_d as usize] = (value_reg_a % value_reg_b) as RegSize;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn rem_s_64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);
    let value_reg_a = reg[reg_a as usize] as RegSize;
    let value_reg_b = reg[reg_b as usize] as RegSize;
    
    if value_reg_b == 0 {
        reg[reg_d as usize] = value_reg_a;
    } else if signed(value_reg_a, 8) == i64::MIN && signed(value_reg_b, 8) == -1 { // TODO revisar esto a ver si esta bien
        reg[reg_d as usize] = 0;
    } else {
        let a_signed = signed(value_reg_a, 8);
        let b_signed = signed(value_reg_b, 8);
        reg[reg_d as usize] = unsigned(a_signed % b_signed, 8) as RegSize;
    }
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn shlo_l_64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);
    let value_reg_a = reg[reg_a as usize] as u128;
    let reg_b_mod64 = (reg[reg_b as usize] % 64) as u64;
    reg[reg_d as usize] = (value_reg_a << reg_b_mod64) as u64;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn shlo_r_64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);
    let value_reg_a = reg[reg_a as usize] as u64;
    let reg_b_mod64 = reg[reg_b as usize] % 64;
    reg[reg_d as usize] = (value_reg_a >> reg_b_mod64) as RegSize;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn shar_r_64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);
    let value_reg_a = reg[reg_a as usize] as u64;
    let reg_b_mod64 = (reg[reg_b as usize] % 64) as u64;
    reg[reg_d as usize] = unsigned(signed(value_reg_a, 8) >> reg_b_mod64, 8) as RegSize;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn set_lt_u(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);
    if reg[reg_a as usize] < reg[reg_b as usize] {
        reg[reg_d as usize] = 1;
        *pc += skip(pc, &program.bitmask) + 1;
        return ExitReason::Continue;
    } 
    reg[reg_d as usize] = 0;
    *pc += skip(pc, &program.bitmask) + 1;
    return ExitReason::Continue;
}

#[inline(always)]
pub fn set_lt_s(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);
    
    if signed(reg[reg_a], 8) < signed(reg[reg_b], 8) {
        reg[reg_d as usize] = 1;
        *pc += skip(pc, &program.bitmask) + 1;
        return ExitReason::Continue;
    } 

    reg[reg_d as usize] = 0;
    *pc += skip(pc, &program.bitmask) + 1;
    return ExitReason::Continue;
}

#[inline(always)]
pub fn cmov_iz(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);
    if reg[reg_b as usize] == 0 {
        reg[reg_d as usize] = reg[reg_a as usize];
    }
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn cmov_nz(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);
    if reg[reg_b as usize] != 0 {
        reg[reg_d as usize] = reg[reg_a as usize];
    }
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn rot_l_64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);
    let value_reg_a = reg[reg_a as usize];
    let value_reg_b = reg[reg_b as usize];
    reg[reg_d as usize] = value_reg_a.rotate_left(value_reg_b as u32);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn rot_l_32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, &program);
    let value_reg_a = reg[reg_a as usize] as u32;
    let value_reg_b = reg[reg_b as usize] as u32;
    let result = value_reg_a.rotate_left(value_reg_b as u32);
    reg[reg_d as usize] = extend_sign(&result.to_le_bytes(), 4) as RegSize;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn rot_r_64(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, &program);
    let value_reg_a = reg[reg_a as usize];
    let value_reg_b = reg[reg_b as usize];
    reg[reg_d as usize] = value_reg_a.rotate_right(value_reg_b as u32);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn rot_r_32(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, &program);
    let value_reg_a = reg[reg_a as usize] as u32;
    let value_reg_b = reg[reg_b as usize] as u32;
    let result = value_reg_a.rotate_right(value_reg_b as u32);
    reg[reg_d as usize] = extend_sign(&result.to_le_bytes(), 4) as RegSize;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn and_inv(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);
    reg[reg_d as usize] = reg[reg_a as usize] & !reg[reg_b as usize];
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn or_inv(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);
    reg[reg_d as usize] = reg[reg_a as usize] | !reg[reg_b as usize];
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn xnor(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, &program);
    reg[reg_d as usize] = !(reg[reg_a as usize] ^ reg[reg_b as usize]);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn max(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);
    let a = signed(reg[reg_a as usize], 8);
    let b = signed(reg[reg_b as usize], 8);
    reg[reg_d as usize] = std::cmp::max(a, b) as RegSize;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn max_u(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);
    reg[reg_d as usize] = std::cmp::max(reg[reg_a as usize], reg[reg_b as usize]);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn min(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);
    let a = signed(reg[reg_a as usize], 8);
    let b = signed(reg[reg_b as usize], 8);
    reg[reg_d as usize] = std::cmp::min(a, b) as RegSize;
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[inline(always)]
pub fn min_u(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(pc, program);
    reg[reg_d as usize] = std::cmp::min(reg[reg_a as usize], reg[reg_b as usize]);
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}
