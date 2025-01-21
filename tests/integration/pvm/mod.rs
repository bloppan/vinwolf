use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

extern crate vinwolf;


use vinwolf::constants::{NUM_REG, PAGE_SIZE};
use vinwolf::pvm::invoke_pvm;
use vinwolf::types::{Context, ExitReason, MemoryChunk, PageMap, PageFlags, RamAddress, RamMemory, Gas, Page, PageTable};

mod isa;

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
}

#[cfg(test)]
mod tests {
    
    use std::ops::Add;

    use vinwolf::types::RamMemory;

    use super::*;
    fn run_pvm_test(filename: &str) {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests/jamtestvectors/pvm/programs/");
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
        };

        let exit_reason = invoke_pvm(&mut pvm_ctx, &testcase.program);

        let mut ram_result: Vec<MemoryChunk> = vec![];

        for page in pvm_ctx.page_table.pages.iter() {
                let mut content = vec![];
                let mut offset: Option<RamAddress> = None;
                if page.1.flags.modified || page.1.flags.referenced {
                    println!("Page modified or referended: {}", page.0);
                    println!("Data: {:?}", page.1.data[0..10].to_vec());
                    for (i, byte) in page.1.data.iter().enumerate() {
                        //println!("Byte: {byte}, pos: {i}");
                        if *byte != 0 {
                            println!("Byte: {byte}, pos: {i}");
                            if offset.is_none() {
                                offset = Some(i as RamAddress);
                                println!("Offset: {}", offset.unwrap());
                            }
                            
                            content.push(*byte);
                        }
                    }
                    if !offset.is_none() {
                        ram_result.push(MemoryChunk {
                            address: (page.0 * PAGE_SIZE).wrapping_add(offset.unwrap() as u32),
                            contents: content,
                        });
                    }
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
        };

        assert_eq!(testcase, result);

    }

    #[test]
    fn test_pvm_programs() {
        
        let test_files = vec![
            /*"inst_move_reg.json",
            "inst_add_imm_32.json",
            "inst_add_imm_32_with_truncation.json",
            "inst_add_imm_32_with_truncation_and_sign_extension.json",
            "inst_add_32.json",
            "inst_add_32_with_overflow.json",
            "inst_add_32_with_truncation.json",
            "inst_add_32_with_truncation_and_sign_extension.json",
            "inst_and.json",
            "inst_and_imm.json",
            "inst_xor_imm.json",
            "inst_xor.json",
            "inst_trap.json",
            "inst_sub_32.json",
            "inst_sub_32_with_overflow.json",
            "inst_sub_64.json",
            "inst_sub_64_with_overflow.json",
            "inst_load_imm_64.json",*/
            "inst_store_imm_u8.json",
            "inst_store_imm_u16.json",
            "inst_store_imm_u32.json",
            //"inst_load_imm.json",
            "inst_load_u8.json",
            /*"inst_load_i8.json",
            "inst_load_u16.json",
            "inst_load_i16.json",
            "inst_load_u32.json",
            "inst_load_i32.json",
            "inst_load_u64.json",*/
            "inst_store_u8.json",
            "inst_store_u8_trap_read_only.json",
            "inst_store_u8_trap_inaccessible.json",
            "inst_store_u16.json",
            "inst_store_u32.json",
            "inst_store_u64.json",
            "inst_store_imm_indirect_u8_with_offset_ok.json",
            "inst_store_imm_indirect_u8_with_offset_nok.json",
            "inst_store_imm_indirect_u8_without_offset_ok.json",
            "inst_store_imm_indirect_u16_with_offset_ok.json",
            "inst_store_imm_indirect_u16_with_offset_nok.json",
            "inst_store_imm_indirect_u32_with_offset_ok.json",
            "inst_store_imm_indirect_u32_with_offset_nok.json",
            "inst_store_imm_indirect_u64_with_offset_ok.json",
            "inst_store_imm_indirect_u64_with_offset_nok.json",
            /*"gas_basic_consume_all.json",
            "inst_add_imm.json",
            "inst_add_with_overflow.json",
            "inst_branch_eq_imm_nok.json",
            "inst_branch_eq_imm_ok.json"*/
        ];
        for file in test_files {
            println!("Running test for file: {}", file);
            run_pvm_test(file);
            println!("Ok\n\n");
        }
    }
}