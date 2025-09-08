#[cfg(test)]
mod tests {

    use std::sync::LazyLock;
    use crate::{codec::tests::{TestBody, encode_decode_test}, FromProcessError, test_types::{InputAssurances, StateAssurances}};
    use std::path::Path;
    use constants::node::{CORES_COUNT, VALIDATORS_COUNT};
    use jam_types::{OutputDataAssurances, OutputAssurances, ValidatorSet, ProcessError};
    use state_handler::{get_global_state};
    use codec::{Decode, BytesReader};
    use utils::log;
    
    static TEST_TYPE: LazyLock<&'static str> = LazyLock::new(|| {
        if VALIDATORS_COUNT == 6 && CORES_COUNT == 2 {
            "tiny"
        } else if VALIDATORS_COUNT == 1023 && CORES_COUNT == 341 {
            "full"
        } else {
            panic!("Invalid configuration for tiny nor full tests");
        }
    });

    impl FromProcessError for OutputAssurances {
        fn from_process_error(error: ProcessError) -> Self {
            match error {
                ProcessError::AssurancesError(code) => OutputAssurances::Err(code),
                _ => panic!("Unexpected error type in conversion"),
            }
        }
    }
    
    fn run_test(filename: &str) {

        let test_content = utils::common::read_bin_file(Path::new(&format!("jamtestvectors/assurances/{}/{}", *TEST_TYPE, filename))).unwrap();
        let test_body: Vec<TestBody> = vec![
                                        TestBody::InputAssurances,
                                        TestBody::StateAssurances,
                                        TestBody::OutputAssurances,
                                        TestBody::StateAssurances];
        
        let _ = encode_decode_test(&test_content, &test_body);
        
        let mut reader = BytesReader::new(&test_content);
        let input = InputAssurances::decode(&mut reader).expect("Error decoding post InputAssurances");
        let pre_state = StateAssurances::decode(&mut reader).expect("Error decoding post Assurances PreState");
        let expected_output = OutputAssurances::decode(&mut reader).expect("Error decoding post OutputAssurances");
        let expected_state = StateAssurances::decode(&mut reader).expect("Error decoding post Assurances PostState");
            
        state_handler::reports::set(pre_state.avail_assignments);
        state_handler::validators::set(pre_state.curr_validators, ValidatorSet::Current);
  
        let current_state = get_global_state().lock().unwrap().clone();
        let mut assurances_state = current_state.availability.clone();

        let output_result = reports::assurances::process(
                                                                            &mut assurances_state, 
                                                                            &input.assurances, 
                                                                            &input.slot,
                                                                            &input.parent);
        
        match output_result {
            Ok(_) => { state_handler::reports::set(assurances_state);},
            Err(_) => { },
        }
        let result_avail_assignments = state_handler::reports::get();
        let result_curr_validators = state_handler::validators::get(ValidatorSet::Current);
        
        assert_eq!(expected_state.avail_assignments, result_avail_assignments);
        assert_eq!(expected_state.curr_validators, result_curr_validators);
        
        match output_result {
            Ok(OutputDataAssurances { reported}) => {
                assert_eq!(expected_output, OutputAssurances::Ok(OutputDataAssurances {reported}));
            }
            Err(error_code) => {
                assert_eq!(expected_output, OutputAssurances::from_process_error(error_code));
            }
        }
    }

    #[test]
    fn run_assurances_tests() {
        
        log::Builder::from_env(log::Env::default().default_filter_or("debug"))
        .with_dotenv(true)
        .init();

        log::info!("Assurances tests in {} mode", *TEST_TYPE);

        let test_files = vec![
            // Progress with an empty assurances extrinsic.
            "no_assurances-1.bin",
            // Several assurances contributing to establishing availability supermajority for some of the cores.
            "some_assurances-1.bin",
            // Progress with an empty assurances extrinsic.
            // Stale work report assignment is removed (but not returned in the output).
            //"no_assurances_with_stale_report-1.bin", // TODO Este creo que lo van a quitar porque han quitado lo de los timeouts
            // One assurance has a bad signature.
            "assurances_with_bad_signature-1.bin",
            // One assurance has a bad validator index.
            "assurances_with_bad_validator_index-1.bin",
            // One assurance targets a core without any assigned work report.
            "assurance_for_not_engaged_core-1.bin",
            // One assurance has a bad attestation parent hash.
            "assurance_with_bad_attestation_parent-1.bin",
            // One assurance targets a core with a stale report.
            // We are lenient on the stale report as far as it is available.
            "assurances_for_stale_report-1.bin",
            // Assurers not sorted.
            "assurers_not_sorted_or_unique-1.bin",
            // Duplicate assurer.
            "assurers_not_sorted_or_unique-2.bin",
        ];
        for file in test_files {
            log::info!("");
            log::info!("Running test: {}", file);
            run_test(file);
        }
    }
}