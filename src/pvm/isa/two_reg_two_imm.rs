/*
    Instruction with Arguments of Two Registers and Two Immediates
*/

use std::cmp::{min, max};

use crate::types::{Context, ExitReason, Program, RegSize};
use crate::pvm::isa::{skip, extend_sign, signed, djump};
use crate::utils::codec::{BytesReader};
use crate::utils::codec::generic::{decode_integer};


fn get_reg(pc: &RegSize, program: &Program) -> (usize, usize) {
    let reg_a = min(12, program.code[*pc as usize + 1] % 16) as usize;
    let reg_b = min(12, program.code[*pc as usize + 1] >> 4) as usize;
    (reg_a, reg_b)
}

fn get_x_length(pc: &RegSize, program: &Program) -> RegSize {
    min(4, program.code[*pc as usize + 2] % 8) as RegSize
}

fn get_y_length(pc: &RegSize, program: &Program) -> RegSize {
    let lx = get_x_length(pc, program);
    min(4, max(0, skip(pc, &program.bitmask) - lx - 2)) as RegSize
}

fn get_x_value(pc: &RegSize, program: &Program) -> u64 {
    let start = (*pc + 3) as usize;
    let end = start + get_x_length(pc, program) as usize;
    extend_sign(&program.code[start..end], get_x_length(pc, program) as usize)
}

fn get_y_value(pc: &RegSize, program: &Program) -> u64 {
    let start = (*pc + 3 + get_x_length(pc, program)) as usize;
    let end = start + get_y_length(pc, program) as usize;
    extend_sign(&program.code[start..end], get_y_length(pc, program) as usize)
}


pub fn load_imm_jump_ind(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let (reg_a, reg_b) = get_reg(&pvm_ctx.pc, program);
    let vx = get_x_value(&pvm_ctx.pc, program);
    let vy = get_y_value(&pvm_ctx.pc, program);
    let n = pvm_ctx.reg[reg_b].wrapping_add(vy) % (1 << 32);
    let exit_reason = djump(&n, &mut pvm_ctx.pc, program);
    pvm_ctx.reg[reg_a] = vx as RegSize;
    return exit_reason;
}

