use once_cell::sync::Lazy;
use crate::integration::read_test_file;
use crate::integration::codec::{TestBody, encode_decode_test};

pub mod schema;
use schema::{InputAuthorizations, StateAuthorizations};

use vinwolf::constants::CORES_COUNT;
use vinwolf::blockchain::block::extrinsic::assurances::{OutputDataAssurances, OutputAssurances};
use vinwolf::blockchain::state::{get_global_state, set_reporting_assurance_state, get_reporting_assurance_state};
use vinwolf::blockchain::state::validators::{set_validators_state, get_validators_state, ValidatorSet};
use vinwolf::blockchain::state::reporting_assurance::process_assurances;
use vinwolf::utils::codec::{Decode, BytesReader};

static TEST_TYPE: Lazy<&'static str> = Lazy::new(|| {
    if CORES_COUNT == 2 {
        "tiny"
    } else if CORES_COUNT == 341 {
        "full"
    } else {
        panic!("Invalid configuration for tiny nor full tests");
    }
});


#[cfg(test)]
mod tests {

    use super::*;

    fn run_test(filename: &str) {

        let test_content = read_test_file(&format!("tests/jamtestvectors/authorizations/{}/{}", *TEST_TYPE, filename));
        let test_body: Vec<TestBody> = vec![
                                        TestBody::InputAuthorizations,
                                        TestBody::StateAuthorizations,
                                        TestBody::StateAuthorizations];
        
        let _ = encode_decode_test(&test_content, &test_body);

        /*let mut reader = BytesReader::new(&test_content);
        let input = InputAssurances::decode(&mut reader).expect("Error decoding post InputWorkReport");
        let pre_state = StateAssurances::decode(&mut reader).expect("Error decoding post WorkReport PreState");
        let expected_output = OutputAssurances::decode(&mut reader).expect("Error decoding post OutputWorkReport");
        let expected_state = StateAssurances::decode(&mut reader).expect("Error decoding post WorkReport PostState");
        
        set_reporting_assurance_state(&pre_state.avail_assignments);
        set_validators_state(&pre_state.curr_validators, ValidatorSet::Current);
  

        let current_state = get_global_state();
        let mut assurances_state = current_state.availability.clone();

        let output_result = process_assurances(
                                                                            &mut assurances_state, 
                                                                            &input.assurances, 
                                                                            &input.slot,
                                                                            &input.parent);
        
        match output_result {
            Ok(_) => { set_reporting_assurance_state(&assurances_state);},
            Err(_) => { },
        }

        //println!("output_result = {:0x?}", output_result);
        let result_avail_assignments = get_reporting_assurance_state();
        let result_curr_validators = get_validators_state(ValidatorSet::Current);

        assert_eq!(expected_state.avail_assignments, result_avail_assignments);
        assert_eq!(expected_state.curr_validators, result_curr_validators);
        
        match output_result {
            Ok(OutputDataAssurances { reported}) => {
                assert_eq!(expected_output, OutputAssurances::Ok(OutputDataAssurances {reported}));
            }
            Err(error_code) => {
                assert_eq!(expected_output, OutputAssurances::Err(error_code));
            }
        }*/
    }

    #[test]
    fn run_authorizations_tests() {
        
        println!("Authorizations tests in {} mode", *TEST_TYPE);

        let test_files = vec![
            // No guarantees.
            // Shift auths left from both pools.
            "progress_authorizations-1.bin",
            // Guarantees for cores 0 and 1.
            // Consume authentication from both cores pools.
            "progress_authorizations-2.bin",
            // Guarantees for core 1.
            // Shift left authentications for core 0 pool.
            // Consume authentication for core 1 pool.
            "progress_authorizations-3.bin",
        ];
        for file in test_files {
            println!("Running test: {}", file);
            run_test(file);
        }
    }
}