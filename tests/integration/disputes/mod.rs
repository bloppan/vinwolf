use once_cell::sync::Lazy;
use crate::integration::{read_test_file, FromProcessError};
use crate::integration::codec::{TestBody, encode_decode_test};

use vinwolf::blockchain::state::{
    ProcessError, get_disputes, get_global_state, get_reporting_assurance, get_time, get_validators, set_disputes, 
    set_reporting_assurance, set_time, set_validators,
};
use vinwolf::constants::{VALIDATORS_COUNT, EPOCH_LENGTH, CORES_COUNT};
use vinwolf::types::{DisputesExtrinsic, OutputDataDisputes};
use vinwolf::blockchain::state::validators::ValidatorSet;
use vinwolf::blockchain::state::disputes::process_disputes;
use vinwolf::utils::codec::{Decode, BytesReader};

pub mod codec;
use codec::{DisputesState, OutputDisputes};

static TEST_TYPE: Lazy<&'static str> = Lazy::new(|| {
    if VALIDATORS_COUNT == 6 && EPOCH_LENGTH == 12 && CORES_COUNT == 2 {
        "tiny"
    } else if VALIDATORS_COUNT == 1023 && EPOCH_LENGTH == 600 && CORES_COUNT == 341 {
        "full"
    } else {
        panic!("Invalid configuration for tiny nor full tests");
    }
});

#[cfg(test)]
mod test {

    use super::*;

    impl FromProcessError for OutputDisputes {
        fn from_process_error(error: ProcessError) -> Self {
            match error {
                ProcessError::DisputesError(code) => OutputDisputes::Err(code),
                _ => panic!("Unexpected error type in conversion"),
            }
        }
    }
    
    fn run_test(filename: &str) {
    
        let test_content = read_test_file(&format!("tests/jamtestvectors/disputes/{}/{}", *TEST_TYPE, filename));
        let test_body: Vec<TestBody> = vec![
                                            TestBody::DisputesExtrinsic,
                                            TestBody::DisputesState,
                                            TestBody::OutputDisputes,
                                            TestBody::DisputesState];
            
            let _ = encode_decode_test(&test_content, &test_body);
    
            let mut reader = BytesReader::new(&test_content);
            let disputes_extrinsic = DisputesExtrinsic::decode(&mut reader).expect("Error decoding post DisputesExtrinsic");
            let pre_state = DisputesState::decode(&mut reader).expect("Error decoding post DisputesState");
            let expected_output = OutputDisputes::decode(&mut reader).expect("Error decoding post OutputDisputes");
            let expected_state = DisputesState::decode(&mut reader).expect("Error decoding post DisputesState");
            
            set_disputes(pre_state.psi);
            set_reporting_assurance(pre_state.rho);
            set_time(pre_state.tau);
            set_validators(pre_state.kappa, ValidatorSet::Current);
            set_validators(pre_state.lambda, ValidatorSet::Previous);
    
            let mut state = get_global_state();
    
            let output_result = process_disputes(
                                                                    &mut state.disputes,
                                                                    &mut state.availability,
                                                                    &disputes_extrinsic);

            match output_result {
                Ok(_) => { 
                    set_disputes(state.disputes.clone());
                    set_reporting_assurance(state.availability.clone());
                },
                Err(_) => { },
            }

            let result_disputes = get_disputes();
            let result_availability = get_reporting_assurance();
            let result_time = get_time();
            let result_curr_validators = get_validators(ValidatorSet::Current);
            let result_prev_validators = get_validators(ValidatorSet::Previous);

            assert_eq!(expected_state.psi, result_disputes);
            assert_eq!(expected_state.rho, result_availability);
            assert_eq!(expected_state.tau, result_time);
            assert_eq!(expected_state.kappa, result_curr_validators);
            assert_eq!(expected_state.lambda, result_prev_validators);

            match output_result {
                Ok(OutputDataDisputes { offenders_mark }) => {
                    assert_eq!(expected_output, OutputDisputes::Ok(OutputDataDisputes { offenders_mark }));
                }
                Err(error) => {
                    assert_eq!(expected_output, OutputDisputes::from_process_error(error));
                }
            }
    
    }

    #[test]
    fn run_disputes_tests() {
        
        println!("Dispute tests in {} mode", *TEST_TYPE);

        let test_files = vec![
            // No verdicts, nothing special happens
            "progress_with_no_verdicts-1.bin",
            // Not sorted work reports within a verdict
            "progress_with_verdicts-1.bin",
            // Not unique votes within a verdict
            "progress_with_verdicts-2.bin",
            // Not sorted, valid verdicts
            "progress_with_verdicts-3.bin",
            // Sorted, valid verdicts
            "progress_with_verdicts-4.bin",
            // Not homogeneous judgements, but positive votes count is not correct
            "progress_with_verdicts-5.bin",
            // Not homogeneous judgements, results in wonky verdict
            "progress_with_verdicts-6.bin",
            // Missing culprits for bad verdict
            "progress_with_culprits-1.bin",
            // Single culprit for bad verdict
            "progress_with_culprits-2.bin",
            // Two culprits for bad verdict, not sorted
            "progress_with_culprits-3.bin",
            // Two culprits for bad verdict, sorted
            "progress_with_culprits-4.bin",
            // Report an already recorded verdict, with culprits
            "progress_with_culprits-5.bin",
            // Culprit offender already in the offenders list
            "progress_with_culprits-6.bin",
            // Offender relative to a not present verdict
            "progress_with_culprits-7.bin",
            // Missing faults for good verdict
            "progress_with_faults-1.bin",
            // One fault offender for good verdict
            "progress_with_faults-2.bin",
            // Two fault offenders for a good verdict, not sorted
            "progress_with_faults-3.bin",
            // Two fault offenders for a good verdict, sorted
            "progress_with_faults-4.bin",
            // Report an already recorded verdict, with faults
            "progress_with_faults-5.bin",
            // Fault offender already in the offenders list
            "progress_with_faults-6.bin",
            // Auditor marked as offender, but vote matches the verdict.
            "progress_with_faults-7.bin",
            // Invalidation of availability assignments
            "progress_invalidates_avail_assignments-1.bin",
            // Bad signature within the verdict judgements
            "progress_with_bad_signatures-1.bin",
            // Use previous epoch validators set for verdict signatures verification
            "progress_with_verdict_signatures_from_previous_set-1.bin",
            // Age too old for verdicts judgements
            "progress_with_verdict_signatures_from_previous_set-2.bin",
            // Bad signature within the culprits sequence
            "progress_with_bad_signatures-2.bin",
        ];
        for file in test_files {
            println!("Running test: {}", file);
            run_test(file);
        }
    }
}