#[cfg(test)]
mod tests {

    use std::collections::HashMap;
    use crate::FromProcessError;
    use crate::codec::{TestBody, encode_decode_test};
    use crate::test_types::{InputPreimages, PreimagesState};
    use jam_types::{Account, OutputPreimages, ServiceAccounts, Statistics, ProcessError, StateKeyType};
    use block::Extrinsic;
    use handler::{set_service_accounts, get_service_accounts, set_time, get_global_state, set_statistics, get_statistics};
    use codec::{Decode, BytesReader};
    use utils::serialization::{StateKeyTrait, construct_lookup_key, construct_preimage_key};

    impl FromProcessError for OutputPreimages {
        fn from_process_error(error: ProcessError) -> Self {
            match error {
                ProcessError::PreimagesError(code) => OutputPreimages::Err(code),
                _ => panic!("Unexpected error type in conversion"),
            }
        }
    }
    
    fn run_test(filename: &str) {

        let test_content = utils::common::read_bin_file(std::path::Path::new(&format!("jamtestvectors/preimages/data/{}", filename))).unwrap();
        let test_body: Vec<TestBody> = vec![
                                        TestBody::InputPreimages,
                                        TestBody::PreimagesState,
                                        TestBody::OutputPreimages,
                                        TestBody::PreimagesState];
        
        let _ = encode_decode_test(&test_content, &test_body);

        let mut reader = BytesReader::new(&test_content);
        let input = InputPreimages::decode(&mut reader).expect("Error decoding post Input Preimages");
        let pre_state = PreimagesState::decode(&mut reader).expect("Error decoding Preimages PreState");
        let expected_output = OutputPreimages::decode(&mut reader).expect("Error decoding post OutputPreimages");
        let expected_state = PreimagesState::decode(&mut reader).expect("Error decoding Preimages PostState");
        
        /*println!("\ninput: {:?}", input);
        println!("pre_state: {:?}", pre_state);
        println!("expected_output: {:?}", expected_output);
        println!("expected_state: {:?}", expected_state);*/

        let mut service_accounts = ServiceAccounts::default();
        for account in pre_state.accounts.iter() {
            let mut preimages_map: HashMap<[u8; 31], Vec<u8>> = HashMap::new();
            for preimage in account.data.preimages.iter() {
                let preimage_key = StateKeyType::Account(account.id, construct_preimage_key(&preimage.hash).to_vec()).construct();
                preimages_map.insert(preimage_key, preimage.blob.clone());
            }
            let mut lookup_map = HashMap::new();
            for lookup_meta in account.data.lookup_meta.iter() {     
                let mut timeslot_values = Vec::new();
                for timeslot in lookup_meta.value.iter() {
                    timeslot_values.push(timeslot.clone());
                }   
                lookup_map.insert(StateKeyType::Account(account.id, construct_lookup_key(&lookup_meta.key.hash, lookup_meta.key.length).to_vec()).construct(), timeslot_values.clone());
                //lookup_map.insert(construct_lookup_key(&lookup_meta.key.hash.clone(), lookup_meta.key.length.clone()), timeslot_values.clone());
            }
            let mut new_account = Account::default();
            new_account.preimages = preimages_map.clone();
            new_account.lookup = lookup_map.clone();
            service_accounts.insert(account.id.clone(), new_account);
        }

        set_time(input.slot.clone());
        set_service_accounts(service_accounts.clone());
        let mut statistics = Statistics::default();
        for service_stats in pre_state.statistics.iter() {
            statistics.services.records.insert(service_stats.id, service_stats.record.clone());
        }
        
        set_statistics(statistics.clone());

        let mut state = get_global_state().lock().unwrap().clone();

        let output_result = services::process(&mut state.service_accounts, &input.slot, &input.preimages);

        let mut extrinsic = Extrinsic::default();
        extrinsic.preimages = input.preimages.clone();

        statistics::process(&mut statistics, &input.slot, &0, &extrinsic, &vec![]);

        match output_result {
            Ok(_) => { 
                set_service_accounts(state.service_accounts);
                set_statistics(statistics.clone());
            },
            Err(_) => { log::error!("Error: {:?}", output_result); },
        }

        let result_service_accounts = get_service_accounts();
        for account in expected_state.accounts.iter() {
            let result_account = result_service_accounts.get(&account.id).unwrap();
            for preimage in account.data.preimages.iter() {
                let preimage_key = StateKeyType::Account(account.id, construct_preimage_key(&preimage.hash).to_vec()).construct();
                if let Some(_) = result_account.preimages.get(&preimage_key) {
                    assert_eq!(preimage.blob, *result_account.preimages.get(&preimage_key).unwrap());
                }
            }
            for lookup_meta in account.data.lookup_meta.iter() {
                let timeslot_values = result_account.lookup.get(&StateKeyType::Account(account.id, construct_lookup_key(&lookup_meta.key.hash, lookup_meta.key.length).to_vec()).construct()).unwrap();
                assert_eq!(lookup_meta.value.len(), timeslot_values.len());
                for (i, byte) in lookup_meta.value.iter().enumerate() {
                    assert_eq!(byte.clone(), timeslot_values[i].clone());
                }
            }
        }
        
        match output_result {
            Ok(OutputPreimages::Ok {  }) => {
                assert_eq!(expected_output, OutputPreimages::Ok());
            }
            Err(error) => {
                assert_eq!(expected_output, OutputPreimages::from_process_error(error));
            }
            _ => panic!("Unexpected output"),
        }

        let post_statistics = get_statistics();

        for service in expected_state.statistics.iter() {
            let result = post_statistics.services.records.get(&service.id).unwrap();
            assert_eq!(service.record, *result);
        }
                
    }

    #[test]
    fn run_preimages_tests() {
        
        println!("Preimages tests");
        dotenv::dotenv().ok();
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

        let test_files = vec![
            // Nothing is provided.
            "preimage_needed-1.bin",
            // Provide one solicited blob.
            "preimage_needed-2.bin",
            // Provide two blobs, but one of them has not been solicited.
            "preimage_not_needed-1.bin",
            // Provide two blobs, but one of them has already been provided.
            "preimage_not_needed-2.bin",
            // Bad order of services.
            "preimages_order_check-1.bin",
            // Bad order of images for a service.
            "preimages_order_check-2.bin",
            // Order is correct.
            "preimages_order_check-3.bin",
            // Duplicate item.
            "preimages_order_check-4.bin",
        ];
        for file in test_files {
            log::info!("");
            log::info!("Running test: {}", file);
            run_test(file);
        }
    }
}