/*
    Instructions with Arguments of One Offset.
*/
use crate::jam_types::{Context, Program, ExitReason};
use crate::pvm::isa::skip;

pub fn trap() -> ExitReason {
    ExitReason::panic
}

pub fn fallthrough(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}