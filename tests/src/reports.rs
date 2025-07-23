#[cfg(test)]
mod tests {
    
    use once_cell::sync::Lazy;
    use crate::FromProcessError;
    use crate::codec::{TestBody, encode_decode_test};
    use crate::test_types::{InputWorkReport, WorkReportState, OutputWorkReport};
    use constants::node::{CORES_COUNT, EPOCH_LENGTH, ROTATION_PERIOD, VALIDATORS_COUNT};
    use jam_types::{DisputesRecords, OutputDataReports, ValidatorSet, ProcessError, Statistics, ServiceAccounts, Account};
    use block::Extrinsic;
    use handler::{
        get_global_state, set_reporting_assurance, get_reporting_assurance, set_auth_pools, get_auth_pools, set_entropy, get_entropy, 
        set_validators, get_validators, set_recent_history, get_recent_history, set_disputes, get_disputes, set_statistics, get_statistics,
        set_service_accounts, get_service_accounts
    };
    use handler::set_current_block_history;
    use codec::{Decode, BytesReader};

    static TEST_TYPE: Lazy<&'static str> = Lazy::new(|| {
        if VALIDATORS_COUNT == 6 && CORES_COUNT == 2 && ROTATION_PERIOD == 4 && EPOCH_LENGTH == 12 {
            "tiny"
        } else if VALIDATORS_COUNT == 1023 && CORES_COUNT == 341 && ROTATION_PERIOD == 10 && EPOCH_LENGTH == 600{
            "full"
        } else {
            panic!("Invalid configuration for tiny nor full tests");
        }
    });

    impl FromProcessError for OutputWorkReport {
        fn from_process_error(error: ProcessError) -> Self {
            match error {
                ProcessError::ReportError(code) => OutputWorkReport::Err(code),
                _ => panic!("Unexpected error type in conversion"),
            }
        }
    }

    fn run_test(filename: &str) {

        let test_content = utils::common::read_bin_file(std::path::Path::new(&format!("jamtestvectors/reports/{}/{}", *TEST_TYPE, filename))).unwrap();
        let test_body: Vec<TestBody> = vec![
                                        TestBody::InputWorkReport,
                                        TestBody::WorkReportState,
                                        TestBody::OutputWorkReport,
                                        TestBody::WorkReportState];
        
        let _ = encode_decode_test(&test_content, &test_body);
        
        let mut reader = BytesReader::new(&test_content);
        let input = InputWorkReport::decode(&mut reader).expect("Error decoding InputWorkReport");
        let pre_state = WorkReportState::decode(&mut reader).expect("Error decoding WorkReport PreState");
        let expected_output = OutputWorkReport::decode(&mut reader).expect("Error decoding OutputWorkReport");
        let expected_state = WorkReportState::decode(&mut reader).expect("Error decoding WorkReport PostState");
      
        /*println!("\ninput: {:x?}", input);
        println!("\npre_state: {:x?}", pre_state);
        println!("\nexpected_output: {:x?}", expected_output);
        println!("\nexpected_output: {:x?}", expected_output);*/

        let disputes_state = DisputesRecords {
            good: vec![],
            bad: vec![],
            wonky: vec![],
            offenders: pre_state.offenders.clone(),
        };

        set_disputes(disputes_state);
        set_reporting_assurance(pre_state.avail_assignments);
        set_validators(pre_state.curr_validators, ValidatorSet::Current);
        set_validators(pre_state.prev_validators, ValidatorSet::Previous);
        set_entropy(pre_state.entropy.clone());
        set_current_block_history(pre_state.recent_blocks.clone());
        set_recent_history(pre_state.recent_blocks);
        set_auth_pools(pre_state.auth_pools);
        
        let mut services_accounts = ServiceAccounts::default();
        for acc in pre_state.services.0.iter() {
            let mut account = Account::default();
            account.code_hash = acc.info.code_hash.clone();
            account.balance = acc.info.balance.clone();
            account.acc_min_gas = acc.info.acc_min_gas.clone();
            account.xfer_min_gas = acc.info.xfer_min_gas.clone();
            services_accounts.insert(acc.id.clone(), account.clone());
        }
        set_service_accounts(services_accounts);
        let mut statistics_state = Statistics::default();
        statistics_state.cores = pre_state.cores_statistics;
        statistics_state.services = pre_state.services_statistics;
        set_statistics(statistics_state.clone());

        let current_state = get_global_state().lock().unwrap().clone();
        let mut assurances_state = current_state.availability.clone();

        let output_result = reports::guarantee::process(&mut assurances_state, 
                                                                             &input.guarantees, 
                                                                             &input.slot,
                                                                            &get_entropy(),
                                                                            &get_validators(ValidatorSet::Previous),
                                                                            &get_validators(ValidatorSet::Current));
        
        match output_result {
            Ok(_) => { 
                set_reporting_assurance(assurances_state);
                let mut extrinsic = Extrinsic::default();
                extrinsic.guarantees = input.guarantees.clone();
                let reports = input.guarantees.report_guarantee.iter().map(|guarantee| guarantee.report.clone()).collect::<Vec<_>>();
                statistics::process(&mut statistics_state, &input.slot, &0, &extrinsic, &reports);
                set_statistics(statistics_state.clone());
            },
            Err(_) => { log::error!("{:?}", output_result) },
        }

        let result_disputes = get_disputes();
        let result_avail_assignments = get_reporting_assurance();
        let result_curr_validators = get_validators(ValidatorSet::Current);
        let result_prev_validators = get_validators(ValidatorSet::Previous);
        let result_entropy = get_entropy();
        let result_history = get_recent_history();
        let result_authpool = get_auth_pools();
        let result_services = get_service_accounts();
        let result_statistics = get_statistics();

        assert_eq!(expected_state.offenders, result_disputes.offenders);
        assert_eq!(expected_state.avail_assignments, result_avail_assignments);
        assert_eq!(expected_state.curr_validators, result_curr_validators);
        assert_eq!(expected_state.prev_validators, result_prev_validators);
        assert_eq!(expected_state.entropy, result_entropy);
        assert_eq!(expected_state.recent_blocks, result_history);
        assert_eq!(expected_state.auth_pools, result_authpool);

        let mut expected_services_accounts = ServiceAccounts::default();
        for acc in expected_state.services.0.iter() {
            let mut account = Account::default();
            account.code_hash = acc.info.code_hash.clone();
            account.balance = acc.info.balance.clone();
            account.acc_min_gas = acc.info.acc_min_gas.clone();
            account.xfer_min_gas = acc.info.xfer_min_gas.clone();
            expected_services_accounts.insert(acc.id.clone(), account.clone());
        }
        
        assert_eq!(expected_services_accounts, result_services);

        for (i, core) in result_statistics.cores.records.iter().enumerate() {
            assert_eq!(core.imports, result_statistics.cores.records[i].imports);
            assert_eq!(core.exports, result_statistics.cores.records[i].exports);
            assert_eq!(core.extrinsic_size, result_statistics.cores.records[i].extrinsic_size);
            assert_eq!(core.extrinsic_count, result_statistics.cores.records[i].extrinsic_count);
            assert_eq!(core.bundle_size, result_statistics.cores.records[i].bundle_size);  
            assert_eq!(core.gas_used, result_statistics.cores.records[i].gas_used);
        }
        
        assert_eq!(expected_state.services_statistics, result_statistics.services);

        match output_result {
            Ok(OutputDataReports { reported, reporters }) => {
                assert_eq!(expected_output, OutputWorkReport::Ok(OutputDataReports {reported, reporters}));
            }
            Err(error) => {
                assert_eq!(expected_output, OutputWorkReport::from_process_error(error));
            }
        }
    }

    #[test]
    fn run_work_report_tests() {
        
        dotenv::dotenv().ok();
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
        log::info!("Work report tests in {} mode", *TEST_TYPE);
        
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
            "too_high_work_report_gas-1.bin",  //***************************************  REPORTAR A DAVXY por que refactoriza el gas en tiny
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
            // Work report output is very big, still less than the limit.
            "big_work_report_output-1.bin",
            // Work report output is size is over the limit.
            "too_big_work_report_output-1.bin",
            "with_avail_assignments-1.bin",
        ];
        for file in test_files {
            log::info!("");
            log::info!("Running test: {}", file);
            run_test(file);
        }
    }
}

