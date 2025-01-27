/*
    Instructions with Arguments of Two Registers & One Offset.
*/

use std::cmp::{min, max};
use crate::pvm;
use crate::types::{Context, ExitReason, Program, RegSigned, RegSize};
use crate::pvm::isa::{skip, extend_sign, signed};
use crate::utils::codec::{BytesReader};
use crate::utils::codec::generic::{decode_integer};


fn get_reg(pc: &RegSize, program: &Program) -> (usize, usize) {
    let reg_a = min(12, program.code[*pc as usize + 1] % 16) as usize;
    let reg_b = min(12, program.code[*pc as usize + 1] >> 4) as usize;
    (reg_a, reg_b)
}

fn get_x_length(pc: &RegSize, program: &Program) -> RegSize {
    min(4, max(0, skip(pc, &program.bitmask) - 1))
}

fn get_value(pc: &RegSize, program: &Program) -> i64 {
    let start = (*pc + 2) as usize;
    let end = start + get_x_length(pc, program) as usize;
    let mut reader = BytesReader::new(&program.code[start..end]);
    let value = decode_integer(&mut reader, get_x_length(pc, program) as usize).unwrap() as u64;
    signed(value, get_x_length(pc, program) as usize) + *pc as i64
}

fn branch_common(
    pvm_ctx: &mut Context,
    program: &Program,
    compare: impl Fn(RegSize, RegSize) -> bool,
) -> ExitReason {
    let (reg_a, reg_b) = get_reg(&pvm_ctx.pc, program);
    let value = get_value(&pvm_ctx.pc, program);
    _branch(pvm_ctx, program, value, compare)
}

fn _branch(pvm_ctx: &mut Context, program: &Program, n: i64, compare: impl Fn(RegSize, RegSize) -> bool) -> ExitReason {
    let (reg_a, reg_b) = get_reg(&pvm_ctx.pc, program);
    let value = get_value(&pvm_ctx.pc, program);
    if compare(pvm_ctx.reg[reg_a], pvm_ctx.reg[reg_b]) {
        pvm_ctx.pc = n as RegSize - 1;
    }
    return ExitReason::Continue;
}

pub fn branch_eq(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    branch_common(pvm_ctx, program, |a, b| a as RegSize == b as RegSize)
}

pub fn branch_ne(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    branch_common(pvm_ctx, program, |a, b| a as RegSize != b as RegSize)
}

pub fn branch_ge_s(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    branch_common(pvm_ctx, program, |a, b| (a as RegSigned) >= (b as RegSigned))
}

pub fn branch_lt_u(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    branch_common(pvm_ctx, program, |a, b| (a as RegSize) < (b as RegSize))
}

pub fn branch_lt_s(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    branch_common(pvm_ctx, program, |a, b| (a as RegSigned) < (b as RegSigned))
}

pub fn branch_ge_u(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    branch_common(pvm_ctx, program, |a, b| a as RegSize >= b as RegSize)
}
