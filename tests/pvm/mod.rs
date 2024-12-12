use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

extern crate vinwolf;

use vinwolf::pvm::invoke_pvm;
use vinwolf::pvm::{PageMap};

#[derive(Deserialize, Debug, PartialEq)]
struct Testcase {
    name: String,
    #[serde(rename = "initial-regs")]
    initial_regs: [u32; 13],
    #[serde(rename = "initial-pc")]
    initial_pc: u32,
    #[serde(rename = "initial-page-map")]
    initial_page_map: Vec<PageMap>,
    #[serde(rename = "initial-memory")]
    initial_memory: Vec<PageMap>,
    #[serde(rename = "initial-gas")]
    initial_gas: i64,
    program: Vec<u8>,
    #[serde(rename = "expected-status")]
    expected_status: String,
    #[serde(rename = "expected-regs")]
    expected_regs: [u32; 13],
    #[serde(rename = "expected-pc")]
    expected_pc: u32,
    #[serde(rename = "expected-memory")]
    expected_memory: Vec<PageMap>,
    #[serde(rename = "expected-gas")]
    expected_gas: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    fn run_pvm_test(filename: &str) {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push(filename);
        let mut file = File::open(&path).expect("Failed to open JSON file");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Failed to read JSON file");
        let testcase: Testcase = serde_json::from_str(&contents).expect("Failed to deserialize JSON");

        let (status, pc, gas, reg, ram) = invoke_pvm(
                                                    testcase.program.clone(),
                                                    testcase.initial_pc,
                                                    testcase.initial_gas,
                                                    testcase.initial_regs.clone(),
                                                    testcase.initial_memory.clone(),
                                                );          
        let result = Testcase {
            name: testcase.name.clone(),
            initial_regs: testcase.initial_regs.clone(),
            initial_pc: testcase.initial_pc,
            initial_page_map: testcase.initial_page_map.clone(),
            initial_memory: testcase.initial_memory.clone(),
            initial_gas: testcase.initial_gas,
            program: testcase.program.clone(),
            expected_status: status,
            expected_regs: reg,
            expected_pc: pc,
            expected_memory: ram,
            expected_gas: gas,
        };
        assert_eq!(testcase, result);
    }

    #[test]
    fn test_pvm_programs() {
        
        let test_files = vec![
            "tests/jamtestvectors/pvm/programs/gas_basic_consume_all.json",
            "tests/jamtestvectors/pvm/programs/inst_add.json",
            "tests/jamtestvectors/pvm/programs/inst_add_imm.json",
            "tests/jamtestvectors/pvm/programs/inst_add_with_overflow.json",
            "tests/jamtestvectors/pvm/programs/inst_and.json",
            "tests/jamtestvectors/pvm/programs/inst_and_imm.json",
            "tests/jamtestvectors/pvm/programs/inst_branch_eq_imm_nok.json",
            "tests/jamtestvectors/pvm/programs/inst_branch_eq_imm_ok.json"
        ];
        for file in test_files {
            println!("Running test for file: {}", file);
            run_pvm_test(file);
        }
    }
}