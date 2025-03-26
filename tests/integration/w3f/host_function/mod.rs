use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::collections::HashMap;

extern crate vinwolf;

use vinwolf::pvm::hostcall::general_functions::info;
use vinwolf::types::GlobalState;

mod parser;
use parser::{parse_account, parse_context, parse_service_accounts, TestPart};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct HostCallTestFile {
    name: String,
    #[serde(rename = "initial-gas")]
    initial_gas: u64,
    #[serde(rename = "initial-regs")]
    initial_regs: HashMap<String, u64>,
    #[serde(rename = "initial-memory")]
    initial_memory: InitialMemory,
    #[serde(rename = "initial-service-account")]
    initial_service_account: DeltaEntry,
    #[serde(rename = "initial-service-index")]
    initial_service_index: u32,
    #[serde(rename = "initial-delta")]
    initial_delta: HashMap<String, DeltaEntry>,
    #[serde(rename = "expected-gas")]
    expected_gas: u64,
    #[serde(rename = "expected-regs")]
    expected_regs: HashMap<String, u64>,
    #[serde(rename = "expected-memory")]
    expected_memory: InitialMemory,
    #[serde(rename = "expected-delta")]
    expected_delta: HashMap<String, DeltaEntry>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct InitialMemory {
    pages: HashMap<String, TestPage>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct TestPage {
    value: Vec<u8>,
    access: Access,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Access {
    inaccessible: bool,
    writable: bool,
    readable: bool,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct LookupMap {
    t: Vec<u32>,
    l: u32,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct DeltaEntry {
    code_hash: String,
    balance: u64,
    g: u64,
    m: u64,
    #[serde(rename = "s_map")]
    s_map: HashMap<String, Vec<u8>>,
    #[serde(rename = "l_map")]
    l_map: HashMap<String, LookupMap>,
    #[serde(rename = "p_map")]
    p_map: HashMap<String, Vec<u8>>,
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn host_call_test() {
        run_hostcall_test("Info", "hostInfoNONE.json");
        //run_hostcall_test("Info", "hostInfoOOB.json");
        //run_hostcall_test("Info", "hostInfoOK.json");
    }

    fn run_hostcall_test(host_call_type: &str, test: &str) {

        println!("Running test {}", test);
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push(format!("tests/test_vectors/w3f/jamtestvectors/host_function/{}/{}", host_call_type, test));

        let mut file = File::open(path).unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();

        let json_data: HostCallTestFile = serde_json::from_str(&data).expect("Error deserializing  JSON");

        let mut context = parse_context(&json_data, TestPart::Initial);
        let service_id: u32 = json_data.initial_service_index;
        let mut account = parse_account(&json_data.initial_service_account);
        
        let mut global_state = GlobalState::default();
        let initial_service_accounts = parse_service_accounts(&json_data.initial_delta);
        global_state.service_accounts = initial_service_accounts;
        
        /*let output = info(&mut context, &service_id, &mut global_state.service_accounts);
        println!("output {:?}", output);*/

        let expected_service_accounts = parse_service_accounts(&json_data.expected_delta);
        let expected_context = parse_context(&json_data, TestPart::Expected);

        assert_eq!(expected_context.reg, context.reg);
        assert_eq!(expected_context.page_table, context.page_table);
        // TODO
        //assert_eq!(expected_context.gas, context.gas);
        assert_eq!(expected_service_accounts, global_state.service_accounts);
    
    }

}