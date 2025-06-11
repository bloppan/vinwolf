use once_cell::sync::Lazy;
use crate::integration::w3f::{read_test_file, FromProcessError};
use crate::integration::w3f::codec::{TestBody, encode_decode_test};

use vinwolf::constants::{*};
use vinwolf::types::{RawState, Block, AuthPools, AuthQueues, BlockHistory, Safrole, DisputesRecords, EntropyPool, ValidatorsData, AvailabilityAssignments,
    Privileges, Statistics, ReadyQueue, AccumulatedHistory, OpaqueHash, Gas, ServiceId, Extrinsic, ServiceAccounts, Account};
use vinwolf::blockchain::state::{
    get_global_state, set_reporting_assurance, get_reporting_assurance, set_auth_pools, get_auth_pools, 
    set_entropy, get_entropy, set_validators, get_validators, set_recent_history, get_recent_history,
    set_disputes, get_disputes, set_statistics, get_statistics, set_service_accounts, get_service_accounts,
    state_transition_function
};
use vinwolf::blockchain::state::reporting_assurance::process_guarantees;
use vinwolf::blockchain::state::statistics::process;
use vinwolf::utils::codec::{Decode, DecodeLen, BytesReader};
use vinwolf::utils::codec::generic::decode;
use vinwolf::utils::trie::merkle_state;

pub mod codec;


#[cfg(test)]
mod tests {
    
    /*impl FromProcessError for OutputWorkReport {
        fn from_process_error(error: ProcessError) -> Self {
            match error {
                ProcessError::ReportError(code) => OutputWorkReport::Err(code),
                _ => panic!("Unexpected error type in conversion"),
            }
        }
    }*/

    use vinwolf::{blockchain::state::set_global_state, types::{GlobalState, TimeSlot}};

    use super::*;

    fn run_test(filename: &str) {

        

    }

    #[test]
    fn run_traces_tests() {

        let test_body: Vec<TestBody> = vec![TestBody::RawState,
                                            TestBody::Block,
                                            TestBody::RawState];

        let test_content = read_test_file(&format!("tests/test_vectors/w3f/jamtestvectors/traces/reports-l0/00000000.bin"));
        let _ = encode_decode_test(&test_content, &test_body);
        
        let mut reader = BytesReader::new(&test_content);
        let pre_state = RawState::decode(&mut reader).expect("Error decoding post WorkReport PreState");
        let block = Block::decode(&mut reader).expect("Error decoding post OutputWorkReport");
        let post_state = RawState::decode(&mut reader).expect("Error decoding post WorkReport PostState");
        //println!("pre_state: {:x?}", pre_state);
        //println!("block: {:x?}", block);
        //println!("post_state: {:x?}", post_state);
        
        let mut slot = 1;
        
        loop {
            println!("\n\nReading test file: {}", slot);
            //let test_content = read_test_file(&format!("tests/test_vectors/w3f/jamtestvectors/traces/reports-l0/{:08}.bin", slot));
            let test_content = read_test_file(&format!("tests/test_vectors/w3f/jamtestvectors/traces/safrole/{:08}.bin", slot));
            let _ = encode_decode_test(&test_content, &test_body);
            
            let mut reader = BytesReader::new(&test_content);
            let pre_state = RawState::decode(&mut reader).expect("Error decoding post WorkReport PreState");
            let block = Block::decode(&mut reader).expect("Error decoding post OutputWorkReport");
            let post_state = RawState::decode(&mut reader).expect("Error decoding post WorkReport PostState");

            let mut state = GlobalState::default();
            let mut expected_state = GlobalState::default();
            set_raw_state(pre_state.clone(), &mut state);
            set_raw_state(post_state.clone(), &mut expected_state);
            
            assert_eq!(pre_state.state_root.clone(), merkle_state(&state.serialize().map, 0).unwrap());

            set_global_state(state.clone());

            let error = state_transition_function(&block);
            
            if error.is_err() {
                println!("****************************************************** Error: {:?}", error);
                return;
            }
            let result_state = get_global_state().lock().unwrap().clone();
            
            assert_eq!(expected_state.time, result_state.time);
            assert_eq!(expected_state.safrole, result_state.safrole);
            assert_eq!(expected_state.entropy, result_state.entropy);
            assert_eq!(expected_state.curr_validators, result_state.curr_validators);
            assert_eq!(expected_state.prev_validators, result_state.prev_validators);
            assert_eq!(expected_state.disputes.offenders, result_state.disputes.offenders);
            assert_eq!(expected_state.availability, result_state.availability);
            assert_eq!(expected_state.ready_queue, result_state.ready_queue);
            assert_eq!(expected_state.accumulation_history, result_state.accumulation_history);
            assert_eq!(expected_state.privileges, result_state.privileges);
            assert_eq!(expected_state.next_validators, result_state.next_validators);
            assert_eq!(expected_state.auth_queues, result_state.auth_queues);

            for (i, block) in expected_state.recent_history.blocks.iter().enumerate() {
                //assert_eq!(*block, state.recent_history.blocks[i]);
                assert_eq!(block.header_hash, result_state.recent_history.blocks[i].header_hash);
                assert_eq!(block.mmr, result_state.recent_history.blocks[i].mmr);
                assert_eq!(block.reported_wp, result_state.recent_history.blocks[i].reported_wp);
                assert_eq!(block.state_root, result_state.recent_history.blocks[i].state_root);
            }
            assert_eq!(expected_state.recent_history.blocks, result_state.recent_history.blocks);           

            for service_account in expected_state.service_accounts.iter() {
                if let Some(account) = result_state.service_accounts.get(&service_account.0) {
                    //assert_eq!(service_account, state.service_accounts.service_accounts.get_key_value(&service_account.0).unwrap());
                    println!("TESTING service {:?}", service_account.0);
                    //println!("Account: {:x?}", account);
                    let (items, octets, _threshold) = account.get_footprint_and_threshold();

                    for item in service_account.1.storage.iter() {
                        if let Some(value) = account.storage.get(item.0) {
                            assert_eq!(item.1, value);
                            //println!("Comparing {:x?} with {:x?}", item.1, value);
                        } else {
                            panic!("Key storage not found: {:?}", *item.0);
                        }
                    }
                    
                    println!("items: {items}, octets: {octets}");
                    assert_eq!(service_account.1.storage, account.storage);
                    assert_eq!(service_account.1.lookup, account.lookup);
                    assert_eq!(service_account.1.preimages, account.preimages);
                    assert_eq!(service_account.1.code_hash, account.code_hash);
                    assert_eq!(service_account.1.balance, account.balance);
                    assert_eq!(service_account.1.acc_min_gas, account.acc_min_gas);
                    assert_eq!(service_account.1.xfer_min_gas, account.xfer_min_gas);

                } else {
                    panic!("Service account not found in state: {:?}", service_account.0);
                }
            }
            assert_eq!(expected_state.service_accounts, result_state.service_accounts);
            assert_eq!(expected_state.auth_pools, result_state.auth_pools);
            assert_eq!(expected_state.statistics.curr, result_state.statistics.curr);
            assert_eq!(expected_state.statistics.prev, result_state.statistics.prev);
            assert_eq!(expected_state.statistics.cores, result_state.statistics.cores);
            assert_eq!(expected_state.statistics.services, result_state.statistics.services);

            /*println!("Statistics curr: {:?}", state.statistics.curr);
            println!("Statistics prev: {:?}", state.statistics.prev);
            println!("Statistics cores: {:?}", state.statistics.cores);
            println!("Statistics services: {:?}", state.statistics.services);*/

            assert_eq!(post_state.state_root, merkle_state(&result_state.serialize().map, 0).unwrap());

            //println!("pre_state: {:x?}", pre_state);
            //println!("block: {:x?}", block);
            //println!("post_state: {:x?}", post_state);

            slot += 1;
        }

    }

    fn set_raw_state(raw_state: RawState, state: &mut GlobalState) {

        for keyval in raw_state.keyvals.iter() {

            let key = keyval.key[0] & 0xFF;
            
            if key > 0 && key <= 0x0F {

                let mut reader = BytesReader::new(&keyval.value);
                
                match key {
                    AUTH_POOLS => {
                        state.auth_pools = AuthPools::decode(&mut reader).expect("Error decoding AuthPools");
                    },
                    AUTH_QUEUE => {
                        state.auth_queues = AuthQueues::decode(&mut reader).expect("Error decoding AuthQueues");
                    },
                    RECENT_HISTORY => {
                        state.recent_history = BlockHistory::decode(&mut reader).expect("Error decoding BlockHistory");
                    },
                    SAFROLE => {
                        state.safrole = Safrole::decode(&mut reader).expect("Error decoding Safrole");
                    },
                    DISPUTES => {
                        state.disputes = DisputesRecords::decode(&mut reader).expect("Error decoding Disputes");
                    },
                    ENTROPY => {
                        state.entropy = EntropyPool::decode(&mut reader).expect("Error decoding Entropy");
                    },
                    NEXT_VALIDATORS => {
                        state.next_validators = ValidatorsData::decode(&mut reader).expect("Error decoding NextValidators");
                    },
                    CURR_VALIDATORS => {
                        state.curr_validators = ValidatorsData::decode(&mut reader).expect("Error decoding CurrValidators");
                    },
                    PREV_VALIDATORS => {
                        state.prev_validators = ValidatorsData::decode(&mut reader).expect("Error decoding PrevValidators");
                    },
                    AVAILABILITY => {
                        state.availability = AvailabilityAssignments::decode(&mut reader).expect("Error decoding Availability");
                    },
                    TIME => {
                        state.time = TimeSlot::decode(&mut reader).expect("Error decoding Time");
                    },
                    PRIVILEGES => {
                        state.privileges = Privileges::decode(&mut reader).expect("Error decoding Privileges");
                    },
                    STATISTICS => {
                        state.statistics = Statistics::decode(&mut reader).expect("Error decoding Statistics");
                    },
                    READY_QUEUE => {
                        state.ready_queue = ReadyQueue::decode(&mut reader).expect("Error decoding ReadyQueue");
                    },
                    ACCUMULATION_HISTORY => {
                        state.accumulation_history = AccumulatedHistory::decode(&mut reader).expect("Error decoding AccumulationHistory");
                    },
                    _ => {
                        panic!("Key {:?} not supported", key);
                    },
                }

            } else if key == 0xFF {

                let mut service_reader = BytesReader::new(&keyval.key[1..]);
                let service_id = ServiceId::decode(&mut service_reader).expect("Error decoding service id");

                if state.service_accounts.get(&service_id).is_none() {
                    let account = Account::default();
                    state.service_accounts.insert(service_id, account);
                }
                let mut account_reader = BytesReader::new(&keyval.value);
                let account = state.service_accounts.get_mut(&service_id).unwrap();
                account.code_hash = OpaqueHash::decode(&mut account_reader).expect("Error decoding code_hash");
                account.balance = Gas::decode(&mut account_reader).expect("Error decoding balance") as u64;
                account.acc_min_gas = Gas::decode(&mut account_reader).expect("Error decoding acc_min_gas");
                account.xfer_min_gas = Gas::decode(&mut account_reader).expect("Error decoding xfer_min_gas");
                // TODO bytes and items
                //state.service_accounts.insert(service_id, account);
            } else {
                let service_id_vec = vec![keyval.key[0], keyval.key[2], keyval.key[4], keyval.key[6]];
                let service_id = decode::<ServiceId>(&service_id_vec, std::mem::size_of::<ServiceId>());
                let mut key_hash = vec![keyval.key[1], keyval.key[3], keyval.key[5]];
                key_hash.extend_from_slice(&keyval.key[7..]);

                // Preimage
                if key_hash[0] == 0xFE && key_hash[1] == 0xFF && key_hash[2] == 0xFF && key_hash[3] == 0xFF { 
                    
                    if state.service_accounts.get(&service_id).is_none() {
                        state.service_accounts.insert(service_id, Account::default());
                    }
                    let hash = sp_core::blake2_256(&keyval.value);
                    state.service_accounts.get_mut(&service_id).unwrap().preimages.insert(hash, keyval.value.clone());
                
                // Storage
                } else if key_hash[0] == 0xFF && key_hash[1] == 0xFF && key_hash[2] == 0xFF && key_hash[3] == 0xFF {
                    
                    if state.service_accounts.get(&service_id).is_none() {
                        state.service_accounts.insert(service_id, Account::default());
                    }

                    let mut storage_key  = [0u8; 32];
                    storage_key.copy_from_slice(&keyval.key);
                    state.service_accounts.get_mut(&service_id).unwrap().storage.insert(storage_key, keyval.value.clone());

                // Lookup
                } else {

                    /*if state.service_accounts.get(&service_id).is_none() {
                        state.service_accounts.insert(service_id, Account::default());
                    }*/                    
                }

                //panic!("Key {:?} not supported yet", key);
            }
        }

        for keyval in raw_state.keyvals.iter() { 
        
            // Check for preimage lookup
            if keyval.key[1] != 0xFF && keyval.key[1] != 0xFE && keyval.key[1] != 0x00 {

                let service_id_vec = vec![keyval.key[0], keyval.key[2], keyval.key[4], keyval.key[6]];
                let service_id = decode::<ServiceId>(&service_id_vec, std::mem::size_of::<ServiceId>());
                
                let length_vec = vec![keyval.key[1], keyval.key[3], keyval.key[5], keyval.key[7]];
                let length = decode::<u32>(&length_vec, std::mem::size_of::<u32>());

                let mut timeslots_reader = BytesReader::new(&keyval.value);
                let timeslots = Vec::<u32>::decode_len(&mut timeslots_reader).expect("Error decoding timeslots");

                let account = state.service_accounts.get_mut(&service_id).unwrap();

                for preimage in account.preimages.iter() {

                    if preimage.1.len() == length as usize {
                        let hash = sp_core::blake2_256(&preimage.1);
                        account.lookup.insert((hash, length), timeslots.clone());
                        println!("Lookup preimage inserted");
                    }
                }
            }
        }
    }
}
