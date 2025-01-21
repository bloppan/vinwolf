/*
    Instructions with Arguments of Three Registers.
*/

use std::cmp::{min, max};
use crate::constants::NUM_REG;
use crate::pvm;
use crate::types::{Context, ExitReason};
use crate::utils::codec::{DecodeSize, BytesReader};
use crate::pvm::isa::{skip, extend_sign};

/*fn get_reg(pc: &u64, memory: &MemoryMap) -> (u8, u8, u8) {
    let reg_a: u8 = min(12, memory.program.code[*pc as usize + 1] % 16);
    let reg_b: u8 = min(12, memory.program.code[*pc as usize + 1] >> 4);
    let reg_d: u8 = min(12, memory.program.code[*pc as usize + 2]);
    (reg_a, reg_b, reg_d)
}

pub fn add_32(pvm_ctx: &mut Context, memory: &MemoryMap) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    let result = (pvm_ctx.reg[reg_a as usize].wrapping_add(pvm_ctx.reg[reg_b as usize]) % (1 << 32)).to_le_bytes();
    pvm_ctx.reg[reg_d as usize] = extend_sign(&result, 4);
    pvm_ctx.pc = skip(&pvm_ctx.pc, &program.bitmask);
    ExitReason::Continue
}

pub fn sub_32(pvm_ctx: &mut Context, memory: &MemoryMap) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    let result = (pvm_ctx.reg[reg_a as usize]
                                                .wrapping_add(1 << 32)
                                                .wrapping_sub(pvm_ctx.reg[reg_b as usize] as u64 % (1 << 32)) % (1 << 32) )
                                                .to_le_bytes();
    pvm_ctx.reg[reg_d as usize] = extend_sign(&result, 4);
    pvm_ctx.pc = skip(&pvm_ctx.pc, &program.bitmask);
    ExitReason::Continue
}

pub fn sub_64(pvm_ctx: &mut Context, memory: &MemoryMap) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    pvm_ctx.reg[reg_d as usize] = pvm_ctx.reg[reg_a as usize].wrapping_sub(pvm_ctx.reg[reg_b as usize]);
    pvm_ctx.pc = skip(&pvm_ctx.pc, &program.bitmask);
    ExitReason::Continue
}

pub fn and(pvm_ctx: &mut Context, memory: &MemoryMap) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    pvm_ctx.reg[reg_d as usize] = pvm_ctx.reg[reg_a as usize] & pvm_ctx.reg[reg_b as usize];
    pvm_ctx.pc = skip(&pvm_ctx.pc, &program.bitmask);
    ExitReason::Continue
}

pub fn xor(pvm_ctx: &mut Context, memory: &MemoryMap) -> ExitReason {
    let (reg_a, reg_b, reg_d) = get_reg(&pvm_ctx.pc, program);
    pvm_ctx.reg[reg_d as usize] = pvm_ctx.reg[reg_a as usize] ^ pvm_ctx.reg[reg_b as usize];
    pvm_ctx.pc = skip(&pvm_ctx.pc, &program.bitmask);
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
            program.data = vec![191, 0x78, 0x9, 191, 0x78, 0x9];
            program.bitmask = vec![true, false, false, true, false, false];
            
            sub_32(&mut pvm_ctx, &program);
            assert_eq!(pvm_ctx.reg[9], 0xFFFFFFFFFFFFFFFD);

            pvm_ctx.pc += 1; // Next instruction
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
            program.data = vec![201, 0x78, 0x9, 201, 0x78, 0x9];
            program.bitmask = vec![true, false, false, true, false, false];
            
            sub_64(&mut pvm_ctx, &program);
            assert_eq!(pvm_ctx.reg[9], 0xFFFFFFFFFFFFFFFD);

            pvm_ctx.pc += 1; // Next instruction
            pvm_ctx.reg[8] = 0x2;
            pvm_ctx.reg[7] = 0x0;

            sub_64(&mut pvm_ctx, &program);
            assert_eq!(pvm_ctx.reg[9], 2);
            //assert_eq!(pvm_ctx.reg[9], 0xFFFFFFFFFFFFFFFF);
        } 
    }  
}*/