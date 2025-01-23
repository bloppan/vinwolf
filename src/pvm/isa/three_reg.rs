/*
    Instructions with Arguments of Three Registers.
*/

use std::cmp::{min, max};
use crate::types::{Context, ExitReason, Program, RegSigned, RegSize};
use crate::pvm::isa::{skip, extend_sign};

fn get_reg(pc: &u64, program: &Program) -> (usize, usize, usize) {
    let reg_a = min(12, program.code[*pc as usize + 1] % 16) as usize;
    let reg_b = min(12, program.code[*pc as usize + 1] >> 4) as usize;
    let reg_d = min(12, program.code[*pc as usize + 2]) as usize;
    (reg_a, reg_b, reg_d)
}

pub fn add_32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    let result = (pvm_ctx.reg[reg_a as usize] as u32).wrapping_add(pvm_ctx.reg[reg_b as usize] as u32).to_le_bytes();
    pvm_ctx.reg[reg_d as usize] = extend_sign(&result);
    ExitReason::Continue
}

pub fn sub_32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    let result = (pvm_ctx.reg[reg_a as usize] as u32).wrapping_sub(pvm_ctx.reg[reg_b as usize] as u32).to_le_bytes();  
    pvm_ctx.reg[reg_d as usize] = extend_sign(&result);
    ExitReason::Continue
}

pub fn mul_32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    let result = (pvm_ctx.reg[reg_a as usize] as u32).wrapping_mul(pvm_ctx.reg[reg_b as usize] as u32).to_le_bytes();
    pvm_ctx.reg[reg_d as usize] = extend_sign(&result);
    ExitReason::Continue
}

pub fn div_u_32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    if pvm_ctx.reg[reg_b as usize] == 0 {
        pvm_ctx.reg[reg_d as usize] = 0xFFFFFFFFFFFFFFFFu64;
    } else {
        pvm_ctx.reg[reg_d as usize] = (pvm_ctx.reg[reg_a as usize] as u32 / pvm_ctx.reg[reg_b as usize] as u32) as RegSize;
    }
    ExitReason::Continue
}

pub fn div_s_32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    if pvm_ctx.reg[reg_b as usize] == 0 {
        pvm_ctx.reg[reg_d as usize] = 0xFFFFFFFFFFFFFFFFu64;
    } else if pvm_ctx.reg[reg_a as usize] as u32 == 0x80000000u32 || pvm_ctx.reg[reg_b as usize] as u32 == 0xFFFFFFFFu32 {
        pvm_ctx.reg[reg_d as usize] = (pvm_ctx.reg[reg_a as usize] as i32) as RegSize;
    } else {
        pvm_ctx.reg[reg_d as usize] = (pvm_ctx.reg[reg_a as usize] as i32 / pvm_ctx.reg[reg_b as usize] as i32) as RegSize;
    }
    ExitReason::Continue
}

pub fn rem_s_32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    let a = pvm_ctx.reg[reg_a as usize] as i32;
    let b = pvm_ctx.reg[reg_b as usize] as i32;
    if b == 0 {
        pvm_ctx.reg[reg_d as usize] = (a as i64) as RegSize;
    } else if a as i64 == 0xFFFFFFFF80000000 || b == 0xFFFFFFFF {
        pvm_ctx.reg[reg_d as usize] = 0;
    } else {
        pvm_ctx.reg[reg_d as usize] = (a % b) as RegSize;
    }

    ExitReason::Continue
}


pub fn add_64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    pvm_ctx.reg[reg_d as usize] = pvm_ctx.reg[reg_a as usize].wrapping_add(pvm_ctx.reg[reg_b as usize]);
    ExitReason::Continue
}

pub fn sub_64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    pvm_ctx.reg[reg_d as usize] = pvm_ctx.reg[reg_a as usize].wrapping_sub(pvm_ctx.reg[reg_b as usize]);
    ExitReason::Continue
}

pub fn and(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    pvm_ctx.reg[reg_d as usize] = pvm_ctx.reg[reg_a as usize] & pvm_ctx.reg[reg_b as usize];
    ExitReason::Continue
}

pub fn xor(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    pvm_ctx.reg[reg_d as usize] = pvm_ctx.reg[reg_a as usize] ^ pvm_ctx.reg[reg_b as usize];
    ExitReason::Continue
}

pub fn or(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    pvm_ctx.reg[reg_d as usize] = pvm_ctx.reg[reg_a as usize] | pvm_ctx.reg[reg_b as usize];
    ExitReason::Continue
}

pub fn mul_64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    pvm_ctx.reg[reg_d as usize] = pvm_ctx.reg[reg_a as usize].wrapping_mul(pvm_ctx.reg[reg_b as usize]);
    ExitReason::Continue
}

pub fn div_u_64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    if pvm_ctx.reg[reg_b as usize] == 0 {
        pvm_ctx.reg[reg_d as usize] = 0xFFFFFFFFFFFFFFFFu64;
    } else {
        pvm_ctx.reg[reg_d as usize] = pvm_ctx.reg[reg_a as usize] / pvm_ctx.reg[reg_b as usize];
    }
    ExitReason::Continue
}

pub fn div_s_64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    if pvm_ctx.reg[reg_b as usize] == 0 {
        pvm_ctx.reg[reg_d as usize] = 0xFFFFFFFFFFFFFFFFu64;
    } else if pvm_ctx.reg[reg_a as usize] == 0x8000000000000000u64 || pvm_ctx.reg[reg_b as usize] == 0xFFFFFFFFFFFFFFFFu64 {
        pvm_ctx.reg[reg_d as usize] = (pvm_ctx.reg[reg_a as usize] as i64) as RegSize;
    } else {
        pvm_ctx.reg[reg_d as usize] = (pvm_ctx.reg[reg_a as usize] as i64 / pvm_ctx.reg[reg_b as usize] as i64) as RegSize;
    }
    ExitReason::Continue
}

pub fn cmov_iz(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    if pvm_ctx.reg[reg_b as usize] == 0 {
        pvm_ctx.reg[reg_d as usize] = pvm_ctx.reg[reg_a as usize];
    }
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
            program.bitmask = vec![true, false, false, true, false, false];
            
            sub_32(&mut pvm_ctx, &program);
            assert_eq!(pvm_ctx.reg[9], 0xFFFFFFFFFFFFFFFD);

            pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1; // Next instruction
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
            program.bitmask = vec![true, false, false, true, false, false];
            
            sub_64(&mut pvm_ctx, &program);
            assert_eq!(pvm_ctx.reg[9], 0xFFFFFFFFFFFFFFFD);


            pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1; // Next instruction
            pvm_ctx.reg[8] = 0x2;
            pvm_ctx.reg[7] = 0x0;

            sub_64(&mut pvm_ctx, &program);
            assert_eq!(pvm_ctx.reg[9], 2);
        } 
    }  
}