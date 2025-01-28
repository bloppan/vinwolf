/*
    Instructions with Arguments of One Register, One Immediate and One Offset.
*/

use std::cmp::{min, max};
use crate::types::{Context, ExitReason, Program, RamAddress, RegSigned, RegSize};
use crate::pvm::isa::{skip, extend_sign, signed};


fn get_reg(pc: &RegSize, program: &Program) -> u8 {
    min(12, program.code[*pc as usize + 1] % 16)
}

fn get_x_length(pc: &RegSize, program: &Program) -> RegSize {
    (min(4, program.code[*pc as usize + 1] >> 4) % 8) as RegSize
}

fn get_y_length(pc: &RegSize, program: &Program) -> RegSize {
    min(4, max(0, skip(pc, &program.bitmask).saturating_sub(1).saturating_sub(get_x_length(pc, program)))) as RegSize
}

fn get_x_value(pc: &RegSize, program: &Program) -> RegSize {
    let start = *pc as usize + 2;
    let end = start + get_x_length(pc, program) as usize;
    let n = get_x_length(pc, program) as usize;
    extend_sign(&program.code[start..end], n) as RegSize
}

fn get_y_value(pc: &RegSize, program: &Program) -> RegSize {
    let start = *pc as usize + 2 + get_x_length(pc, program) as usize;
    let end = start + get_y_length(pc, program) as usize;
    let n = get_y_length(pc, program) as usize;
    extend_sign(&program.code[start..end], n) as RegSize
}

fn branch_common_imm(
    pvm_ctx: &mut Context,
    program: &Program,
    compare: impl Fn(u64, u64) -> bool,
) -> ExitReason {
    let reg = get_reg(&pvm_ctx.pc, program);
    let x_value = get_x_value(&pvm_ctx.pc, program);
    let y_value = get_y_value(&pvm_ctx.pc, program);

    if compare(pvm_ctx.reg[reg as usize], x_value) {
        pvm_ctx.pc += (y_value as RegSize).saturating_sub(1);
    }
    ExitReason::Continue
}

pub fn branch_eq_imm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    branch_common_imm(pvm_ctx, program, |a, b| a as RegSize == b as RegSize)
}

pub fn branch_ne_imm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    branch_common_imm(pvm_ctx, program, |a, b| a as RegSize != b as RegSize)
}

pub fn branch_lt_u_imm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    branch_common_imm(pvm_ctx, program, |a, b| (a as RegSize) < (b as RegSize))
}

pub fn branch_le_u_imm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    branch_common_imm(pvm_ctx, program, |a, b| a as RegSize <= b as RegSize)
}

pub fn branch_ge_u_imm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    branch_common_imm(pvm_ctx, program, |a, b| a as RegSize >= b as RegSize) 
}

pub fn branch_gt_u_imm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    branch_common_imm(pvm_ctx, program, |a, b| a as RegSize > b as RegSize)
}

pub fn branch_lt_s_imm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    branch_common_imm(pvm_ctx, program, |a, b| signed(a, 8) < signed(b, 8))
}

pub fn branch_le_s_imm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    branch_common_imm(pvm_ctx, program, |a, b| signed(a, 8) <= signed(b, 8))
}

pub fn branch_ge_s_imm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    branch_common_imm(pvm_ctx, program, |a, b| signed(a, 8) >= signed(b, 8))
}

pub fn branch_gt_s_imm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    branch_common_imm(pvm_ctx, program, |a, b| signed(a, 8) > signed(b, 8))
}

