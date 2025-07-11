/*
    Instructions with Arguments of Three Registers.
*/

use crate::jam_types::{Context, ExitReason, Program, RegSize};
use crate::pvm::isa::{extend_sign, signed, unsigned};
use crate::pvm::isa::skip;

fn get_reg(pc: &u64, program: &Program) -> (usize, usize, usize) {
    let reg_a = std::cmp::min(12, program.code[*pc as usize + 1] % 16) as usize;
    let reg_b = std::cmp::min(12, program.code[*pc as usize + 1] >> 4) as usize;
    let reg_d = std::cmp::min(12, program.code[*pc as usize + 2]) as usize;
    (reg_a, reg_b, reg_d)
}

pub fn add_32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    let result = (pvm_ctx.reg[reg_a].wrapping_add(pvm_ctx.reg[reg_b]) % (1 << 32)) as u32;
    pvm_ctx.reg[reg_d] = extend_sign(&result.to_le_bytes(), 4);
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn sub_32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    let result = (pvm_ctx.reg[reg_a as usize].wrapping_add(1 << 32).wrapping_sub(pvm_ctx.reg[reg_b as usize] % (1 << 32)) % (1 << 32)) as u32;
    pvm_ctx.reg[reg_d as usize] = extend_sign(&result.to_le_bytes(), 4);
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn mul_32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    let result = pvm_ctx.reg[reg_a as usize].wrapping_mul(pvm_ctx.reg[reg_b as usize] % (1 << 32)) as u32;
    pvm_ctx.reg[reg_d as usize] = extend_sign(&result.to_le_bytes(), 4);
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn div_u_32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);

    if pvm_ctx.reg[reg_b as usize] % (1 << 32) == 0 {
        pvm_ctx.reg[reg_d as usize] = u64::MAX;
        pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
        return ExitReason::Continue;
    }

    let result = pvm_ctx.reg[reg_a as usize] as u32 / pvm_ctx.reg[reg_b as usize] as u32;
    pvm_ctx.reg[reg_d as usize] = extend_sign(&result.to_le_bytes(), 4);
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    return ExitReason::Continue;
}

pub fn div_s_32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    let a = signed(pvm_ctx.reg[reg_a as usize] % (1 << 32), 4);
    let b = signed(pvm_ctx.reg[reg_b as usize] % (1 << 32), 4);
    if pvm_ctx.reg[reg_b as usize] == 0 {
        pvm_ctx.reg[reg_d as usize] = u64::MAX;
    } else if a == i32::MIN as i64 && b == -1 {
        pvm_ctx.reg[reg_d as usize] = a as RegSize;
    } else {
        pvm_ctx.reg[reg_d as usize] = unsigned(a / b, 8) as RegSize;
    }
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn rem_u_32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);

    if pvm_ctx.reg[reg_b as usize] % (1 << 32) == 0 {
        pvm_ctx.reg[reg_d as usize] = extend_sign(&(pvm_ctx.reg[reg_a as usize] as u32).to_le_bytes(), 4);
        pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
        return ExitReason::Continue;
    }

    let value_reg_a = (pvm_ctx.reg[reg_a as usize] % (1 << 32)) as u32;
    let value_reg_b = (pvm_ctx.reg[reg_b as usize] % (1 << 32)) as u32;
    pvm_ctx.reg[reg_d as usize] = extend_sign(&(value_reg_a % value_reg_b).to_le_bytes(), 4) as RegSize;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn rem_s_32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    let a = signed(pvm_ctx.reg[reg_a as usize] % (1 << 32), 4);
    let b = signed(pvm_ctx.reg[reg_b as usize] % (1 << 32), 4);
    if b == 0 {
        pvm_ctx.reg[reg_d as usize] = unsigned(a, 8) as RegSize;
    } else if a == i32::MIN as i64 && b == -1 { // TODO revisar esto a ver si esta bien
        pvm_ctx.reg[reg_d as usize] = 0;
    } else {
        pvm_ctx.reg[reg_d as usize] = unsigned(a % b, 8) as RegSize;
    }
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn shlo_l_32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    let value_reg_a = (pvm_ctx.reg[reg_a as usize] % (1 << 32)) as u64;
    let reg_b_mod32 = (pvm_ctx.reg[reg_b as usize] % 32) as u32;
    let result = ((value_reg_a << reg_b_mod32) % (1 << 32)) as u32;
    pvm_ctx.reg[reg_d as usize] = extend_sign(&result.to_le_bytes(), 4);
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn shlo_r_32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    let value_reg_a = pvm_ctx.reg[reg_a as usize] % (1 << 32);
    let reg_b_mod32 = (pvm_ctx.reg[reg_b as usize] % 32) as u32;
    pvm_ctx.reg[reg_d as usize] = extend_sign(&((value_reg_a >> reg_b_mod32) as u32).to_le_bytes(), 4);
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn shar_r_32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    let value_reg_a = (pvm_ctx.reg[reg_a as usize] % (1 << 32)) as u32;
    let reg_b_mod32 = (pvm_ctx.reg[reg_b as usize] % 32) as u32;
    pvm_ctx.reg[reg_d as usize] = unsigned(signed(value_reg_a as u64, 4) >> reg_b_mod32, 8) as RegSize;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn add_64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    pvm_ctx.reg[reg_d] = pvm_ctx.reg[reg_a].wrapping_add(pvm_ctx.reg[reg_b]) as u64;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn sub_64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    pvm_ctx.reg[reg_d as usize] = ((pvm_ctx.reg[reg_a as usize] as u128).wrapping_add(1 << 64).wrapping_sub(pvm_ctx.reg[reg_b as usize] as u128) % (1 << 64)) as u64;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn and(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    pvm_ctx.reg[reg_d as usize] = pvm_ctx.reg[reg_a as usize] & pvm_ctx.reg[reg_b as usize];
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn xor(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    pvm_ctx.reg[reg_d as usize] = pvm_ctx.reg[reg_a as usize] ^ pvm_ctx.reg[reg_b as usize];
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn or(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    pvm_ctx.reg[reg_d as usize] = pvm_ctx.reg[reg_a as usize] | pvm_ctx.reg[reg_b as usize];
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn mul_upper_s_s(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    let a = signed(pvm_ctx.reg[reg_a as usize], 8) as i128;
    let b = signed(pvm_ctx.reg[reg_b as usize], 8) as i128;
    let result = a * b;
    pvm_ctx.reg[reg_d as usize] = unsigned((result >> 64) as i64, 8) as RegSize;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn mul_upper_u_u(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    let a = pvm_ctx.reg[reg_a as usize] as u128;
    let b = pvm_ctx.reg[reg_b as usize] as u128;
    let result = a * b;
    pvm_ctx.reg[reg_d as usize] = (result >> 64) as RegSize;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn mul_upper_s_u(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    let a = signed(pvm_ctx.reg[reg_a as usize], 8) as i128;
    let b = pvm_ctx.reg[reg_b as usize] as i128;
    let result = a * b;
    pvm_ctx.reg[reg_d as usize] = unsigned((result >> 64) as i64, 8) as RegSize;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn mul_64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    pvm_ctx.reg[reg_d as usize] = pvm_ctx.reg[reg_a as usize].wrapping_mul(pvm_ctx.reg[reg_b as usize]) as u64;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn div_u_64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    if pvm_ctx.reg[reg_b as usize] == 0 {
        pvm_ctx.reg[reg_d as usize] = u64::MAX;
    } else {
        pvm_ctx.reg[reg_d as usize] = pvm_ctx.reg[reg_a as usize] / pvm_ctx.reg[reg_b as usize];
    }
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn div_s_64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    let a = signed(pvm_ctx.reg[reg_a as usize], 8);
    let b = signed(pvm_ctx.reg[reg_b as usize], 8);
    if pvm_ctx.reg[reg_b as usize] == 0 {
        pvm_ctx.reg[reg_d as usize] = u64::MAX;
    } else if a == i64::MIN && b == -1 {
        pvm_ctx.reg[reg_d as usize] = pvm_ctx.reg[reg_a as usize];
    } else {
        pvm_ctx.reg[reg_d as usize] = unsigned(a / b, 8) as RegSize;
    }
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn rem_u_64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);

    if pvm_ctx.reg[reg_b as usize] == 0 {
        pvm_ctx.reg[reg_d as usize] = pvm_ctx.reg[reg_a as usize];
        pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
        return ExitReason::Continue;
    }

    let value_reg_a = pvm_ctx.reg[reg_a as usize] as u64;
    let value_reg_b = pvm_ctx.reg[reg_b as usize] as u64;
    pvm_ctx.reg[reg_d as usize] = (value_reg_a % value_reg_b) as RegSize;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn rem_s_64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    let value_reg_a = pvm_ctx.reg[reg_a as usize] as RegSize;
    let value_reg_b = pvm_ctx.reg[reg_b as usize] as RegSize;
    
    if value_reg_b == 0 {
        pvm_ctx.reg[reg_d as usize] = value_reg_a;
    } else if signed(value_reg_a, 8) == i64::MIN && signed(value_reg_b, 8) == -1 { // TODO revisar esto a ver si esta bien
        pvm_ctx.reg[reg_d as usize] = 0;
    } else {
        let a_signed = signed(value_reg_a, 8);
        let b_signed = signed(value_reg_b, 8);
        pvm_ctx.reg[reg_d as usize] = unsigned(a_signed % b_signed, 8) as RegSize;
    }
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn shlo_l_64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    let value_reg_a = pvm_ctx.reg[reg_a as usize] as u128;
    let reg_b_mod64 = (pvm_ctx.reg[reg_b as usize] % 64) as u64;
    pvm_ctx.reg[reg_d as usize] = ((value_reg_a << reg_b_mod64) % (1 << 64)) as u64;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn shlo_r_64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    let value_reg_a = pvm_ctx.reg[reg_a as usize] as u64;
    let reg_b_mod64 = pvm_ctx.reg[reg_b as usize] % 64;
    pvm_ctx.reg[reg_d as usize] = (value_reg_a >> reg_b_mod64) as RegSize;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn shar_r_64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    let value_reg_a = pvm_ctx.reg[reg_a as usize] as u64;
    let reg_b_mod64 = (pvm_ctx.reg[reg_b as usize] % 64) as u64;
    pvm_ctx.reg[reg_d as usize] = unsigned(signed(value_reg_a, 8) >> reg_b_mod64, 8) as RegSize;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn set_lt_u(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    if pvm_ctx.reg[reg_a as usize] < pvm_ctx.reg[reg_b as usize] {
        pvm_ctx.reg[reg_d as usize] = 1;
        pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
        return ExitReason::Continue;
    } 
    pvm_ctx.reg[reg_d as usize] = 0;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    return ExitReason::Continue;
}

pub fn set_lt_s(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    
    if signed(pvm_ctx.reg[reg_a], 8) < signed(pvm_ctx.reg[reg_b], 8) {
        pvm_ctx.reg[reg_d as usize] = 1;
        pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
        return ExitReason::Continue;
    } 

    pvm_ctx.reg[reg_d as usize] = 0;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    return ExitReason::Continue;
}

pub fn cmov_iz(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    if pvm_ctx.reg[reg_b as usize] == 0 {
        pvm_ctx.reg[reg_d as usize] = pvm_ctx.reg[reg_a as usize];
    }
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn cmov_nz(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    if pvm_ctx.reg[reg_b as usize] != 0 {
        pvm_ctx.reg[reg_d as usize] = pvm_ctx.reg[reg_a as usize];
    }
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn rot_l_64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    let value_reg_a = pvm_ctx.reg[reg_a as usize] as u128;
    let value_reg_b = pvm_ctx.reg[reg_b as usize];
    let mut result = 0 as u64;
    for i in 0..64 {
        let bit_a = (value_reg_a >> i) & 1;
        result |= (bit_a << (((i as u64).wrapping_add(value_reg_b)) % 64)) as u64;
    }
    pvm_ctx.reg[reg_d as usize] = result as RegSize;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn rot_l_32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, &program);
    let value_reg_a = pvm_ctx.reg[reg_a as usize] as u32;
    let value_reg_b = pvm_ctx.reg[reg_b as usize] as u32;
    let mut result = 0 as u32;
    for i in 0..32 {
        let bit_a = (value_reg_a >> i) & 1;
        result |= (bit_a << (((i as u32).wrapping_add(value_reg_b)) % 32)) as u32;
    }
    pvm_ctx.reg[reg_d as usize] = extend_sign(&result.to_le_bytes(), 4) as RegSize;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn rot_r_64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, &program);
    let value_reg_a = pvm_ctx.reg[reg_a as usize] as u64;
    let value_reg_b = pvm_ctx.reg[reg_b as usize];
    let mut result = 0 as u64;
    for i in 0..64 {
        let bit_a = (value_reg_a >> ((i as u64).wrapping_add(value_reg_b)) % 64) & 1;
        result |= (bit_a << i) as u64;
    }
    pvm_ctx.reg[reg_d as usize] = result as RegSize;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn rot_r_32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, &program);
    let value_reg_a = pvm_ctx.reg[reg_a as usize] as u32;
    let value_reg_b = pvm_ctx.reg[reg_b as usize] as u32;
    let mut result = 0 as u32;
    for i in 0..32 {
        let bit_a = (value_reg_a >> ((i as u32).wrapping_add(value_reg_b)) % 32) & 1;
        result |= (bit_a << i) as u32;
    }
    pvm_ctx.reg[reg_d as usize] = extend_sign(&result.to_le_bytes(), 4) as RegSize;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn and_inv(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    pvm_ctx.reg[reg_d as usize] = pvm_ctx.reg[reg_a as usize] & !pvm_ctx.reg[reg_b as usize];
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn or_inv(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    pvm_ctx.reg[reg_d as usize] = pvm_ctx.reg[reg_a as usize] | !pvm_ctx.reg[reg_b as usize];
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn xnor(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, &program);
    pvm_ctx.reg[reg_d as usize] = !(pvm_ctx.reg[reg_a as usize] ^ pvm_ctx.reg[reg_b as usize]);
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn max(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    let a = signed(pvm_ctx.reg[reg_a as usize], 8);
    let b = signed(pvm_ctx.reg[reg_b as usize], 8);
    pvm_ctx.reg[reg_d as usize] = std::cmp::max(a, b) as RegSize;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn max_u(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    pvm_ctx.reg[reg_d as usize] = std::cmp::max(pvm_ctx.reg[reg_a as usize], pvm_ctx.reg[reg_b as usize]);
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn min(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    let a = signed(pvm_ctx.reg[reg_a as usize], 8);
    let b = signed(pvm_ctx.reg[reg_b as usize], 8);
    pvm_ctx.reg[reg_d as usize] = std::cmp::min(a, b) as RegSize;
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn min_u(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    pvm_ctx.reg[reg_d as usize] = std::cmp::min(pvm_ctx.reg[reg_a as usize], pvm_ctx.reg[reg_b as usize]);
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

#[cfg(test)]
mod test {
    use super::*;

    #[cfg(test)]
    mod test {
        use super::*;
        
        #[test]
        fn test_sub_32() {
            let mut pvm_ctx = Context::default();
            pvm_ctx.reg[8] = 0xFFFFFFFFFFFFFFFF;
            pvm_ctx.reg[7] = 0x2;
            let mut program = Program::default();
            program.code = vec![191, 0x78, 0x9, 191, 0x78, 0x9];
            program.bitmask = vec![true, false, false, true, false, false, true];
            
            sub_32(&mut pvm_ctx, &program);
            assert_eq!(pvm_ctx.reg[9], 0xFFFFFFFFFFFFFFFD);

            //pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1; // Next instruction
            pvm_ctx.reg[8] = 0x1;
            pvm_ctx.reg[7] = 0x2;

            sub_32(&mut pvm_ctx, &program);
            assert_eq!(pvm_ctx.reg[9], 0xFFFFFFFFFFFFFFFF);
        } 

        #[test]
        fn test_sub_64() {
            let mut pvm_ctx = Context::default();
            pvm_ctx.reg[8] = 0xFFFFFFFFFFFFFFFF;
            pvm_ctx.reg[7] = 0x2;
            let mut program = Program::default();
            program.code = vec![201, 0x78, 0x9, 201, 0x78, 0x9];
            program.bitmask = vec![true, false, false, true, false, false, true];
            
            sub_64(&mut pvm_ctx, &program);
            assert_eq!(pvm_ctx.reg[9], 0xFFFFFFFFFFFFFFFD);


            //pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1; // Next instruction
            pvm_ctx.reg[8] = 0x2;
            pvm_ctx.reg[7] = 0x0;

            sub_64(&mut pvm_ctx, &program);
            assert_eq!(pvm_ctx.reg[9], 2);
        } 
    }  
}