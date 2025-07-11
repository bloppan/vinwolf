/*
    Instructions with Arguments of One Immediate.
*/

use std::cmp::{min, max};
use crate::jam_types::{Context, ExitReason, Program, RegSize, HostCallFn};
use crate::pvm::isa::{skip, extend_sign};


fn get_imm(pc: &RegSize, program: &Program) -> RegSize {
    let start= (*pc + 1) as usize;
    let end = start + get_x_length(pc, program) as usize;
    extend_sign(&program.code[start..end], get_x_length(pc, program) as usize) as RegSize
 }
 
 fn get_x_length(pc: &RegSize, program: &Program) -> RegSize {
     min(4, max(0, skip(pc, &program.bitmask)))
 }
 
 pub fn ecalli(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
     let value_imm = get_imm(&pvm_ctx.pc, program) as u8;
     pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
     let hostcall_fn = HostCallFn::try_from(value_imm).unwrap_or(HostCallFn::Unknown);
     ExitReason::HostCall(hostcall_fn)
 }