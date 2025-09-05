/*
    Instructions with Arguments of One Offset.
*/
use crate::{RegSize, Gas, RamMemory, Registers, Program, ExitReason};
use crate::isa::skip;

pub fn trap(_program: &Program, _pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, _reg: &mut Registers) -> ExitReason {
    ExitReason::panic
}

pub fn fallthrough(program: &Program, pc: &mut RegSize, _gas: &mut Gas, _ram: &mut RamMemory, _reg: &mut Registers) -> ExitReason {
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}