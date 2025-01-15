/*
    Instructions with Arguments of Two Registers.
*/

use crate::constants::NUM_REG;
use crate::types::{Context, ProgramSequence, ExitReason};
use crate::pvm::isa::skip;


pub fn move_reg(pvm_ctx: &mut Context, program: &ProgramSequence) // Two regs -> 64 12 -> r1 = r2
    -> Result<(), ExitReason> {
    let dest: u8 = program.data[pvm_ctx.pc as usize + 1] >> 4;
    if dest > NUM_REG { return Err(ExitReason::Panic) };
    let a: u8 = program.data[pvm_ctx.pc as usize + 1] & 0x0F;
    if a > NUM_REG { return Err(ExitReason::Panic) };
    /*println!("dest = {dest}, a = {a}");
    println!("pvm_ctx.pc = {}", pvm_ctx.pc);
    println!("program.data[pvm_ctx.pc + 1] = {}", program.data[pvm_ctx.pc + 1]);*/
    pvm_ctx.reg[dest as usize] = pvm_ctx.reg[a as usize];
    pvm_ctx.pc = skip(&pvm_ctx.pc, &program.bitmask);
    Ok(())
}


