/*
    Instructions with Arguments of One Register & One Immediate.
*/

use std::cmp::{min, max};
use frame_support::sp_runtime::traits::CheckedConversion;

use crate::constants::{NUM_REG, PAGE_SIZE};
use crate::pvm;
use crate::types::{Context, ExitReason, MemoryChunk, Program, RamAccess, RamAddress, RegSize};
use crate::utils::codec::{EncodeSize, DecodeSize, BytesReader};
use crate::pvm::isa::{skip, extend_sign, check_page_fault, _store};

fn get_reg(pc: &RegSize, program: &Program) -> RegSize {
    min(12, program.code[*pc as usize + 1] % 16) as RegSize
}

fn get_x_length(pc: &RegSize, program: &Program) -> RegSize {
    min(4, max(0, skip(pc, &program.bitmask) - 1))
}

fn get_x_imm(pc: &RegSize, program: &Program) -> RegSize {
    let start = *pc as usize + 2;
    let end = start + get_x_length(pc, program) as usize;
    extend_sign(&program.code[start..end], get_x_length(pc, program) as usize)
}

pub fn load_imm(pvm_ctx: &mut Context, program: &Program)-> ExitReason {
    let reg_a = get_reg(&pvm_ctx.pc, program);
    pvm_ctx.reg[reg_a as usize] = get_x_imm(&pvm_ctx.pc, program);
    pvm_ctx.pc = skip(&pvm_ctx.pc, &program.bitmask);
    ExitReason::Continue
}

pub fn load_u8(pvm_ctx: &mut Context, program: &Program)-> ExitReason {
    let reg_a = get_reg(&pvm_ctx.pc, program);
    let address = get_x_imm(&pvm_ctx.pc, program);
    /*if let Some(value) = pvm_ctx.ram.get(&(address as usize)) {
        pvm_ctx.reg[reg_a as usize] = *value[0] as u64;
    } else {
        return ExitReason::PageFault(address as u32);
    }*/
    pvm_ctx.pc = skip(&pvm_ctx.pc, &program.bitmask);
    ExitReason::Continue
}

/*pub fn load_i8(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let reg_a = get_reg(&pvm_ctx.pc, program);
    let vx = get_x_imm(&pvm_ctx.pc, program);
    let address = vx % pvm_ctx.ram.page_map[0].address as u64;
    pvm_ctx.reg[reg_a as usize] = extend_sign(&[pvm_ctx.ram.chunk[0].contents[address as usize]], 1);
    pvm_ctx.pc = skip(&pvm_ctx.pc, &program.bitmask);
    ExitReason::Continue
}

pub fn load_u16(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let reg_a = get_reg(&pvm_ctx.pc, program);
    let vx = get_x_imm(&pvm_ctx.pc, program);
    let address = vx % pvm_ctx.ram.page_map[0].address as u64;
    let mut buffer = [0; std::mem::size_of::<u16>()];
    buffer.copy_from_slice(&pvm_ctx.ram.chunk[0].contents[address as usize..address as usize + 2]);
    pvm_ctx.reg[reg_a as usize] = u16::from_le_bytes(buffer) as u64;
    pvm_ctx.pc = skip(&pvm_ctx.pc, &program.bitmask);
    ExitReason::Continue
}

pub fn load_i16(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let reg_a = get_reg(&pvm_ctx.pc, program);
    let vx = get_x_imm(&pvm_ctx.pc, program);
    let address = vx % pvm_ctx.ram.page_map[0].address as u64;
    pvm_ctx.reg[reg_a as usize] = extend_sign(&pvm_ctx.ram.chunk[0].contents[address as usize..address as usize + 2], 2);
    pvm_ctx.pc = skip(&pvm_ctx.pc, &program.bitmask);
    ExitReason::Continue
}

pub fn load_u32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let reg_a = get_reg(&pvm_ctx.pc, program);
    let vx = get_x_imm(&pvm_ctx.pc, program);
    let address = vx % pvm_ctx.ram.page_map[0].address as u64;
    let mut buffer = [0; std::mem::size_of::<u32>()];
    buffer.copy_from_slice(&pvm_ctx.ram.chunk[0].contents[address as usize..address as usize + 4]);
    pvm_ctx.reg[reg_a as usize] = u32::from_le_bytes(buffer) as u64;
    pvm_ctx.pc = skip(&pvm_ctx.pc, &program.bitmask);
    ExitReason::Continue
}

pub fn load_i32(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let reg_a = get_reg(&pvm_ctx.pc, program);
    let vx = get_x_imm(&pvm_ctx.pc, program);
    let address = vx % pvm_ctx.ram.page_map[0].address as u64;
    pvm_ctx.reg[reg_a as usize] = extend_sign(&pvm_ctx.ram.chunk[0].contents[address as usize..address as usize + 4], 4);
    pvm_ctx.pc = skip(&pvm_ctx.pc, &program.bitmask);
    ExitReason::Continue
}

pub fn load_u64(pvm_ctx: &mut Context, program: &Program) -> ExitReason {
    let reg_a = get_reg(&pvm_ctx.pc, program);
    let vx = get_x_imm(&pvm_ctx.pc, program);
    let address = vx % pvm_ctx.ram.page_map[0].address as u64;
    let mut buffer = [0; std::mem::size_of::<u64>()];
    buffer.copy_from_slice(&pvm_ctx.ram.chunk[0].contents[address as usize..address as usize + 8]);
    pvm_ctx.reg[reg_a as usize] = u64::from_le_bytes(buffer);
    pvm_ctx.pc = skip(&pvm_ctx.pc, &program.bitmask);
    ExitReason::Continue
}*/

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

