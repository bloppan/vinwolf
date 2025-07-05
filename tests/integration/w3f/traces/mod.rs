use crate::integration::w3f::read_test_file;

use vinwolf::constants::{*};
use vinwolf::types::{
    RawState, Block, AuthPools, AuthQueues, BlockHistory, Safrole, DisputesRecords, EntropyPool, ValidatorsData, AvailabilityAssignments,
    Privileges, Statistics, ReadyQueue, AccumulatedHistory, OpaqueHash, Gas, ServiceId, Account, KeyValue
};
use vinwolf::blockchain::state::{get_global_state, state_transition_function};
use vinwolf::utils::codec::{Decode, DecodeLen, BytesReader};
use vinwolf::utils::codec::generic::decode;
use vinwolf::utils::trie::merkle_state;
use vinwolf::{blockchain::state::set_global_state, types::{GlobalState, TimeSlot}};

pub mod codec;

#[derive(Debug, Clone, PartialEq)]
pub struct TestCase {

    pub pre_state: RawState,
    pub block: Block,
    pub post_state: RawState,
}


#[cfg(test)]
mod tests {
    
    use super::*;

    #[test]
    fn run_traces_tests() {

        /*let test_body: Vec<TestBody> = vec![TestBody::RawState,
                                            TestBody::Block,
                                            TestBody::RawState];

        let test_content = read_test_file(&format!("tests/test_vectors/w3f/jamtestvectors/traces/reports-l0/00000000.bin"));
        let _ = encode_decode_test(&test_content, &test_body);
        let mut reader = BytesReader::new(&test_content);
        let pre_state = RawState::decode(&mut reader).expect("Error decoding post WorkReport PreState");
        let block = Block::decode(&mut reader).expect("Error decoding post OutputWorkReport");
        let post_state = RawState::decode(&mut reader).expect("Error decoding post WorkReport PostState");*/
        
        let mut slot = 1;
        
        loop {
            println!("\n\n**********************    Reading trace test file: {}    **********************************", slot);

            let test_content = read_test_file(&format!("tests/test_vectors/w3f/jamtestvectors/traces/fallback/{:08}.bin", slot));
            let mut reader = BytesReader::new(&test_content);
            let pre_state = RawState::decode(&mut reader).expect("Error decoding post WorkReport PreState");
            let block = Block::decode(&mut reader).expect("Error decoding post OutputWorkReport");
            let post_state = RawState::decode(&mut reader).expect("Error decoding post WorkReport PostState");

            let mut state = GlobalState::default();
            let mut expected_state = GlobalState::default();

            set_raw_state(pre_state.clone(), &mut state);
            set_raw_state(post_state.clone(), &mut expected_state);

            assert_eq!(pre_state.state_root.clone(), merkle_state(&state.serialize().map, 0).unwrap());
            assert_eq!(post_state.state_root.clone(), merkle_state(&expected_state.serialize().map, 0).unwrap());

            set_global_state(state.clone());

            let error = state_transition_function(&block);
            
            if error.is_err() {
                println!("****************************************************** Error: {:?}", error);
                return;
            }

            let result_state = get_global_state().lock().unwrap().clone();
            
            assert_eq_state(&expected_state, &result_state);

            println!("post_sta state_root: {:x?}", post_state.state_root);
            println!("expected state_root: {:x?}", merkle_state(&expected_state.serialize().map, 0).unwrap());
            println!("result   state_root: {:x?}", merkle_state(&result_state.serialize().map, 0).unwrap());
            
            assert_eq!(post_state.state_root, merkle_state(&result_state.serialize().map, 0).unwrap());

            slot += 1;

            if slot == 101 {
                return;
            }
        }

    }

    fn set_raw_state(raw_state: RawState, state: &mut GlobalState) {

        for keyval in raw_state.keyvals.iter() {
            
            if is_simple_key(keyval) {

                let mut reader = BytesReader::new(&keyval.value);
                let key = keyval.key[0] & 0xFF;

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

            } else if is_service_info_key(keyval) {

                let mut service_reader = BytesReader::new(&keyval.key[1..]);
                let service_id = ServiceId::decode(&mut service_reader).expect("Error decoding service id");

                if state.service_accounts.get(&service_id).is_none() {
                    let account = Account::default();
                    state.service_accounts.insert(service_id, account);
                }
                let mut account_reader = BytesReader::new(&keyval.value);
                let mut account = state.service_accounts.get(&service_id).unwrap().clone();
                account.code_hash = OpaqueHash::decode(&mut account_reader).expect("Error decoding code_hash");
                account.balance = Gas::decode(&mut account_reader).expect("Error decoding balance") as u64;
                account.acc_min_gas = Gas::decode(&mut account_reader).expect("Error decoding acc_min_gas");
                account.xfer_min_gas = Gas::decode(&mut account_reader).expect("Error decoding xfer_min_gas");
                // TODO bytes and items
                state.service_accounts.insert(service_id, account);
            } else {
                let service_id_vec = vec![keyval.key[0], keyval.key[2], keyval.key[4], keyval.key[6]];
                let service_id = decode::<ServiceId>(&service_id_vec, std::mem::size_of::<ServiceId>());
                let mut key_hash = vec![keyval.key[1], keyval.key[3], keyval.key[5]];
                key_hash.extend_from_slice(&keyval.key[7..]);

                // Preimage
                if is_preimage_key(keyval) { 
                    
                    if state.service_accounts.get(&service_id).is_none() {
                        state.service_accounts.insert(service_id, Account::default());
                    }
                    let hash = sp_core::blake2_256(&keyval.value);
                    state.service_accounts.get_mut(&service_id).unwrap().preimages.insert(hash, keyval.value.clone());
                    /*println!("preimage key: {:x?}", hash);
                    println!("preimage len: {:?}", keyval.value.len());
                    println!("----------------------------------------------------------------------");*/

                // Storage
                } else if is_storage_key(keyval) {
                    
                    if state.service_accounts.get(&service_id).is_none() {
                        state.service_accounts.insert(service_id, Account::default());
                    }

                    let mut storage_key  = [0u8; 31];
                    storage_key.copy_from_slice(&keyval.key);
                    //println!("insert to service: {:?} storage key: {:x?}", service_id, storage_key);
                    //println!("insert value: {:x?}", keyval.value);
                    state.service_accounts.get_mut(&service_id).unwrap().storage.insert(storage_key, keyval.value.clone());
                    /*println!("storage key: {:x?}", storage_key);
                    println!("storage val: {:x?}", keyval.value);
                    println!("----------------------------------------------------------------------");*/

                // Lookup
                } else {
                    let service_id_vec = vec![keyval.key[0], keyval.key[2], keyval.key[4], keyval.key[6]];
                    let service_id = decode::<ServiceId>(&service_id_vec, std::mem::size_of::<ServiceId>());
                    
                    let mut timeslots_reader = BytesReader::new(&keyval.value);
                    let timeslots = Vec::<u32>::decode_len(&mut timeslots_reader).expect("Error decoding timeslots");
                    
                    if state.service_accounts.get(&service_id).is_none() {
                        state.service_accounts.insert(service_id, Account::default());
                    }
                    
                    let account = state.service_accounts.get_mut(&service_id).unwrap();
                    account.lookup.insert(keyval.key, timeslots.clone());
                }
            }
        }
    }

    fn assert_eq_state(expected_state: &GlobalState, result_state: &GlobalState) {
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
        assert_eq!(expected_state.recent_history.blocks, result_state.recent_history.blocks);           
        //assert_eq!(expected_state.service_accounts, result_state.service_accounts);
        for service_account in expected_state.service_accounts.iter() {
            if let Some(account) = result_state.service_accounts.get(&service_account.0) {
                //assert_eq!(service_account.1, account);
                println!("TESTING service {:?}", service_account.0);
                //println!("Account: {:x?}", account);
                let (_items, _octets, _threshold) = account.get_footprint_and_threshold();
                for item in service_account.1.storage.iter() {
                    if let Some(value) = account.storage.get(item.0) {
                        assert_eq!(item.1, value);
                        //println!("storage Key {:x?} ", item.0);
                    } else {
                        panic!("Key storage not found : {:x?}", *item.0);
                    }
                }

                assert_eq!(service_account.1.storage, account.storage);
                //println!("items: {items}, octets: {octets}");
                /*println!("Lookup expected");
                for item in expected_state.service_accounts.get(&service_account.0).unwrap().lookup.iter() {
                    println!("{:x?} | {:?}", item.0, item.1);
                }
                println!("Lookup result");
                for item in account.lookup.iter() {
                    println!("{:x?} | {:?}", item.0, item.1);
                }
                println!("Lookup pre_state");
                for item in test_state.service_accounts.get(&service_account.0).unwrap().lookup.iter() {
                    println!("{:x?} | {:?}", item.0, item.1);
                }

                assert_eq!(service_account.1.lookup, account.lookup);*/
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
    }

    fn is_simple_key(keyval: &KeyValue) -> bool {

        keyval.key[0] <= 0x0F && keyval.key[1..].iter().all(|&b| b == 0)
    }

    fn is_service_info_key(keyval: &KeyValue) -> bool {

        keyval.key[0] == 0xFF && keyval.key[1..].iter().all(|&b| b == 0)
    }

    fn is_storage_key(keyval: &KeyValue) -> bool {

        keyval.key[1] == 0xFF && keyval.key[3] == 0xFF && keyval.key[5] == 0xFF && keyval.key[7] == 0xFF
    }

    fn is_preimage_key(keyval: &KeyValue) -> bool {

        keyval.key[1] == 0xFE && keyval.key[3] == 0xFF && keyval.key[5] == 0xFF && keyval.key[7] == 0xFF
    }

    /*fn is_lookup_key(keyval: &KeyValue) -> bool {
        
        !is_simple_key(keyval) && !is_service_info_key(keyval) && !is_storage_key(keyval) && !is_preimage_key(keyval)
    }*/
}