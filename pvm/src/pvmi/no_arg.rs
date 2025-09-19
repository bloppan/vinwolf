/*
    Instructions with Arguments of One Offset.
*/
use crate::pvm_types::{RegSize, Gas, RamMemory, Registers, Program, ExitReason};
use crate::pvmi::skip;

#[inline(always)]
pub fn trap(_program: &Program, _pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, _reg: &mut Registers) -> ExitReason {
    ExitReason::Panic
}

#[inline(always)]
pub fn fallthrough(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, _reg: &mut Registers) -> ExitReason {
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}