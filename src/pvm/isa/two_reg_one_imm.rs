/*
    Instructions with Arguments of Two Registers & One Immediate
*/

use std::cmp::{min, max};
use crate::constants::NUM_REG;
use crate::types::{Context, ProgramSequence, ExitReason};
use crate::utils::codec::{DecodeSize, BytesReader};
use crate::pvm::isa::{skip, extend_sign};

fn get_imm(pc: &u64, program: &ProgramSequence) -> u64 {
   let i = *pc + 2;
   let l_x = min(4, max(0, skip(&i, &program.bitmask).saturating_sub(1))) as usize;
   let number = program.data[i as usize..i as usize + l_x as usize].to_vec();
   return extend_sign(&number, l_x);
}

pub fn add_imm_32(pvm_ctx: &mut Context, program: &ProgramSequence) // Two regs one imm -> 02 79 02 | r9 = r7 + 0x2
    -> Result<(), ExitReason> {
    let dest: u8 = program.data[pvm_ctx.pc as usize + 1] & 0x0F;
    if dest > NUM_REG { return Err(ExitReason::Panic) };
    let vx = get_imm(&pvm_ctx.pc, program);
    //println!("value = {value}");
    let b: u8 = program.data[pvm_ctx.pc as usize + 1] >> 4;
    if b > NUM_REG { return Err(ExitReason::Panic) };
    pvm_ctx.reg[dest as usize] = pvm_ctx.reg[b as usize].wrapping_add(vx);
    pvm_ctx.pc = skip(&pvm_ctx.pc, &program.bitmask);
    Ok(())
}

/*fn get_imm(program: &ProgramSequence, pc: u32, instr_type: usize) -> u32 {
    let mut i = pc; 
    let l_x = match instr_type {
        ONE_REG_ONE_IMM | 
        TWO_REG_ONE_IMM => { 
                            i += 2; 
                            //println!("TWO_REG_ONE_IMM");
                            let x: isize = skip(pc, &program.k).saturating_sub(1) as isize;
                            let x_u32 = if x < 0 { 0 } else { x as u32 }; 
                            std::cmp::min(4_u32, x_u32)
                        },
        ONE_REG_ONE_IMM_ONE_OFFSET => {
                            i += 2;
                            //println!("ONE_REG_ONE_IMM_ONE_OFFSET");
                            std::cmp::min(4_u32, (program.c[pc as usize + 1] / 16) as u32)
        },
        _ => return 0,
    };
    //println!("lx = {l_x}");
    return extend_sign(&program.c[i as usize ..i as usize + l_x as usize].to_vec(), (4 - l_x as usize) as u32);
}



*/

/*fn add(pvm_ctx: &mut PVM, program: &ProgramSequence) // Three regs -> 08 87 09 | r9 = r7 + r8
    -> Result<(), String> {
    let dest: u8 = program.c[pvm_ctx.pc as usize + 2] & 0x0F;
    if dest > 13 { return Err("panic".to_string()) }; 
    let a: u8 = program.c[pvm_ctx.pc as usize + 1] & 0x0F;
    let b: u8 = program.c[pvm_ctx.pc as usize + 1] >> 4;
    pvm_ctx.reg[dest as usize] = pvm_ctx.reg[a as usize].wrapping_add(pvm_ctx.reg[b as usize]);
    pvm_ctx.pc = skip(pvm_ctx.pc, &program.k);
    Ok(())
}

fn and(pvm_ctx: &mut PVM, program: &ProgramSequence) // Three regs -> 17 87 09 | r9 = r7 & r8
    -> Result<(), String> {
    let dest: u8 = program.c[pvm_ctx.pc as usize + 2] & 0x0F;
    if dest > 13 { return Err("panic".to_string()) };
    let a: u8 = program.c[pvm_ctx.pc as usize + 1] & 0x0F;
    let b: u8 = program.c[pvm_ctx.pc as usize + 1] >> 4;
    pvm_ctx.reg[dest as usize] = pvm_ctx.reg[a as usize] & pvm_ctx.reg[b as usize];
    pvm_ctx.pc = skip(pvm_ctx.pc, &program.k);
    Ok(())
}

fn and_imm(pvm_ctx: &mut PVM, program: &ProgramSequence) // Two regs one imm -> 12 79 03 | r9 = r7 & 0x3
    -> Result<(), String> {
    let dest: u8 = program.c[pvm_ctx.pc as usize + 1] & 0x0F;
    if dest > 13 { return Err("panic".to_string()) };
    let b: u8 = program.c[pvm_ctx.pc as usize + 1] >> 4;
    let value = get_imm(program, pvm_ctx.pc, TWO_REG_ONE_IMM);
    pvm_ctx.reg[dest as usize] = pvm_ctx.reg[b as usize] & value;
    pvm_ctx.pc = skip(pvm_ctx.pc, &program.k);
    Ok(())
}
    */