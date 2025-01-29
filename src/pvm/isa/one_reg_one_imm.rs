/*
    Instructions with Arguments of One Register & One Immediate.
*/

use std::cmp::{min, max};

use crate::constants::PAGE_SIZE;
use crate::types::{Context, ExitReason, Program, RamAccess, RamAddress, RegSize};
use crate::utils::codec::generic::decode;
use crate::pvm::isa::{skip, extend_sign, check_page_fault, _load, _store, djump};

fn get_reg(pc: &RegSize, program: &Program) -> RegSize {
    min(12, program.code[*pc as usize + 1] % 16) as RegSize
}

fn get_x_length(pc: &RegSize, program: &Program) -> RegSize {
    min(4, max(0, skip(pc, &program.bitmask) - 1))
}

fn get_x_imm(pc: &RegSize, program: &Program) -> RegSize {
    let start = *pc as usize + 2;
    let end = start + get_x_length(pc, program) as usize;
    let n = get_x_length(pc, program) as usize;
    extend_sign(&program.code[start..end], n) as RegSize
}

pub fn jump_ind(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let reg_a = get_reg(&pvm_ctx.pc, program);
    let value_imm = get_x_imm(&pvm_ctx.pc, program);
    let value_reg_a = pvm_ctx.reg[reg_a as usize];
    let a = (value_reg_a.wrapping_add(value_imm) % (1 << 32)) as RegSize;
    println!("jump_ind: a = {a}");
    println!("jump_ind: pc = {}", pvm_ctx.pc);
    djump(&a, &mut pvm_ctx.pc, program)
}

pub fn load_imm(pvm_ctx: &mut Context, program: &Program)-> ExitReason {
    let reg_a = get_reg(&pvm_ctx.pc, program);
    pvm_ctx.reg[reg_a as usize] = get_x_imm(&pvm_ctx.pc, program);
    ExitReason::Continue
}

pub fn load<T>(pvm_ctx: &mut Context, program: &Program, signed: bool) -> ExitReason {
    let to_reg = get_reg(&pvm_ctx.pc, program) as RegSize;
    let address = get_x_imm(&pvm_ctx.pc, program) as RamAddress;
    _load::<T>(pvm_ctx, address as RamAddress, to_reg, signed)
}

pub fn load_u8(pvm_ctx: &mut Context, program: &Program)-> ExitReason {
    load::<u8>(pvm_ctx, program, false)
}

pub fn load_u16(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    load::<u16>(pvm_ctx, program, false)
}

pub fn load_u32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    load::<u32>(pvm_ctx, program, false)
}

pub fn load_u64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    load::<u64>(pvm_ctx, program, false)
}

pub fn load_i8(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    load::<i8>(pvm_ctx, program, true)
}

pub fn load_i16(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    load::<i16>(pvm_ctx, program, true)
}

pub fn load_i32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    load::<i32>(pvm_ctx, program, true)
}

pub fn store<T>(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let reg_a = get_reg(&pvm_ctx.pc, program);
    let address = get_x_imm(&pvm_ctx.pc, program) as RamAddress;
    let value = ((pvm_ctx.reg[reg_a as usize] as u128) % (1 << (std::mem::size_of::<T>() * 8))) as RegSize;
    _store::<T>(pvm_ctx, program, address, value)
}


pub fn store_u8(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    store::<u8>(pvm_ctx, program)
}

pub fn store_u16(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    store::<u16>(pvm_ctx, program)
}

pub fn store_u32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    store::<u32>(pvm_ctx, program)
}

pub fn store_u64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    store::<u64>(pvm_ctx, program)
}

