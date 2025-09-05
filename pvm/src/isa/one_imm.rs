/*
    Instructions with Arguments of One Immediate.
*/

use std::cmp::{min, max};
use crate::pvm_types::{Gas, RamMemory, Registers, ExitReason, Program, RegSize, HostCallFn};
use crate::isa::{skip, extend_sign};


fn get_imm(pc: &RegSize, program: &Program) -> RegSize {
    let start= (*pc + 1) as usize;
    let end = start + get_x_length(pc, program) as usize;
    extend_sign(&program.code[start..end], get_x_length(pc, program) as usize) as RegSize
 }
 
 fn get_x_length(pc: &RegSize, program: &Program) -> RegSize {
     min(4, max(0, skip(pc, &program.bitmask)))
 }
 
 pub fn ecalli(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, _reg: &mut Registers) -> ExitReason {
     let value_imm = get_imm(&pc, program) as u8;
     *pc += skip(&pc, &program.bitmask) + 1;
     let hostcall_fn = HostCallFn::try_from(value_imm).unwrap_or(HostCallFn::Unknown);
     ExitReason::HostCall(hostcall_fn)
 }