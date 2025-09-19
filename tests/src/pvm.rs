#[cfg(test)]
mod tests {

    use std::fs::File;
    use std::io::Read;
    use std::path::PathBuf;

    use codec::{BytesReader, Decode};
    use constants::pvm::{NUM_REG, PAGE_SIZE};
    use pvm::invoke_pvm;
    use jam_types::{Gas};
    use pvm::pvm_types::{Program, PageFlags, RamAddress, ExitReason, Page, RamMemory};
    use utils::serde::{Deserialize, Value, from_json_str};

    #[derive(Debug, PartialEq)]
    struct Testcase {
        name: String,
        initial_regs: Registers,
        initial_pc: u64,
        initial_page_map: Vec<PageMap>,
        initial_memory: Vec<MemoryChunk>,
        initial_gas: Gas,
        program: Vec<u8>,
        expected_status: ExitReason,
        expected_regs: Registers,
        expected_pc: u64,
        expected_memory: Vec<MemoryChunk>,
        expected_gas: Gas,
        expected_page_fault_address: Option<RamAddress>,
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct PageMap {
        pub address: u32,
        pub length: u32,
        pub is_writable: bool,
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct MemoryChunk {
        pub address: u32,
        pub contents: Vec<u8>,
    }
    #[derive(Debug, PartialEq)]
    struct Registers([u64; NUM_REG]);

    impl Deserialize for PageMap {
        fn from_value(v: &Value) -> Result<Self, String> {
            let o = match v {
                Value::Object(o) => o,
                _ => return Err("expected object".into()),
            };
            let address = RamAddress::from_value(o.get("address").ok_or("missing address")?)?;
            let length = RamAddress::from_value(o.get("length").ok_or("missing length")?)?;
            let is_writable = bool::from_value(o.get("is-writable").ok_or("missing is-writable")?)?;
            Ok(PageMap {
                address,
                length,
                is_writable,
            })
        }
    }

    impl Deserialize for MemoryChunk {
        fn from_value(v: &Value) -> Result<Self, String> {
            let o = match v {
                Value::Object(o) => o,
                _ => return Err("expected object".into()),
            };
            let address = u32::from_value(o.get("address").ok_or("missing address")?)?;
            let contents = Vec::<u8>::from_value(o.get("contents").ok_or("missing contents")?)?;
            Ok(MemoryChunk { address, contents })
        }
    }

    impl Deserialize for Registers {
        fn from_value(v: &Value) -> Result<Self, String> {
            match v {
                Value::Array(a) => {
                    if a.len() != NUM_REG {
                        return Err(format!("expected {} elements", NUM_REG));
                    }
                    let mut arr = Registers([0u64; NUM_REG]);
                    for (i, value) in a.iter().enumerate() {
                        arr.0[i] = u64::from_value(value)?;
                    }
                    Ok(arr)
                }
                _ => Err("expected array".into()),
            }
        }
    }

    impl Deserialize for Testcase {
        fn from_value(v: &Value) -> Result<Self, String> {
            let o = match v {
                Value::Object(o) => o,
                _ => return Err("expected object".into()),
            };
            let name = String::from_value(o.get("name").ok_or("missing name")?)?;
            let initial_regs = Registers::from_value(
                o.get("initial-regs").ok_or("missing initial-regs")?,
            )?;
            let initial_pc = u64::from_value(o.get("initial-pc").ok_or("missing initial-pc")?)?;
            let initial_page_map = Vec::<PageMap>::from_value(
                o.get("initial-page-map").ok_or("missing initial-page-map")?,
            )?;
            let initial_memory = Vec::<MemoryChunk>::from_value(
                o.get("initial-memory").ok_or("missing initial-memory")?,
            )?;
            let initial_gas = Gas::from_value(o.get("initial-gas").ok_or("missing initial-gas")?)?;
            let program = Vec::<u8>::from_value(o.get("program").ok_or("missing program")?)?;
            let status_str = String::from_value(o.get("expected-status").ok_or("missing expected-status")?)?;
            let expected_regs = Registers::from_value(
                o.get("expected-regs").ok_or("missing expected-regs")?,
            )?;
            let expected_pc = u64::from_value(o.get("expected-pc").ok_or("missing expected-pc")?)?;
            let expected_memory = Vec::<MemoryChunk>::from_value(
                o.get("expected-memory").ok_or("missing expected-memory")?,
            )?;
            let expected_gas = Gas::from_value(o.get("expected-gas").ok_or("missing expected-gas")?)?;
            let expected_page_fault_address = match o.get("expected-page-fault-address") {
                Some(v) => Some(RamAddress::from_value(v)?),
                None => None,
            };
            let expected_status = match status_str.as_str() {
                "panic" => ExitReason::panic,
                "halt" => ExitReason::halt,
                "page-fault" => ExitReason::PageFault(
                    expected_page_fault_address.ok_or("missing expected-page-fault-address for page-fault status")?
                ),
                _ => return Err(format!("unknown exit reason: {}", status_str)),
            };
            Ok(Testcase {
                name,
                initial_regs,
                initial_pc,
                initial_page_map,
                initial_memory,
                initial_gas,
                program,
                expected_status,
                expected_regs,
                expected_pc,
                expected_memory,
                expected_gas,
                expected_page_fault_address,
            })
        }
    }

    fn run_pvm_test(filename: &str) {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("jamtestvectors/pvm/programs/");
        path.push(filename);
        let mut file = File::open(&path).expect("Failed to open JSON file");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Failed to read JSON file");
        let testcase: Testcase = from_json_str(&contents).unwrap();
        let mut ram = RamMemory::default();

        for page in &testcase.initial_page_map {
            let page_number = page.address / PAGE_SIZE;
            let page_content = Page {
                flags: PageFlags { 
                    read_access: true, 
                    write_access: page.is_writable, 
                    referenced: false, 
                    modified: false, 
                },
                data: Box::new([0u8; PAGE_SIZE as usize]),
            };
            ram.pages[page_number as usize] = Some(page_content);
        }

        for chunk in &testcase.initial_memory {
            let page_number = chunk.address / PAGE_SIZE;
            let offset = chunk.address % PAGE_SIZE;
            let page = ram.pages[page_number as usize].as_mut().unwrap();
            for (i, byte) in chunk.contents.iter().enumerate() {
                page.data[offset as usize + i] = *byte;
            }
        }

        let program = match Program::decode(&mut BytesReader::new(&testcase.program)) {
            Ok(program) => { program },
            Err(_) => { 
                utils::log::error!("Panic: Decoding code program");
                return; 
            }
        };

        let mut pc = testcase.initial_pc;
        let mut reg = testcase.initial_regs;
        let mut gas = testcase.initial_gas;

        utils::log::info!("Initial regs: {:?}", reg);
        let exit_reason = invoke_pvm(&program, &mut pc, &mut gas, &mut ram, &mut reg.0);

        let mut ram_result: Vec<MemoryChunk> = vec![];              

        for chunk in testcase.expected_memory.iter() {

            let address = chunk.address;
            let contents = chunk.contents.clone();

            let page_target = address / PAGE_SIZE;
            let offset = address % PAGE_SIZE;

            if ram.pages[page_target as usize].is_some() {
                let mut bytes_contents: Vec<u8> = vec![];
                for (i, byte) in contents.iter().enumerate() {
                    assert_eq!(*byte, ram.pages[page_target as usize].as_ref().unwrap().data[offset as usize + i]);
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

        //assert_eq!(testcase.program, result.program);
        assert_eq!(testcase.expected_status, exit_reason);
        assert_eq!(testcase.expected_regs.0, reg.0);
        //assert_eq!(testcase.expected_pc, pc);
        assert_eq!(testcase.expected_memory, ram_result);
    }

    #[test]
    fn test_programs() {
        utils::log::Builder::from_env(utils::log::Env::default().default_filter_or("debug"))
            .with_dotenv(true)
            .init();

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
            //"inst_load_u8_nok.json",
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
            utils::log::info!("Running test for file: {}", file);
            run_pvm_test(file);
            //println!("Ok");
        }
    }
}