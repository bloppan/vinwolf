/*
    Instructions with Arguments of One Register, One Immediate and One Offset.
*/

use std::cmp::{min, max};
use crate::jam_types::{Context, ExitReason, Program, RegSigned, RegSize};
use crate::pvm::isa::{skip, extend_sign, signed, _branch};
use crate::utils::codec::BytesReader;
use crate::utils::codec::generic::decode_integer;

fn get_reg(pc: &RegSize, program: &Program) -> u8 {
    min(12, program.code[*pc as usize + 1] % 16)
}

fn get_x_length(pc: &RegSize, program: &Program) -> RegSize {
    (min(4, program.code[*pc as usize + 1] >> 4) % 8) as RegSize
}

fn get_y_length(pc: &RegSize, program: &Program) -> RegSize {
    min(4, max(0, skip(pc, &program.bitmask) - 1 - get_x_length(pc, program))) as RegSize
}

fn get_x_value(pc: &RegSize, program: &Program) -> RegSize {
    let start = *pc as usize + 2;
    let end = start + get_x_length(pc, program) as usize;
    extend_sign(&program.code[start..end], get_x_length(pc, program) as usize) as RegSize
}

fn get_y_value(pc: &RegSize, program: &Program) -> i64 {
    let ly = get_y_length(pc, program) as usize;
    let start = *pc as usize + 2 + get_x_length(pc, program) as usize;
    let end = start + ly;
    let mut reader = BytesReader::new(&program.code[start..end]);
    let value = decode_integer(&mut reader, ly).unwrap() as u64;
    signed(value, ly) + *pc as i64
}

fn branch(
    pvm_ctx: &mut Context,
    program: &Program,
    compare: impl Fn(u64, u64) -> bool,
) -> ExitReason {
    let reg = get_reg(&pvm_ctx.pc, program);
    let l_value = pvm_ctx.reg[reg as usize];
    let r_value = get_x_value(&pvm_ctx.pc, program);
    let n = get_y_value(&pvm_ctx.pc, program);
    if !compare(l_value, r_value) {
        pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
        return ExitReason::Continue;
    }
    _branch(pvm_ctx, program, n as RegSigned)
}

pub fn load_imm_jump(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let reg_a = get_reg(&pvm_ctx.pc, program);
    let vx = get_x_value(&pvm_ctx.pc, program);
    let vy = get_y_value(&pvm_ctx.pc, program);
    let exit_reason = _branch(pvm_ctx, program, vy);
    pvm_ctx.reg[reg_a as usize] = vx as RegSize;
    return exit_reason;
}

pub fn branch_eq_imm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    branch(pvm_ctx, program, |a, b| a as RegSize == b as RegSize)
}

pub fn branch_ne_imm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    branch(pvm_ctx, program, |a, b| a as RegSize != b as RegSize)
}

pub fn branch_lt_u_imm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    branch(pvm_ctx, program, |a, b| (a as RegSize) < (b as RegSize))
}

pub fn branch_le_u_imm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    branch(pvm_ctx, program, |a, b| a as RegSize <= b as RegSize)
}

pub fn branch_ge_u_imm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    branch(pvm_ctx, program, |a, b| a as RegSize >= b as RegSize) 
}

pub fn branch_gt_u_imm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    branch(pvm_ctx, program, |a, b| a as RegSize > b as RegSize)
}

pub fn branch_lt_s_imm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    branch(pvm_ctx, program, |a, b| signed(a, 8) < signed(b, 8))
}

pub fn branch_le_s_imm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    branch(pvm_ctx, program, |a, b| signed(a, 8) <= signed(b, 8))
}

pub fn branch_ge_s_imm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    branch(pvm_ctx, program, |a, b| signed(a, 8) >= signed(b, 8))
}

pub fn branch_gt_s_imm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    branch(pvm_ctx, program, |a, b| signed(a, 8) > signed(b, 8))
}

