#[cfg(test)]
mod tests {
    
    use jam_types::{Block, RawState, GlobalState};
    use state_handler::{get_global_state, set_global_state};
    use state_controller::state_transition_function;
    use codec::{Decode, BytesReader};
    use utils::{common::parse_state_keyvals, serialization};
    use utils::trie::merkle_state;

    /*#[derive(Debug, Clone, PartialEq)]
    pub struct TestCase {

        pub pre_state: RawState,
        pub block: Block,
        pub post_state: RawState,
    }*/

    #[test]
    fn run_traces_tests() {

        use dotenv::dotenv;
        dotenv().ok();
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

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

            log::info!("\n\nProcess trace test file: {}\n", slot);

            let test_content = utils::common::read_bin_file(std::path::Path::new(&format!("jamtestvectors/traces/reports-l1/{:08}.bin", slot))).unwrap();
            let mut reader = BytesReader::new(&test_content);
            let pre_state = RawState::decode(&mut reader).expect("Error decoding post WorkReport PreState");
            let block = Block::decode(&mut reader).expect("Error decoding post OutputWorkReport");
            let post_state = RawState::decode(&mut reader).expect("Error decoding post WorkReport PostState");

            let mut state = GlobalState::default();
            let mut expected_state = GlobalState::default();

            parse_state_keyvals(&pre_state.keyvals, &mut state).expect("Error decoding pre state keyvals");
            parse_state_keyvals(&post_state.keyvals, &mut expected_state).expect("Error decoding post state keyvals");

            assert_eq!(pre_state.state_root.clone(), merkle_state(&serialization::serialize(&state).map, 0));
            assert_eq!(post_state.state_root.clone(), merkle_state(&serialization::serialize(&expected_state).map, 0));

            set_global_state(state.clone());

            let error = state_transition_function(&block);
            
            if error.is_err() {
                log::error!("****************************************************** Error: {:?}", error);
                return;
            }

            let result_state = get_global_state().lock().unwrap().clone();
            
            assert_eq_state(&expected_state, &result_state);

            /*log::info!("post_sta state_root: 0x{}", hex::encode(post_state.state_root));
            log::info!("expected state_root: 0x{}", hex::encode(merkle_state(&expected_state.serialize().map, 0).unwrap()));
            log::info!("result   state_root: 0x{}", hex::encode(merkle_state(&result_state.serialize().map, 0).unwrap()));*/
            
            assert_eq!(post_state.state_root, merkle_state(&serialization::serialize(&result_state).map, 0));

            slot += 1;

            if slot == 101 {
                return;
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
        for service_account in expected_state.service_accounts.iter() {
            if let Some(account) = result_state.service_accounts.get(&service_account.0) {
                let (_items, _octets, _threshold) = utils::common::get_footprint_and_threshold(&account);
                for item in service_account.1.storage.iter() {
                    if let Some(value) = account.storage.get(item.0) {
                        assert_eq!(item.1, value);
                    } else {
                        panic!("Key storage not found : {:x?}", *item.0);
                    }
                }
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
    }
}