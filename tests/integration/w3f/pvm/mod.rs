use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

extern crate vinwolf;


use vinwolf::constants::{NUM_REG, PAGE_SIZE};
use vinwolf::pvm::invoke_pvm;
use vinwolf::types::{Context, ExitReason, MemoryChunk, PageMap, PageFlags, RamAddress, Gas, Page, PageTable};

#[derive(Deserialize, Debug, PartialEq)]
struct Testcase {
    name: String,
    #[serde(rename = "initial-regs")]
    initial_regs: [u64; NUM_REG as usize],
    #[serde(rename = "initial-pc")]
    initial_pc: u64,
    #[serde(rename = "initial-page-map")]
    initial_page_map: Vec<PageMap>,
    #[serde(rename = "initial-memory")]
    initial_memory: Vec<MemoryChunk>,
    #[serde(rename = "initial-gas")]
    initial_gas: Gas,
    program: Vec<u8>,
    #[serde(rename = "expected-status")]
    expected_status: ExitReason,
    #[serde(rename = "expected-regs")]
    expected_regs: [u64; NUM_REG as usize],
    #[serde(rename = "expected-pc")]
    expected_pc: u64,
    #[serde(rename = "expected-memory")]
    expected_memory: Vec<MemoryChunk>,
    #[serde(rename = "expected-gas")]
    expected_gas: Gas,
    #[serde(rename = "expected-page-fault-address")]
    expected_page_fault_address: Option<RamAddress>,
}

#[cfg(test)]
mod tests {

    use super::*;
    fn run_pvm_test(filename: &str) {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests/test_vectors/jamtestvectors/pvm/programs/");
        path.push(filename);
        let mut file = File::open(&path).expect("Failed to open JSON file");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Failed to read JSON file");
        let testcase: Testcase = serde_json::from_str(&contents).expect("Failed to deserialize JSON");

        let mut page_table = PageTable::default();
        for page in &testcase.initial_page_map {
            let page_number = page.address / PAGE_SIZE;
            page_table.pages.insert(page_number, Page {
                flags: PageFlags {
                    is_writable: page.is_writable,
                    referenced: false,
                    modified: false,
                },
                data: Box::new([0u8; PAGE_SIZE as usize]),
            });
        }

        for chunk in &testcase.initial_memory {
            let page_number = chunk.address / PAGE_SIZE;
            let offset = chunk.address % PAGE_SIZE;
            let page = page_table.pages.get_mut(&page_number).unwrap();
            for (i, byte) in chunk.contents.iter().enumerate() {
                page.data[offset as usize + i] = *byte;
            }
        }

        let mut pvm_ctx = Context {
            pc: testcase.initial_pc.clone(),
            gas: testcase.initial_gas.clone(),
            reg: testcase.initial_regs.clone(),
            page_table: page_table.clone(),
            page_fault: None,
        };

        let exit_reason = invoke_pvm(&mut pvm_ctx, &testcase.program);

        let mut ram_result: Vec<MemoryChunk> = vec![];              

        for chunk in testcase.expected_memory.iter() {

            let address = chunk.address;
            let contents = chunk.contents.clone();

            let page_target = address / PAGE_SIZE;
            let offset = address % PAGE_SIZE;

            if let Some(page) = pvm_ctx.page_table.pages.get_mut(&page_target) {
                let mut bytes_contents: Vec<u8> = vec![];
                for (i, byte) in contents.iter().enumerate() {
                    assert_eq!(*byte, page.data[offset as usize + i]);
                    bytes_contents.push(*byte);
                }
                let memory_chunk = MemoryChunk {
                    address: address,
                    contents: bytes_contents,
                };
                ram_result.push(memory_chunk);
            } else {
                panic!("Page not found");
            }
        }

        let result = Testcase {
            name: testcase.name.clone(),
            initial_regs: testcase.initial_regs.clone(),
            initial_pc: testcase.initial_pc,
            initial_page_map: testcase.initial_page_map.clone(),
            initial_memory: testcase.initial_memory.clone(),
            initial_gas: testcase.initial_gas,
            program: testcase.program.clone(),
            expected_status: exit_reason,
            expected_regs: pvm_ctx.reg.clone(),
            expected_pc: pvm_ctx.pc.clone(),
            expected_memory: ram_result,
            expected_gas: pvm_ctx.gas.clone(),
            expected_page_fault_address: pvm_ctx.page_fault.clone(),
        };

        assert_eq!(testcase.initial_regs, result.initial_regs);
        assert_eq!(testcase.initial_pc, result.initial_pc);
        assert_eq!(testcase.initial_page_map, result.initial_page_map);
        assert_eq!(testcase.initial_memory, result.initial_memory);
        assert_eq!(testcase.initial_gas, result.initial_gas);
        assert_eq!(testcase.program, result.program);
        assert_eq!(testcase.expected_status, result.expected_status);
        assert_eq!(testcase.expected_regs, result.expected_regs);
        assert_eq!(testcase.expected_pc, result.expected_pc);
        assert_eq!(testcase.expected_memory, result.expected_memory);
        assert_eq!(testcase.expected_gas, result.expected_gas);

        if pvm_ctx.page_fault.is_some() {
            assert_eq!(testcase.expected_page_fault_address, result.expected_page_fault_address);
        }

        assert_eq!(testcase, result);

    }

    #[test]
    fn test_programs() {
        let test_files = vec![
            "gas_basic_consume_all.json",
            "inst_add_32.json",
            "inst_add_32_with_overflow.json",
            "inst_add_32_with_truncation.json",
            "inst_add_32_with_truncation_and_sign_extension.json",
            "inst_add_64.json",
            "inst_add_64_with_overflow.json",
            "inst_add_imm_32.json",
            "inst_add_imm_32_with_truncation.json",
            "inst_add_imm_32_with_truncation_and_sign_extension.json",
            "inst_add_imm_64.json",
            "inst_and.json",
            "inst_and_imm.json",
            "inst_branch_eq_imm_nok.json",
            "inst_branch_eq_imm_ok.json",
            "inst_branch_eq_nok.json",
            "inst_branch_eq_ok.json",
            "inst_branch_greater_or_equal_signed_imm_nok.json",
            "inst_branch_greater_or_equal_signed_imm_ok.json",
            "inst_branch_greater_or_equal_signed_nok.json",
            "inst_branch_greater_or_equal_signed_ok.json",
            "inst_branch_greater_or_equal_unsigned_imm_nok.json",
            "inst_branch_greater_or_equal_unsigned_imm_ok.json",
            "inst_branch_greater_or_equal_unsigned_nok.json",
            "inst_branch_greater_or_equal_unsigned_ok.json",
            "inst_branch_greater_signed_imm_nok.json",
            "inst_branch_greater_signed_imm_ok.json",
            "inst_branch_greater_unsigned_imm_nok.json",
            "inst_branch_greater_unsigned_imm_ok.json",
            "inst_branch_less_or_equal_signed_imm_nok.json",
            "inst_branch_less_or_equal_signed_imm_ok.json",
            "inst_branch_less_or_equal_unsigned_imm_nok.json",
            "inst_branch_less_or_equal_unsigned_imm_ok.json",
            "inst_branch_less_signed_imm_nok.json",
            "inst_branch_less_signed_imm_ok.json",
            "inst_branch_less_signed_nok.json",
            "inst_branch_less_signed_ok.json",
            "inst_branch_less_unsigned_imm_nok.json",
            "inst_branch_less_unsigned_imm_ok.json",
            "inst_branch_less_unsigned_nok.json",
            "inst_branch_less_unsigned_ok.json",
            "inst_branch_not_eq_imm_nok.json",
            "inst_branch_not_eq_imm_ok.json",
            "inst_branch_not_eq_nok.json",
            "inst_branch_not_eq_ok.json",
            "inst_cmov_if_zero_imm_nok.json",
            "inst_cmov_if_zero_imm_ok.json",
            "inst_cmov_if_zero_nok.json",
            "inst_cmov_if_zero_ok.json",
            "inst_div_signed_32.json",
            "inst_div_signed_32_by_zero.json",
            "inst_div_signed_32_with_overflow.json",
            "inst_div_signed_64.json",
            "inst_div_signed_64_by_zero.json",
            "inst_div_signed_64_with_overflow.json",
            "inst_div_unsigned_32.json",
            "inst_div_unsigned_32_by_zero.json",
            "inst_div_unsigned_32_with_overflow.json",
            "inst_div_unsigned_64.json",
            "inst_div_unsigned_64_by_zero.json",
            "inst_div_unsigned_64_with_overflow.json",
            "inst_fallthrough.json",
            "inst_jump.json",
            "inst_jump_indirect_invalid_djump_to_zero_nok.json",
            "inst_jump_indirect_misaligned_djump_with_offset_nok.json",
            "inst_jump_indirect_misaligned_djump_without_offset_nok.json",
            "inst_jump_indirect_with_offset_ok.json",
            "inst_jump_indirect_without_offset_ok.json",
            "inst_load_i16.json",
            "inst_load_i32.json",
            "inst_load_i8.json",
            "inst_load_imm.json",
            "inst_load_imm_64.json",
            "inst_load_imm_and_jump.json",
            "inst_load_imm_and_jump_indirect_different_regs_with_offset_ok.json",
            "inst_load_imm_and_jump_indirect_different_regs_without_offset_ok.json",
            "inst_load_imm_and_jump_indirect_invalid_djump_to_zero_different_regs_without_offset_nok.json",
            "inst_load_imm_and_jump_indirect_invalid_djump_to_zero_same_regs_without_offset_nok.json",
            "inst_load_imm_and_jump_indirect_misaligned_djump_different_regs_with_offset_nok.json",
            "inst_load_imm_and_jump_indirect_misaligned_djump_different_regs_without_offset_nok.json",
            "inst_load_imm_and_jump_indirect_misaligned_djump_same_regs_with_offset_nok.json",
            "inst_load_imm_and_jump_indirect_misaligned_djump_same_regs_without_offset_nok.json",
            "inst_load_imm_and_jump_indirect_same_regs_with_offset_ok.json",
            "inst_load_imm_and_jump_indirect_same_regs_without_offset_ok.json",
            "inst_load_indirect_i16_with_offset.json",
            "inst_load_indirect_i16_without_offset.json",
            "inst_load_indirect_i32_with_offset.json",
            "inst_load_indirect_i32_without_offset.json",
            "inst_load_indirect_i8_with_offset.json",
            "inst_load_indirect_i8_without_offset.json",
            "inst_load_indirect_u16_with_offset.json",
            "inst_load_indirect_u16_without_offset.json",
            "inst_load_indirect_u32_with_offset.json",
            "inst_load_indirect_u32_without_offset.json",
            "inst_load_indirect_u64_with_offset.json",
            "inst_load_indirect_u64_without_offset.json",
            "inst_load_indirect_u8_with_offset.json",
            "inst_load_indirect_u8_without_offset.json",
            "inst_load_u16.json",
            "inst_load_u32.json",
            "inst_load_u64.json",
            "inst_load_u8.json",
            "inst_load_u8_nok.json",
            "inst_move_reg.json",
            "inst_mul_32.json",
            "inst_mul_64.json",
            "inst_mul_imm_32.json",
            "inst_mul_imm_64.json",
            "inst_negate_and_add_imm_32.json",
            "inst_negate_and_add_imm_64.json",
            "inst_or.json",
            "inst_or_imm.json",
            "inst_rem_signed_32.json",
            "inst_rem_signed_32_by_zero.json",
            "inst_rem_signed_32_with_overflow.json",
            "inst_rem_signed_64.json",
            "inst_rem_signed_64_by_zero.json",
            "inst_rem_signed_64_with_overflow.json",
            "inst_rem_unsigned_32.json",
            "inst_rem_unsigned_32_by_zero.json",
            "inst_rem_unsigned_64.json",
            "inst_rem_unsigned_64_by_zero.json",
            "inst_ret_halt.json",
            "inst_ret_invalid.json",
            "inst_set_greater_than_signed_imm_0.json",
            "inst_set_greater_than_signed_imm_1.json",
            "inst_set_greater_than_unsigned_imm_0.json",
            "inst_set_greater_than_unsigned_imm_1.json",
            "inst_set_less_than_signed_0.json",
            "inst_set_less_than_signed_1.json",
            "inst_set_less_than_signed_imm_0.json",
            "inst_set_less_than_signed_imm_1.json",
            "inst_set_less_than_unsigned_0.json",
            "inst_set_less_than_unsigned_1.json",
            "inst_set_less_than_unsigned_imm_0.json",
            "inst_set_less_than_unsigned_imm_1.json",
            "inst_shift_arithmetic_right_32.json",
            "inst_shift_arithmetic_right_32_with_overflow.json",
            "inst_shift_arithmetic_right_64.json",
            "inst_shift_arithmetic_right_64_with_overflow.json",
            "inst_shift_arithmetic_right_imm_32.json",
            "inst_shift_arithmetic_right_imm_64.json",
            "inst_shift_arithmetic_right_imm_alt_32.json",
            "inst_shift_arithmetic_right_imm_alt_64.json",
            "inst_shift_logical_left_32.json",
            "inst_shift_logical_left_32_with_overflow.json",
            "inst_shift_logical_left_64.json",
            "inst_shift_logical_left_64_with_overflow.json",
            "inst_shift_logical_left_imm_32.json",
            "inst_shift_logical_left_imm_64.json",
            "inst_shift_logical_left_imm_alt_32.json",
            "inst_shift_logical_left_imm_alt_64.json",
            "inst_shift_logical_right_32.json",
            "inst_shift_logical_right_32_with_overflow.json",
            "inst_shift_logical_right_64.json",
            "inst_shift_logical_right_64_with_overflow.json",
            "inst_shift_logical_right_imm_32.json",
            "inst_shift_logical_right_imm_64.json",
            "inst_shift_logical_right_imm_alt_32.json",
            "inst_shift_logical_right_imm_alt_64.json",
            "inst_store_imm_indirect_u16_with_offset_nok.json",
            "inst_store_imm_indirect_u16_with_offset_ok.json",
            "inst_store_imm_indirect_u16_without_offset_ok.json",
            "inst_store_imm_indirect_u32_with_offset_nok.json",
            "inst_store_imm_indirect_u32_with_offset_ok.json",
            "inst_store_imm_indirect_u32_without_offset_ok.json",
            "inst_store_imm_indirect_u64_with_offset_nok.json",
            "inst_store_imm_indirect_u64_with_offset_ok.json",
            "inst_store_imm_indirect_u64_without_offset_ok.json",
            "inst_store_imm_indirect_u8_with_offset_nok.json",
            "inst_store_imm_indirect_u8_with_offset_ok.json",
            "inst_store_imm_indirect_u8_without_offset_ok.json",
            "inst_store_imm_u16.json",
            "inst_store_imm_u32.json",
            "inst_store_imm_u64.json",
            "inst_store_imm_u8.json",
            "inst_store_imm_u8_trap_inaccessible.json",
            // "inst_store_indirect_u16_with_offset_nok.json", lo tiene que revisar JanBujak
            "inst_store_indirect_u16_with_offset_ok.json",
            "inst_store_indirect_u16_without_offset_ok.json",
            // "inst_store_indirect_u32_with_offset_nok.json", lo tiene que revisar JanBujak
            "inst_store_indirect_u32_with_offset_ok.json",
            "inst_store_indirect_u32_without_offset_ok.json",
            // "inst_store_indirect_u64_with_offset_nok.json", lo tiene que revisar JanBujak
            "inst_store_indirect_u64_with_offset_ok.json",
            "inst_store_indirect_u64_without_offset_ok.json",
            // "inst_store_indirect_u8_with_offset_nok.json", lo tiene que revisar JanBujak
            "inst_store_indirect_u8_with_offset_ok.json",
            "inst_store_indirect_u8_without_offset_ok.json",
            "inst_store_u16.json",
            "inst_store_u32.json",
            "inst_store_u64.json",
            "inst_store_u8.json",
            "inst_store_u8_trap_inaccessible.json",
            "inst_sub_32.json",
            "inst_sub_32_with_overflow.json",
            "inst_sub_64.json",
            "inst_sub_64_with_overflow.json",
            "inst_sub_imm_32.json",
            "inst_sub_imm_64.json",
            "inst_trap.json",
            "inst_xor.json",
            "inst_xor_imm.json",
            "riscv_rv64ua_amoadd_d.json",
            "riscv_rv64ua_amoadd_w.json",
            "riscv_rv64ua_amoand_d.json",
            "riscv_rv64ua_amoand_w.json",
            "riscv_rv64ua_amomax_d.json",
            "riscv_rv64ua_amomax_w.json",
            "riscv_rv64ua_amomaxu_d.json",
            "riscv_rv64ua_amomaxu_w.json",
            "riscv_rv64ua_amomin_d.json",
            "riscv_rv64ua_amomin_w.json",
            "riscv_rv64ua_amominu_d.json",
            "riscv_rv64ua_amominu_w.json",
            "riscv_rv64ua_amoor_d.json",
            "riscv_rv64ua_amoor_w.json",
            "riscv_rv64ua_amoswap_d.json",
            "riscv_rv64ua_amoswap_w.json",
            "riscv_rv64ua_amoxor_d.json",
            "riscv_rv64ua_amoxor_w.json",
            "riscv_rv64uc_rvc.json",
            "riscv_rv64ui_add.json",
            "riscv_rv64ui_addi.json",
            "riscv_rv64ui_addiw.json",
            "riscv_rv64ui_addw.json",
            "riscv_rv64ui_and.json",
            "riscv_rv64ui_andi.json",
            "riscv_rv64ui_beq.json",
            "riscv_rv64ui_bge.json",
            "riscv_rv64ui_bgeu.json",
            "riscv_rv64ui_blt.json",
            "riscv_rv64ui_bltu.json",
            "riscv_rv64ui_bne.json",
            "riscv_rv64ui_jal.json",
            "riscv_rv64ui_jalr.json",
            "riscv_rv64ui_lb.json",
            "riscv_rv64ui_lbu.json",
            "riscv_rv64ui_ld.json",
            "riscv_rv64ui_lh.json",
            "riscv_rv64ui_lhu.json",
            "riscv_rv64ui_lui.json",
            "riscv_rv64ui_lw.json",
            "riscv_rv64ui_lwu.json",
            "riscv_rv64ui_ma_data.json",
            "riscv_rv64ui_or.json",
            "riscv_rv64ui_ori.json",
            "riscv_rv64ui_sb.json",
            "riscv_rv64ui_sd.json",
            "riscv_rv64ui_sh.json",
            "riscv_rv64ui_simple.json",
            "riscv_rv64ui_sll.json",
            "riscv_rv64ui_slli.json",
            "riscv_rv64ui_slliw.json",
            "riscv_rv64ui_sllw.json",
            "riscv_rv64ui_slt.json",
            "riscv_rv64ui_slti.json",
            "riscv_rv64ui_sltiu.json",
            "riscv_rv64ui_sltu.json",
            "riscv_rv64ui_sra.json",
            "riscv_rv64ui_srai.json",
            "riscv_rv64ui_sraiw.json",
            "riscv_rv64ui_sraw.json",
            "riscv_rv64ui_srl.json",
            "riscv_rv64ui_srli.json",
            "riscv_rv64ui_srliw.json",
            "riscv_rv64ui_srlw.json",
            "riscv_rv64ui_sub.json",
            "riscv_rv64ui_subw.json",
            "riscv_rv64ui_sw.json",
            "riscv_rv64ui_xor.json",
            "riscv_rv64ui_xori.json",
            "riscv_rv64um_div.json",
            "riscv_rv64um_divu.json",
            "riscv_rv64um_divuw.json",
            "riscv_rv64um_divw.json",
            "riscv_rv64um_mul.json",
            "riscv_rv64um_mulh.json",
            "riscv_rv64um_mulhsu.json",
            "riscv_rv64um_mulhu.json",
            "riscv_rv64um_mulw.json",
            "riscv_rv64um_rem.json",
            "riscv_rv64um_remu.json",
            "riscv_rv64um_remuw.json",
            "riscv_rv64um_remw.json",
            "riscv_rv64uzbb_andn.json",
            "riscv_rv64uzbb_clz.json",
            "riscv_rv64uzbb_clzw.json",
            "riscv_rv64uzbb_cpop.json",
            "riscv_rv64uzbb_cpopw.json",
            "riscv_rv64uzbb_ctz.json",
            "riscv_rv64uzbb_ctzw.json",
            "riscv_rv64uzbb_max.json",
            "riscv_rv64uzbb_maxu.json",
            "riscv_rv64uzbb_min.json",
            "riscv_rv64uzbb_minu.json",
            "riscv_rv64uzbb_orc_b.json",
            "riscv_rv64uzbb_orn.json",
            "riscv_rv64uzbb_rev8.json",
            "riscv_rv64uzbb_rol.json",
            "riscv_rv64uzbb_rolw.json",
            "riscv_rv64uzbb_ror.json",
            "riscv_rv64uzbb_rori.json",
            "riscv_rv64uzbb_roriw.json",
            "riscv_rv64uzbb_rorw.json",
            "riscv_rv64uzbb_sext_b.json",
            "riscv_rv64uzbb_sext_h.json",
            "riscv_rv64uzbb_xnor.json",
            "riscv_rv64uzbb_zext_h.json",
        ];
        
        for file in test_files {
            println!("Running test for file: {}", file);
            run_pvm_test(file);
            //println!("Ok");
        }
    }
}