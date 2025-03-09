use crate::types::{Context, ExitReason, Program};
use crate::utils::codec::{BytesReader, Decode};
use crate::pvm::isa::skip;

use isa::one_offset::*;
use isa::no_arg::*;
use isa::two_reg::*;
use isa::two_reg_one_imm::*;
use isa::two_reg_two_imm::*;
use isa::two_reg_one_offset::*;
use isa::three_reg::*;
use isa::one_reg_one_ext_imm::*;
use isa::two_imm::*;
use isa::one_reg_one_imm::*;
use isa::one_reg_two_imm::*;
use isa::one_reg_one_imm_one_offset::*;

pub mod isa;
pub mod mm;
pub mod host_call;

pub fn invoke_pvm(pvm_ctx: &mut Context, program_blob: &[u8]) -> ExitReason {
    
    let program = Program::decode(&mut BytesReader::new(program_blob)).unwrap();

    while pvm_ctx.gas > 0 {

        let exit_reason = single_step_pvm(pvm_ctx, &program);
        //println!("reg_0 = {}", pvm_ctx.reg[0]);
        //println!("\n");
        // TODO revisar esto
        match exit_reason {
            ExitReason::Continue => {
                pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
            },
            ExitReason::Branch => {
              
            },
            ExitReason::PageFault(_) => {
                pvm_ctx.gas -= 1; 
                return ExitReason::page_fault;
            },
            _ => { return exit_reason; },
        }
    }

    return ExitReason::OutOfGas;
}

fn single_step_pvm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {

    pvm_ctx.gas -= 1;

    let next_instruction = program.code[pvm_ctx.pc as usize];
    /*println!("\nnext instruction = {next_instruction}");
    println!("pc = {:x?}", pvm_ctx.pc);
    println!("gas = {}", pvm_ctx.gas);
    println!("reg_0 = 0x{:x?}", pvm_ctx.reg[0]);
    println!("reg_1 = 0x{:x?}", pvm_ctx.reg[1]);
    println!("reg_2 = 0x{:x?}", pvm_ctx.reg[2]);
    println!("reg_3 = 0x{:x?}", pvm_ctx.reg[3]);
    println!("reg_4 = 0x{:x?}", pvm_ctx.reg[4]);
    println!("reg_5 = 0x{:x?}", pvm_ctx.reg[5]);
    println!("reg_6 = 0x{:x?}", pvm_ctx.reg[6]);
    println!("reg_7 = 0x{:x?}", pvm_ctx.reg[7]);
    println!("reg_8 = 0x{:x?}", pvm_ctx.reg[8]);
    println!("reg_9 = 0x{:x?}", pvm_ctx.reg[9]);
    println!("reg_10 = 0x{:x?}", pvm_ctx.reg[10]);
    println!("reg_11 = 0x{:x?}", pvm_ctx.reg[11]);
    println!("reg_12 = 0x{:x?}", pvm_ctx.reg[12]);*/

    let exit_reason = match next_instruction { 

        0_u8    => { trap() },
        1_u8    => { fallthrough() },
        //10_u8   => { ecalli(pvm_ctx, program) },
        20_u8   => { load_imm_64(pvm_ctx, program) },
        30_u8   => { store_imm_u8(pvm_ctx, program) },
        31_u8   => { store_imm_u16(pvm_ctx, program) },
        32_u8   => { store_imm_u32(pvm_ctx, program) },
        33_u8   => { store_imm_u64(pvm_ctx, program) },
        40_u8   => { jump(pvm_ctx, program) },
        50_u8   => { jump_ind(pvm_ctx, program) },
        51_u8   => { load_imm(pvm_ctx, program) },
        52_u8   => { load_u8(pvm_ctx, program) },
        53_u8   => { load_i8(pvm_ctx, program) },
        54_u8   => { load_u16(pvm_ctx, program) },
        55_u8   => { load_i16(pvm_ctx, program) },
        56_u8   => { load_u32(pvm_ctx, program) },
        57_u8   => { load_i32(pvm_ctx, program) },
        58_u8   => { load_u64(pvm_ctx, program) },
        59_u8   => { store_u8(pvm_ctx, program) },
        60_u8   => { store_u16(pvm_ctx, program) },
        61_u8   => { store_u32(pvm_ctx, program) },
        62_u8   => { store_u64(pvm_ctx, program) },
        70_u8   => { store_imm_ind_u8(pvm_ctx, program) },
        71_u8   => { store_imm_ind_u16(pvm_ctx, program) },
        72_u8   => { store_imm_ind_u32(pvm_ctx, program) },
        73_u8   => { store_imm_ind_u64(pvm_ctx, program) },
        80_u8   => { load_imm_jump(pvm_ctx, program) },
        81_u8   => { branch_eq_imm(pvm_ctx, program) },
        82_u8   => { branch_ne_imm(pvm_ctx, program) },
        83_u8   => { branch_lt_u_imm(pvm_ctx, program) },
        84_u8   => { branch_le_u_imm(pvm_ctx, program) },
        85_u8   => { branch_ge_u_imm(pvm_ctx, program) },
        86_u8   => { branch_gt_u_imm(pvm_ctx, program) },
        87_u8   => { branch_lt_s_imm(pvm_ctx, program) },
        88_u8   => { branch_le_s_imm(pvm_ctx, program) },
        89_u8   => { branch_ge_s_imm(pvm_ctx, program) },
        90_u8   => { branch_gt_s_imm(pvm_ctx, program) },
        100_u8  => { move_reg(pvm_ctx, program) }, 
        102_u8  => { count_set_bits_64(pvm_ctx, program) },
        103_u8  => { count_set_bits_32(pvm_ctx, program) },
        104_u8  => { leading_zero_bits_64(pvm_ctx, program) },
        105_u8  => { leading_zero_bits_32(pvm_ctx, program) },
        106_u8  => { trailing_zero_bits_64(pvm_ctx, program) },
        107_u8  => { trailing_zero_bits_32(pvm_ctx, program) },
        108_u8  => { sign_extend_8(pvm_ctx, program) },
        109_u8  => { sign_extend_16(pvm_ctx, program) },
        110_u8  => { zero_extend_16(pvm_ctx, program) },
        111_u8  => { reverse_bytes(pvm_ctx, program) },
        120_u8  => { store_ind_u8(pvm_ctx, program) },
        121_u8  => { store_ind_u16(pvm_ctx, program) },
        122_u8  => { store_ind_u32(pvm_ctx, program) },
        123_u8  => { store_ind_u64(pvm_ctx, program) },
        124_u8  => { load_ind_u8(pvm_ctx, program) },
        125_u8  => { load_ind_i8(pvm_ctx, program) },
        126_u8  => { load_ind_u16(pvm_ctx, program) },
        127_u8  => { load_ind_i16(pvm_ctx, program) },
        128_u8  => { load_ind_u32(pvm_ctx, program) },
        129_u8  => { load_ind_i32(pvm_ctx, program) },
        130_u8  => { load_ind_u64(pvm_ctx, program) },
        131_u8  => { add_imm_32(pvm_ctx, program) }, 
        132_u8  => { and_imm(pvm_ctx, program) },
        133_u8  => { xor_imm(pvm_ctx, program) },
        134_u8  => { or_imm(pvm_ctx, program) },
        135_u8  => { mul_imm_32(pvm_ctx, program) },
        136_u8  => { set_lt_u_imm(pvm_ctx, program) },
        137_u8  => { set_lt_s_imm(pvm_ctx, program) },
        138_u8  => { shlo_l_imm_32(pvm_ctx, program) },
        139_u8  => { shlo_r_imm_32(pvm_ctx, program) },
        140_u8  => { shar_r_imm_32(pvm_ctx, program) },
        141_u8  => { neg_add_imm_32(pvm_ctx, program) },
        142_u8  => { set_gt_u_imm(pvm_ctx, program) },
        143_u8  => { set_gt_s_imm(pvm_ctx, program) },
        144_u8  => { shlo_l_imm_alt_32(pvm_ctx, program) },
        145_u8  => { shlo_r_imm_alt_32(pvm_ctx, program) },
        146_u8  => { shar_r_imm_alt_32(pvm_ctx, program) },
        147_u8  => { cmov_iz_imm(pvm_ctx, program) },
        149_u8  => { add_imm_64(pvm_ctx, program) },
        150_u8  => { mul_imm_64(pvm_ctx, program) },
        151_u8  => { shlo_l_imm_64(pvm_ctx, program) },
        152_u8  => { shlo_r_imm_64(pvm_ctx, program) },
        153_u8  => { shar_r_imm_64(pvm_ctx, program) },    
        154_u8  => { neg_add_imm_64(pvm_ctx, program) },
        155_u8  => { shlo_l_imm_alt_64(pvm_ctx, program) },
        156_u8  => { shlo_r_imm_alt_64(pvm_ctx, program) },
        157_u8  => { shar_r_imm_alt_64(pvm_ctx, program) },
        158_u8  => { rot_r_64_imm(pvm_ctx, program) },
        159_u8  => { rot_r_64_imm_alt(pvm_ctx, program) },
        160_u8  => { rot_r_32_imm(pvm_ctx, program) },
        161_u8  => { rot_r_32_imm_alt(pvm_ctx, program) },
        170_u8  => { branch_eq(pvm_ctx, program) },
        171_u8  => { branch_ne(pvm_ctx, program) },
        172_u8  => { branch_lt_u(pvm_ctx, program) },
        173_u8  => { branch_lt_s(pvm_ctx, program) },
        174_u8  => { branch_ge_u(pvm_ctx, program) },
        175_u8  => { branch_ge_s(pvm_ctx, program) },
        180_u8  => { load_imm_jump_ind(pvm_ctx, program) },
        190_u8  => { add_32(pvm_ctx, program) },
        191_u8  => { sub_32(pvm_ctx, program) },
        192_u8  => { mul_32(pvm_ctx, program) },
        193_u8  => { div_u_32(pvm_ctx, program) },
        194_u8  => { div_s_32(pvm_ctx, program) },
        195_u8  => { rem_u_32(pvm_ctx, program) },
        196_u8  => { rem_s_32(pvm_ctx, program) },
        197_u8  => { shlo_l_32(pvm_ctx, program) },
        198_u8  => { shlo_r_32(pvm_ctx, program) },
        199_u8  => { shar_r_32(pvm_ctx, program) },
        200_u8  => { add_64(pvm_ctx, program) },
        201_u8  => { sub_64(pvm_ctx, program) },
        202_u8  => { mul_64(pvm_ctx, program) },
        203_u8  => { div_u_64(pvm_ctx, program) },
        204_u8  => { div_s_64(pvm_ctx, program) },
        205_u8  => { rem_u_64(pvm_ctx, program) },
        206_u8  => { rem_s_64(pvm_ctx, program) },
        207_u8  => { shlo_l_64(pvm_ctx, program) },
        208_u8  => { shlo_r_64(pvm_ctx, program) },
        209_u8  => { shar_r_64(pvm_ctx, program) },
        210_u8  => { and(pvm_ctx, program) },
        211_u8  => { xor(pvm_ctx, program) },
        212_u8  => { or(pvm_ctx, program) },
        213_u8  => { mul_upper_s_s(pvm_ctx, program) },
        214_u8  => { mul_upper_u_u(pvm_ctx, program) },
        215_u8  => { mul_upper_s_u(pvm_ctx, program) },
        216_u8  => { set_lt_u(pvm_ctx, program) },
        217_u8  => { set_lt_s(pvm_ctx, program) },
        218_u8  => { cmov_iz(pvm_ctx, program) },
        219_u8  => { cmov_nz(pvm_ctx, program) },
        220_u8  => { rot_l_64(pvm_ctx, program) },
        221_u8  => { rot_l_32(pvm_ctx, program) },
        222_u8  => { rot_r_64(pvm_ctx, program) },
        223_u8  => { rot_r_32(pvm_ctx, program) },
        224_u8  => { and_inv(pvm_ctx, program) },
        225_u8  => { or_inv(pvm_ctx, program) },
        226_u8  => { xnor(pvm_ctx, program) },
        227_u8  => { max(pvm_ctx, program) },
        228_u8  => { max_u(pvm_ctx, program) },
        229_u8  => { min(pvm_ctx, program) },
        230_u8  => { min_u(pvm_ctx, program) },
        _       => { return ExitReason::panic },   // Panic
    };

    return exit_reason;
}
