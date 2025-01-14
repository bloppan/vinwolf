/*
    Instructions with Arguments of Two Registers.
*/

use crate::constants::NUM_REG;
use crate::types::{Context, ProgramSequence, ExitReason};
use crate::pvm::skip;


pub fn move_reg(pvm_ctx: &mut Context, program: &ProgramSequence) // Two regs -> 64 12 -> r1 = r2
    -> Result<(), ExitReason> {
    let dest: u8 = program.data[pvm_ctx.pc + 1] >> 4;
    if dest > NUM_REG { return Err(ExitReason::Panic) };
    let a: u8 = program.data[pvm_ctx.pc + 1] & 0x0F;
    pvm_ctx.reg[dest as usize] = pvm_ctx.reg[a as usize];
    skip(&mut pvm_ctx.pc, &program.bitmask);
    Ok(())
}


