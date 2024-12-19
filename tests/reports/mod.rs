use once_cell::sync::Lazy;
use crate::read_test_file;
use crate::codec::{TestBody, encode_decode_test};

use vinwolf::constants::{CORES_COUNT, EPOCH_LENGTH, ROTATION_PERIOD, VALIDATORS_COUNT};
use vinwolf::types::DisputesRecords;
use vinwolf::blockchain::state::{get_global_state, set_reporting_assurance_state, get_reporting_assurance_state};
use vinwolf::blockchain::state::disputes::{set_disputes_state, get_disputes_state};
use vinwolf::blockchain::state::validators::{set_validators_state, get_validators_state, ValidatorSet};
use vinwolf::blockchain::state::entropy::{set_entropy_state, get_entropy_state};
use vinwolf::blockchain::state::reporting_assurance::process_report_assurance;
use vinwolf::blockchain::state::recent_history::{set_history_state, get_history_state}; // TODO update this
use vinwolf::blockchain::state::authorization::{set_authpool_state, get_authpool_state};
use vinwolf::blockchain::state::services::{set_services_state, get_services_state};
use vinwolf::blockchain::state::time::set_time_state;
use vinwolf::utils::codec::{Decode, BytesReader};
use vinwolf::utils::codec::work_report::{InputWorkReport, WorkReportState, OutputWorkReport, OutputData, ErrorCode};

static TEST_TYPE: Lazy<&'static str> = Lazy::new(|| {
    if VALIDATORS_COUNT == 6 && CORES_COUNT == 2 && ROTATION_PERIOD == 4 && EPOCH_LENGTH == 12 {
        "tiny"
    } else if VALIDATORS_COUNT == 1023 && CORES_COUNT == 341 && ROTATION_PERIOD == 10 && EPOCH_LENGTH == 600{
        "full"
    } else {
        panic!("Invalid configuration for tiny nor full tests");
    }
});

#[cfg(test)]
mod tests {
    

    use super::*;

    fn run_test(filename: &str) {

        let test_content = read_test_file(&format!("tests/jamtestvectors/reports/{}/{}", *TEST_TYPE, filename));
        let test_body: Vec<TestBody> = vec![
                                        TestBody::InputWorkReport,
                                        TestBody::WorkReportState,
                                        TestBody::OutputWorkReport,
                                        TestBody::WorkReportState];
        
        let _ = encode_decode_test(&test_content, &test_body);

        let mut reader = BytesReader::new(&test_content);
        let input = InputWorkReport::decode(&mut reader).expect("Error decoding post InputWorkReport");
        let pre_state = WorkReportState::decode(&mut reader).expect("Error decoding post WorkReport PreState");
        let expected_output = OutputWorkReport::decode(&mut reader).expect("Error decoding post OutputWorkReport");
        let expected_state = WorkReportState::decode(&mut reader).expect("Error decoding post WorkReport PostState");
        
        let disputes_state = DisputesRecords {
            good: vec![],
            bad: vec![],
            wonky: vec![],
            offenders: pre_state.offenders.offenders.clone(),
        };
        set_disputes_state(&disputes_state);
        set_time_state(&input.slot);
        set_reporting_assurance_state(&pre_state.avail_assignments);
        set_validators_state(&pre_state.curr_validators, ValidatorSet::Current);
        set_validators_state(&pre_state.prev_validators, ValidatorSet::Previous);
        set_entropy_state(&pre_state.entropy);
        set_history_state(&pre_state.recent_blocks);
        set_authpool_state(&pre_state.auth_pools);
        set_services_state(&pre_state.services);

        let current_state = get_global_state();
        let mut assurances_state = current_state.availability.clone();

        let output_result = process_report_assurance(
                                                                            &mut assurances_state, 
                                                                            &input.guarantees, 
                                                                            &input.slot);
        
        match output_result {
            Ok(_) => { set_reporting_assurance_state(&assurances_state);},
            Err(_) => { },
        }

        //println!("output_result = {:0x?}", output_result);
        let result_avail_assignments = get_reporting_assurance_state();
        let result_curr_validators = get_validators_state(ValidatorSet::Current);
        let result_prev_validators = get_validators_state(ValidatorSet::Previous);
        let result_entropy = get_entropy_state();
        let result_disputes = get_disputes_state();
        let result_history = get_history_state();
        let result_authpool = get_authpool_state();
        let result_services = get_services_state();

        assert_eq!(expected_state.avail_assignments, result_avail_assignments);
        assert_eq!(expected_state.curr_validators, result_curr_validators);
        assert_eq!(expected_state.prev_validators, result_prev_validators);
        assert_eq!(expected_state.entropy, result_entropy);
        assert_eq!(expected_state.offenders.offenders, result_disputes.offenders);
        assert_eq!(expected_state.recent_blocks, result_history);
        assert_eq!(expected_state.auth_pools, result_authpool);
        assert_eq!(expected_state.services, result_services);

        match output_result {
            Ok(OutputData { reported, reporters }) => {
                assert_eq!(expected_output, OutputWorkReport::Ok(OutputData {reported, reporters}));
            }
            Err(error_code) => {
                assert_eq!(expected_output, OutputWorkReport::Err(error_code));
            }
        }
    }

    #[test]
    fn run_work_report_tests() {
        
        println!("Work report tests in {} mode", *TEST_TYPE);

        let test_files = vec![
            // Report uses current guarantors rotation
            "report_curr_rotation-1.bin",
            // Report uses previous guarantors rotation.
            // Previous rotation falls within previous epoch, thus previous epoch validators set is used to construct 
            // report core assignment to pick expected guarantors.
            "report_prev_rotation-1.bin",             
            // Multiple good work reports.
            "multiple_reports-1.bin",
            // Context anchor is not recent enough.
            "anchor_not_recent-1.bin",
            // Context Beefy MMR root doesn't match the one at anchor.
            "bad_beefy_mmr-1.bin",
            // Work result code hash doesn't match the one expected for the service.
            "bad_code_hash-1.bin",
            // Core index is too big.
            "bad_core_index-1.bin",
            // Work result service identifier doesn't have any associated account in state.
            "bad_service_id-1.bin",
            // Context state root doesn't match the one at anchor.
            "bad_state_root-1.bin",
            // Validator index is too big.
            "bad_validator_index-1.bin",
            // Multiple authorizers are available for the same work report.
            // Only one is consumed.
            // "consume_authorization_once-1.bin", Este no se por que lo tengo, no esta en el repo original
            // A core is not available.
            "core_engaged-1.bin",
            // Prerequisite is missing.
            "dependency_missing-1.bin",
            // Package was already available in recent history.
            "duplicate_package_in_recent_history-1.bin",
            // Report contains a duplicate package.
            "duplicated_package_in_report-1.bin",
            // Report refers to a slot in the future with respect to container block slot.
            "future_report_slot-1.bin",
            // Invalid report guarantee signature.
            "bad_signature-1.bin",
            // Work report per core gas is very high, still less than the limit.
            "high_work_report_gas-1.bin",
            // Work report per core gas is too much high.
            "too_high_work_report_gas-1.bin",
            // Accumulate gas is below the service minimum.
            "service_item_gas_too_low-1.bin",
            // Work report has many dependencies, still less than the limit.
            "many_dependencies-1.bin",
            // Work report has too many dependencies.
            "too_many_dependencies-1.bin", 
            // Report with no enough guarantors signatures.
            "no_enough_guarantees-1.bin",
            // Target core without any authorizer.
            "not_authorized-1.bin",       
            // Target core with unexpected authorizer.
            "not_authorized-2.bin",
            // Guarantors indices are not sorted or unique.
            "not_sorted_guarantor-1.bin",
            // Reports cores are not sorted or unique.
            "out_of_order_guarantees-1.bin",
            // Report guarantee slot is too old with respect to block slot.
            "report_before_last_rotation-1.bin",                         
            // Simple report dependency satisfied by another work report in the same extrinsic.
            "reports_with_dependencies-1.bin",
            // Work reports mutual dependency (indirect self-referential dependencies).
            "reports_with_dependencies-2.bin",
            // Work report direct self-referential dependency.
            "reports_with_dependencies-3.bin",
            // Work report dependency satisfied by recent blocks history.
            "reports_with_dependencies-4.bin",
            // Work report segments tree root lookup dependency satisfied by another work report in the same extrinsic.
            "reports_with_dependencies-5.bin",
            // Work report segments tree root lookup dependency satisfied by recent blocks history.
            "reports_with_dependencies-6.bin",
            // Segments tree root lookup item not found in recent blocks history.
            "segment_root_lookup_invalid-1.bin",
            // Segments tree root lookup item found in recent blocks history but with an unexpected value.
            "segment_root_lookup_invalid-2.bin",
            // Unexpected guarantor for work report core.
            "wrong_assignment-1.bin",
        ];
        for file in test_files {
            println!("Running test: {}", file);
            run_test(file);
        }
    }
}

