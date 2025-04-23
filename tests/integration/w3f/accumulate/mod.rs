use once_cell::sync::Lazy;
use crate::integration::w3f::read_test_file;
use crate::integration::w3f::codec::{TestBody, encode_decode_test};

pub mod codec;
use codec::{InputAccumulate, StateAccumulate};

use vinwolf::types::{EntropyPool, ServiceAccounts, OutputAccumulation, Account};
use vinwolf::constants::{VALIDATORS_COUNT, EPOCH_LENGTH};
use vinwolf::blockchain::state::{
    set_service_accounts, set_entropy, set_time, get_global_state, set_accumulation_history, set_privileges, set_ready_queue
};
use vinwolf::blockchain::state::accumulation::process_accumulation;
use vinwolf::utils::codec::{Decode, BytesReader};

extern crate vinwolf;

static TEST_TYPE: Lazy<&'static str> = Lazy::new(|| {
    if VALIDATORS_COUNT == 6 && EPOCH_LENGTH == 12 {
        "tiny"
    } else if VALIDATORS_COUNT == 1023 && EPOCH_LENGTH == 600 {
        "full"
    } else {
        panic!("Invalid configuration for tiny nor full tests");
    }
});

#[cfg(test)]
mod tests {

    use super::*;

    fn run_test(filename: &str) {
      
        let test_content = read_test_file(&format!("tests/test_vectors/w3f/jamtestvectors/accumulate/{}/{}", *TEST_TYPE, filename));
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
        
        let mut entropy = EntropyPool::default();
        entropy.buf[0] = pre_state.entropy;
        set_entropy(entropy);
        set_time(pre_state.slot.clone());
        set_ready_queue(pre_state.ready.clone());
        set_accumulation_history(pre_state.accumulated.clone());
        set_privileges(pre_state.privileges.clone());

        let mut service_accounts = ServiceAccounts::default();
        for account in pre_state.accounts.iter() {
            let mut new_account = Account::default();
            new_account.balance = account.data.service.balance.clone();
            new_account.code_hash = account.data.service.code_hash.clone();
            new_account.gas = account.data.service.min_item_gas.clone();
            new_account.min_gas = account.data.service.min_memo_gas.clone();
            for preimage in account.data.preimages.iter() {
                new_account.preimages.insert(preimage.hash.clone(), preimage.blob.clone());
            }
            service_accounts.service_accounts.insert(account.id.clone(), new_account);
        }

        set_service_accounts(service_accounts.clone());

        let mut state = get_global_state().lock().unwrap().clone();

        /*let output_accumulation = process_accumulation(
            &mut state.accumulation_history,
            &mut state.ready_queue,
            /*&state.entropy,
            &state.privileges,
            &state.service_accounts,*/
            &input.slot,
            &input.reports);

        match output_accumulation {
            Ok(_) => { 
                set_accumulation_history(state.accumulation_history.clone());
                set_ready_queue(state.ready_queue.clone());
            },
            Err(_) => { },
        }*/
        
        let result_state = get_global_state().lock().unwrap().clone();

        assert_eq!(expected_state.accumulated, result_state.accumulation_history);
        assert_eq!(expected_state.ready, result_state.ready_queue);
        assert_eq!(expected_state.entropy, result_state.entropy.buf[0]);
        //assert_eq!(expected_state.slot, result_state.time);
        assert_eq!(expected_state.privileges, result_state.privileges);
        
        for account in expected_state.accounts.iter() {
            let result_account = result_state.service_accounts.service_accounts.get(&account.id).unwrap();
            assert_eq!(account.data.service.balance, result_account.balance);
            assert_eq!(account.data.service.code_hash, result_account.code_hash);
            assert_eq!(account.data.service.min_item_gas, result_account.gas);
            assert_eq!(account.data.service.min_memo_gas, result_account.min_gas);
            for preimage in account.data.preimages.iter() {
                assert_eq!(&preimage.blob, result_account.preimages.get(&preimage.hash).unwrap());
            }
        }

        /*match output_accumulation { // TODO arreglar esto
            Ok(accumulation_root) => {
                assert_eq!(expected_output, OutputAccumulation([0u8; 32]));
            },
            Err(_) => {
                
            },
        }*/

    }

    #[test]
    fn run_accumulate_test() {

        println!("Accumulate tests in {} mode", *TEST_TYPE);

        let test_files = vec![
            // No reports.
            /*"no_available_reports-1.bin",
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
            "ready_queue_editing-2.bin",*/
            // One report unlocks reports in the ready-queue.
            "ready_queue_editing-3.bin",
        ];
        for file in test_files {
            println!("Running test: {}", file);

            run_test(file);
        }

    }

}