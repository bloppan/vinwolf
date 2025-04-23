use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use crate::integration::w3f::codec::{TestBody, encode_decode_test};

pub mod codec;
pub mod parser;
use parser::{deserialize_state_transition_file, read_state_snapshot};

extern crate vinwolf;

use vinwolf::types::{Block, GlobalState, OpaqueHash};
use vinwolf::constants::{*};
use vinwolf::blockchain::state::{get_global_state, state_transition_function, set_state_root};
use vinwolf::blockchain::state::set_global_state;
use vinwolf::utils::codec::{Decode, BytesReader};

use vinwolf::utils::trie::merkle_state;

#[derive(Debug, Deserialize)]
struct TestnetState {
    pub state_root: String,
    pub keyvals: Vec<(String, String, String, String)>, 
}

#[derive(Debug, Deserialize)]
struct TestData {
    pub pre_state: TestnetState,
    pub post_state: TestnetState,
}

#[derive(Debug)]
pub struct ParsedTransitionFile {
    pub pre_state_root: OpaqueHash,
    pub pre_state: GlobalState,
    pub post_state_root: OpaqueHash,
    pub post_state: GlobalState,
}

pub fn read_test(filename: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(filename);
    println!("Reading test file: {:?}", path);
    let mut file = match File::open(&path) {
        Ok(file) => file,
        Err(e) => {
            return Err(Box::new(e));
        }
    };
    let mut test_content = Vec::new();
    let _ = file.read_to_end(&mut test_content);
    Ok(test_content)
}


#[cfg(test)]
mod tests {

    use core::panic;

    use serde::ser;
    use vinwolf::blockchain::state::services;

    use super::*;

    #[test]
    fn run_testnet() {

        //run_jamduna_blocks("tests/test_vectors/testnet/jamtestnet/data/fallback");
        //run_jamduna_blocks("tests/test_vectors/testnet/jamtestnet/data/safrole");
        run_jamduna_blocks("tests/test_vectors/testnet/jamtestnet/data/assurances");
        //run_javajam_blocks("tests/test_vectors/testnet/javajam-trace/stf");
    }

    fn run_jamduna_blocks(dir: &str) {

        println!("Running blocks in {} mode", dir);

        let json_file = deserialize_state_transition_file(&format!("{}/state_transitions", dir), "1_000.json").unwrap();
        set_global_state(json_file.pre_state.clone());

        let body_block: Vec<TestBody> = vec![TestBody::Block];
    
        let mut epoch = 1;
        let mut slot = 0;

        loop {

            let filename = format!("{}_{}.bin", epoch, format!("{:03}", slot));
            let block_content = read_test(&format!("{}/blocks/{}", dir, filename));

            if block_content.is_err() {
                return;
            }

            let state_json_filename = format!("{}_{}.json", epoch, format!("{:03}", slot));
            let json_file = deserialize_state_transition_file(&format!("{}/state_transitions", dir), &state_json_filename).unwrap();
            println!("Importing block {}/{}", format!("{}/state_transitions", dir), state_json_filename);
            let block_content = block_content.unwrap();
            let encode_decode= encode_decode_test(&block_content.clone(), &body_block);
            if encode_decode.is_err() {
                println!("Error encoding/decoding block: {:?}", encode_decode);
                return;
            }
            let mut block_reader = BytesReader::new(&block_content);
            let block = Block::decode(&mut block_reader).expect("Error decoding Block");

            let error = state_transition_function(&block);
            if error.is_err() {
                println!("****************************************************** Error: {:?}", error);
                return;
            }
            let state = get_global_state().lock().unwrap().clone();
            
            
            assert_eq!(json_file.post_state.time, state.time);
            assert_eq!(json_file.post_state.safrole, state.safrole);
            assert_eq!(json_file.post_state.entropy, state.entropy);
            assert_eq!(json_file.post_state.curr_validators, state.curr_validators);
            assert_eq!(json_file.post_state.prev_validators, state.prev_validators);
            assert_eq!(json_file.post_state.disputes.offenders, state.disputes.offenders);
            assert_eq!(json_file.post_state.availability, state.availability);
            assert_eq!(json_file.post_state.ready_queue, state.ready_queue);
            assert_eq!(json_file.post_state.accumulation_history, state.accumulation_history);
            assert_eq!(json_file.post_state.privileges, state.privileges);
            assert_eq!(json_file.post_state.next_validators, state.next_validators);
            assert_eq!(json_file.post_state.auth_queues, state.auth_queues);
            assert_eq!(json_file.post_state.recent_history, state.recent_history);

            //assert_eq!(json_file.post_state.service_accounts, state.service_accounts);
            /*for account in state.service_accounts.service_accounts.iter() {
                println!("Service: {:?}", account.0);
                println!("Account: {:x?}", account.1);
            }*/
            for service_account in json_file.post_state.service_accounts.service_accounts.iter() {
                if let Some(account) = state.service_accounts.service_accounts.get(&service_account.0) {
                    //assert_eq!(service_account, state.service_accounts.service_accounts.get_key_value(&service_account.0).unwrap());
                    println!("TESTING service {:?}", service_account.0);
                    let (items, octets, _threshold) = account.get_footprint_and_threshold();

                    assert_eq!(service_account.1.storage, account.storage);
                    assert_eq!(service_account.1.lookup, account.lookup);
                    assert_eq!(service_account.1.preimages, account.preimages);
                    assert_eq!(service_account.1.code_hash, account.code_hash);
                    assert_eq!(service_account.1.balance, account.balance);
                    assert_eq!(service_account.1.items, items);
                    assert_eq!(service_account.1.gas, account.gas);
                    assert_eq!(service_account.1.min_gas, account.min_gas);
                    assert_eq!(service_account.1.bytes, octets);
                } else {
                    panic!("Service account not found in state: {:?}", service_account.0);
                }
            }
            assert_eq!(json_file.post_state.auth_pools, state.auth_pools);            
            assert_eq!(json_file.post_state.statistics, state.statistics);
            
            assert_eq!(json_file.post_state_root, merkle_state(&state.serialize().map, 0).unwrap());

            println!("state root: {:x?}", json_file.post_state_root);
            slot += 1;

            if slot == EPOCH_LENGTH {
                slot = 0;
                epoch += 1;
            } 
        }
    }

    fn run_javajam_blocks(dir: &str) {

        let body_block: Vec<TestBody> = vec![TestBody::Block];
        println!("Running JavaJAM blocks");
        let json_file = deserialize_state_transition_file(&format!("{}/state_transitions", dir), "1350458.json").unwrap();
        set_global_state(json_file.pre_state.clone());
        let mut filenumber = 1350458;
        println!("Importing block sequence");

        loop {

            let bin_filename = format!("{}/blocks/{}.bin", dir, filenumber);
            let block_content = read_test(&bin_filename);

            if block_content.is_err() {
                return;
            }

            println!("Importing block {}", bin_filename);
            let state_json_filename = format!("{}.json", filenumber);
            let json_file = deserialize_state_transition_file(&format!("{}/state_transitions", dir), &state_json_filename).unwrap();
            let block_content = block_content.unwrap();
            let _ = encode_decode_test(&block_content.clone(), &body_block);
            let mut block_reader = BytesReader::new(&block_content);
            let block = Block::decode(&mut block_reader).expect("Error decoding Block");

            let error = state_transition_function(&block);
            if error.is_err() {
                println!("****************************************************** Error: {:?}", error);
                return;
            }

            let state = get_global_state().lock().unwrap().clone();

            assert_eq!(json_file.post_state.auth_pools, state.auth_pools);
            assert_eq!(json_file.post_state.auth_queues, state.auth_queues);
            assert_eq!(json_file.post_state.recent_history, state.recent_history);
            assert_eq!(json_file.post_state.safrole, state.safrole);            
            assert_eq!(json_file.post_state.disputes.offenders, state.disputes.offenders);
            assert_eq!(json_file.post_state.entropy, state.entropy);
            assert_eq!(json_file.post_state.next_validators, state.next_validators);
            assert_eq!(json_file.post_state.curr_validators, state.curr_validators);
            assert_eq!(json_file.post_state.prev_validators, state.prev_validators);
            assert_eq!(json_file.post_state.availability, state.availability);
            assert_eq!(json_file.post_state.time, state.time);
            assert_eq!(json_file.post_state.privileges, state.privileges);
            assert_eq!(json_file.post_state.statistics, state.statistics);
            assert_eq!(json_file.post_state.accumulation_history, state.accumulation_history);
            assert_eq!(json_file.post_state.ready_queue, state.ready_queue);
            assert_eq!(json_file.post_state.service_accounts, state.service_accounts);        

            assert_eq!(json_file.post_state_root, merkle_state(&state.serialize().map, 0).unwrap());

            filenumber += 1;
        }
    }

}





