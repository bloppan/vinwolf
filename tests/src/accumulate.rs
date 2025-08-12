#[cfg(test)]
mod tests {

    use once_cell::sync::Lazy;
    use crate::codec::tests::{TestBody, encode_decode_test};
    use crate::test_types::{InputAccumulate, StateAccumulate};
    use jam_types::{EntropyPool, Extrinsic, ServiceAccounts, OutputAccumulation, Account, Statistics, StateKeyType, ValidatorSet};
    use constants::node::{VALIDATORS_COUNT, EPOCH_LENGTH};
    use state_handler::{get_global_state};
    use codec::{Decode, BytesReader};
    use utils::serialization::{StateKeyTrait, construct_preimage_key, construct_lookup_key, construct_storage_key};

    static TEST_TYPE: Lazy<&'static str> = Lazy::new(|| {
        if VALIDATORS_COUNT == 6 && EPOCH_LENGTH == 12 {
            "tiny"
        } else if VALIDATORS_COUNT == 1023 && EPOCH_LENGTH == 600 {
            "full"
        } else {
            panic!("Invalid configuration for tiny nor full tests");
        }
    });

    fn run_test(filename: &str) {
        
        let test_content = utils::common::read_bin_file(std::path::Path::new(&format!("jamtestvectors/accumulate/{}/{}", *TEST_TYPE, filename))).unwrap();
        let test_body: Vec<TestBody> = vec![
                                        TestBody::InputAccumulate,
                                        TestBody::StateAccumulate,
                                        TestBody::OutputAccumulation,
                                        TestBody::StateAccumulate];
        
        let _ = encode_decode_test(&test_content, &test_body);

        let mut reader = BytesReader::new(&test_content);
        let input = InputAccumulate::decode(&mut reader).expect("Error decoding InputAccumulate");
        let pre_state = StateAccumulate::decode(&mut reader).expect("Error decoding Accumulate PreState");
        let expected_output = OutputAccumulation::decode(&mut reader).expect("Error decoding OutputAccumulate");
        let expected_state = StateAccumulate::decode(&mut reader).expect("Error decoding Accumulate PostState");
                
        
        let mut statistics = Statistics::default();
        statistics.services = pre_state.statistics;
        state_handler::statistics::set(statistics);
        let mut entropy = EntropyPool::default();
        entropy.buf[0].entropy = pre_state.entropy.clone();
        state_handler::entropy::set(entropy);
        state_handler::entropy::set_recent(pre_state.entropy.clone());
        state_handler::time::set(pre_state.slot.clone());
        state_handler::time::set_current(&input.slot);
        state_handler::ready_queue::set(pre_state.ready.clone());
        state_handler::acc_history::set(pre_state.accumulated.clone());
        state_handler::privileges::set(pre_state.privileges.clone());
        
        let mut service_accounts = ServiceAccounts::default();
        for account in pre_state.accounts.iter() {
            let mut new_account = Account::default();
            new_account.balance = account.data.service.balance.clone();
            new_account.code_hash = account.data.service.code_hash.clone();
            new_account.acc_min_gas = account.data.service.acc_min_gas.clone();
            new_account.xfer_min_gas = account.data.service.xfer_min_gas.clone();
            new_account.created_at = account.data.service.created_at;
            new_account.gratis_storage_offset = account.data.service.gratis_storage_offset;
            new_account.items = account.data.service.items;
            new_account.last_acc = account.data.service.last_acc;
            new_account.octets = account.data.service.octets;
            new_account.parent_service = account.data.service.parent_service;
            for preimage in account.data.preimages.iter() {
                let preimage_key = StateKeyType::Account(account.id, construct_preimage_key(&preimage.hash)).construct();
                new_account.storage.insert(preimage_key, preimage.blob.clone());
            }
            for storage in account.data.storage.iter() {
                let storage_key = StateKeyType::Account(account.id, construct_storage_key(&storage.key)).construct();
                new_account.storage.insert(storage_key, storage.value.clone());
            }
            service_accounts.insert(account.id.clone(), new_account);
        }

        state_handler::service_accounts::set(service_accounts.clone());

        let mut state = get_global_state().lock().unwrap().clone();

        /*let (acc_root, 
         service_accounts, 
         next_validators, 
         queue_auth, 
         privileges) = accumulation::process(
                                                        &mut state.accumulation_history,
                                                        &mut state.ready_queue,
                                                        state.service_accounts,
                                                        state.next_validators,
                                                        state.auth_queues,
                                                        state.privileges,
                                                        &input.slot,
                                                        &input.reports)?;*/

        match accumulation::process(
                                &mut state.accumulation_history,
                                &mut state.ready_queue,
                                state.service_accounts,
                                state.next_validators,
                                state.auth_queues,
                                state.privileges,
                                &input.slot,
                                &input.reports) 
        {
            Ok((acc_root, service_accounts, next_validators, auth_queues, privileges)) => {
                state_handler::acc_history::set(state.accumulation_history.clone());
                state_handler::ready_queue::set(state.ready_queue.clone());
                state_handler::service_accounts::set(service_accounts.clone());
                state_handler::validators::set(next_validators, ValidatorSet::Next);
                state_handler::auth_queues::set(auth_queues.clone());
                state_handler::privileges::set(privileges.clone());

                statistics::process(
                                &mut state.statistics, 
                                &input.slot, 
                                &0, 
                                &Extrinsic::default(),
                                &input.reports,
                            );
                state_handler::statistics::set(state.statistics.clone());

                assert_eq!(expected_output, OutputAccumulation::Ok(acc_root)); 
            },
            Err(_) => { },
        }
        
        let result_state = get_global_state().lock().unwrap().clone();

        assert_eq!(expected_state.accumulated, result_state.accumulation_history);
        assert_eq!(expected_state.ready, result_state.ready_queue);
        //assert_eq!(expected_state.entropy, result_state.entropy.buf[0]);
        //assert_eq!(expected_state.slot, result_state.time);
        assert_eq!(expected_state.privileges, result_state.privileges);
        
        for account in expected_state.accounts.iter() {
            let result_account = result_state.service_accounts.get(&account.id).unwrap();
            assert_eq!(account.data.service.balance, result_account.balance);
            assert_eq!(account.data.service.code_hash, result_account.code_hash);
            assert_eq!(account.data.service.acc_min_gas, result_account.acc_min_gas);
            assert_eq!(account.data.service.xfer_min_gas, result_account.xfer_min_gas);
            assert_eq!(account.data.service.created_at, result_account.created_at);
            assert_eq!(account.data.service.gratis_storage_offset, result_account.gratis_storage_offset);
            assert_eq!(account.data.service.items, result_account.items);
            assert_eq!(account.data.service.last_acc, result_account.last_acc);
            assert_eq!(account.data.service.octets, result_account.octets);
            assert_eq!(account.data.service.parent_service, result_account.parent_service);
            
            for preimage in account.data.preimages.iter() {
                let preimage_key = StateKeyType::Account(account.id, construct_preimage_key(&preimage.hash)).construct();
                assert_eq!(&preimage.blob, result_account.storage.get(&preimage_key).unwrap());
            }
            for storage in account.data.storage.iter() {
                let storage_key = StateKeyType::Account(account.id, construct_storage_key(&storage.key)).construct();
                assert_eq!(&storage.value, result_account.storage.get(&storage_key).unwrap());
            }
        }

        assert_eq!(expected_state.statistics, result_state.statistics.services);
    }

    #[test]
    fn run_accumulate_test() {

        dotenv::dotenv().ok();
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
        log::info!("Accumulate tests in {} mode", *TEST_TYPE);

        let test_files = vec![
            // No reports.
            "no_available_reports-1.bin",
            // Report with no dependencies.
            "process_one_immediate_report-1.bin",
            // Report with unsatisfied dependency added to the ready queue.
            "enqueue_and_unlock_simple-1.bin",
            // Report with no dependencies that resolves previous dependency.
            "enqueue_and_unlock_simple-2.bin",
            // Report with unsatisfied segment tree root dependency added to the ready queue.
            "enqueue_and_unlock_with_sr_lookup-1.bin",
            // Report with no dependencies that resolves previous dependency.
            "enqueue_and_unlock_with_sr_lookup-2.bin",
            // Two reports with unsatisfied dependencies added to the ready queue.
            "enqueue_and_unlock_chain-1.bin",
            // Two additional reports with unsatisfied dependencies added to the ready queue.
            "enqueue_and_unlock_chain-2.bin",
            // Two additional reports. One with unsatisfied dependencies, thus added to the ready queue.
            // One report is accumulated and resolves two previously enqueued reports.
            "enqueue_and_unlock_chain-3.bin",
            // Report that resolves all remaining queued dependencies.
            "enqueue_and_unlock_chain-4.bin",                                 
            // Two reports with unsatisfied dependencies added to the ready queue.
            "enqueue_and_unlock_chain_wraps-1.bin",
            // Two additional reports, one with no dependencies and thus immediately accumulated.
            // The other is pushed to the ready queue which fills up and wraps around (ready queue is a ring buffer).
            "enqueue_and_unlock_chain_wraps-2.bin",
            // Two additional reports with unsatisfied dependencies pushed to the ready queue.
            "enqueue_and_unlock_chain_wraps-3.bin",
            // Two additional reports, one with no dependencies and thus immediately accumulated.
            // Three old entries in the ready queue are removed.
            "enqueue_and_unlock_chain_wraps-4.bin",
            // Report with no dependencies resolves all previous enqueued reports.
            "enqueue_and_unlock_chain_wraps-5.bin",
            // Report with direct dependency on itself.
            // This makes the report stale, but pushed to the ready queue anyway.
            "enqueue_self_referential-1.bin",
            // Two reports with indirect circular dependency.
            // This makes the reports stale, but pushed to the ready queue anyway.
            "enqueue_self_referential-2.bin",
            // Two reports. First depends on second, which depends on unseen report.
            "enqueue_self_referential-3.bin",
            // New report creates a cycle with the previously queued reports.
            // This makes the reports stale, but pushed to the ready queue anyway.
            "enqueue_self_referential-4.bin",
            // There are some reports in the ready-queue ready to be accumulated.
            // Even though we don't supply any new available work report these are processed.
            // This condition may result because of gas exhausition during previous block execution.
            "accumulate_ready_queued_reports-1.bin",
            // Check that ready-queue and accumulated-reports queues are shifted.
            // A new available report is supplied.
            "queues_are_shifted-1.bin",
            // Check that ready-queue and accumulated-reports queues are shifted.
            // No new report is supplied.
            "queues_are_shifted-2.bin",
            // Two reports with unsatisfied dependencies added to the ready-queue.
            "ready_queue_editing-1.bin",
            // Two reports with unsatisfied dependencies added to the ready-queue.
            // One accumulated. Ready queue items dependencies are edited.
            "ready_queue_editing-2.bin",
            // One report unlocks reports in the ready-queue.
            "ready_queue_editing-3.bin",
            "same_code_different_services-1.bin",
        ];
        for file in test_files {
            log::info!("");
            log::info!("Running test: {}", file);
            log::info!("");
            run_test(file);
        }

    }

}