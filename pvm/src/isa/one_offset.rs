/*
    Instructions with Arguments of One Offset.
*/

use std::cmp::min;
use crate::{Context, ExitReason, Program};
use codec::generic_codec::decode_integer;
use codec::BytesReader;
use crate::isa::{skip, _branch, signed};

fn get_lx_length(pc: &u64, bitmask: &[bool]) -> u64 {
    min(4, skip(pc, bitmask)) as u64
}

fn get_lx_imm(pc: &u64, program: &Program) -> i64 {
    let start = *pc as usize + 1;
    let lx = get_lx_length(pc, &program.bitmask) as usize;
    let end = start + lx;
    let mut reader = BytesReader::new(&program.code[start..end]);
    let value = decode_integer(&mut reader, lx).unwrap() as u64;
    signed(value, lx) + *pc as i64
}

pub fn jump(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let value_x = get_lx_imm(&pvm_ctx.pc, program);
    _branch(pvm_ctx, program, value_x)
}
