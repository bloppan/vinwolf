use once_cell::sync::Lazy;
use crate::read_test_file;
use crate::codec::{TestBody, encode_decode_test};

use vinwolf::constants::{VALIDATORS_COUNT, EPOCH_LENGTH, CORES_COUNT};
use vinwolf::types::DisputesExtrinsic;
use vinwolf::blockchain::block::extrinsic::disputes::{DisputesState, OutputDisputes};
use vinwolf::blockchain::state::disputes::{set_old_disputes_state, get_old_disputes_state, update_disputes_state};
use vinwolf::utils::codec::{Decode, BytesReader};

static TEST_TYPE: Lazy<&'static str> = Lazy::new(|| {
    if VALIDATORS_COUNT == 6 && EPOCH_LENGTH == 12 && CORES_COUNT == 2 {
        "tiny"
    } else if VALIDATORS_COUNT == 1023 && EPOCH_LENGTH == 600 && CORES_COUNT == 341 {
        "full"
    } else {
        panic!("Invalid configuration for tiny nor full tests");
    }
});

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
        let disputes_state = DisputesState::decode(&mut reader).expect("Error decoding post DisputesState");
        let expected_output = OutputDisputes::decode(&mut reader).expect("Error decoding post OutputDisputes");
        let expected_state = DisputesState::decode(&mut reader).expect("Error decoding post DisputesState");
        
        set_old_disputes_state(&disputes_state);

        if let Some(current_state) = get_old_disputes_state() {
            assert_eq!(disputes_state, current_state);
        } else {
            panic!("Disputes State was not set before comparison");
        }

        let output_result = update_disputes_state(&disputes_extrinsic);


        if let Some(state_result) = get_old_disputes_state() {
            /*assert_eq!(expected_state, state_result);
            assert_eq!(expected_output, output_result);*/

            assert_eq!(expected_state.psi, state_result.psi);
            assert_eq!(expected_state.rho, state_result.rho);
            assert_eq!(expected_state.tau, state_result.tau);
            assert_eq!(expected_state.kappa, state_result.kappa);
            assert_eq!(expected_state.lambda, state_result.lambda);
            assert_eq!(expected_output, output_result);
        } else {
            panic!("Disputes State was not set before comparison");
        }

}

#[cfg(test)]
mod test {

    use super::*;

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