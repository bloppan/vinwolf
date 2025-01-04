/*use once_cell::sync::Lazy;
use crate::integration::{read_test_file, FromProcessError};
use crate::integration::codec::{TestBody, encode_decode_test};

pub mod codec;
use codec::{InputAssurances, StateAssurances};

use vinwolf::constants::{CORES_COUNT, VALIDATORS_COUNT};
use vinwolf::types::{OutputDataAssurances, OutputAssurances};
use vinwolf::blockchain::state::{ProcessError, get_global_state, set_reporting_assurance, get_reporting_assurance, set_validators, get_validators};
use vinwolf::blockchain::state::validators::ValidatorSet;
use vinwolf::blockchain::state::reporting_assurance::process_assurances;
use vinwolf::utils::codec::{Decode, BytesReader};

static TEST_TYPE: Lazy<&'static str> = Lazy::new(|| {
    if VALIDATORS_COUNT == 6 && CORES_COUNT == 2 {
        "tiny"
    } else if VALIDATORS_COUNT == 1023 && CORES_COUNT == 341 {
        "full"
    } else {
        panic!("Invalid configuration for tiny nor full tests");
    }
});

#[cfg(test)]
mod tests {

    use super::*;

    impl FromProcessError for OutputAssurances {
        fn from_process_error(error: ProcessError) -> Self {
            match error {
                ProcessError::AssurancesError(code) => OutputAssurances::Err(code),
                _ => panic!("Unexpected error type in conversion"),
            }
        }
    }
    
    fn run_test(filename: &str) {

        let test_content = read_test_file(&format!("tests/jamtestvectors/assurances/{}/{}", *TEST_TYPE, filename));
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
        
        set_reporting_assurance(pre_state.avail_assignments);
        set_validators(pre_state.curr_validators, ValidatorSet::Current);
  
        let current_state = get_global_state().lock().unwrap();
        let mut assurances_state = current_state.availability.clone();

        let output_result = process_assurances(
                                                                            &mut assurances_state, 
                                                                            &input.assurances, 
                                                                            &input.slot,
                                                                            &input.parent);
        
        match output_result {
            Ok(_) => { set_reporting_assurance(assurances_state);},
            Err(_) => { },
        }

        let result_avail_assignments = get_reporting_assurance();
        let result_curr_validators = get_validators(ValidatorSet::Current);

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
    fn run_preimages_tests() {
        
        println!("Assurances tests in {} mode", *TEST_TYPE);

        let test_files = vec![
            "no_assurances-1.bin",

        ];
        for file in test_files {
            println!("Running test: {}", file);
            run_test(file);
        }
    }
}*/