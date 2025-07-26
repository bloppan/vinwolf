#[cfg(test)]
mod tests {

    use once_cell::sync::Lazy;
    use crate::codec::tests::{TestBody, encode_decode_test};
    use crate::test_types::{InputStatistics, StateStatistics};
    use constants::node::{CORES_COUNT, EPOCH_LENGTH, VALIDATORS_COUNT};
    use jam_types::{ValidatorSet, Statistics};
    use state_handler::{get_global_state};
    use codec::{Decode, BytesReader};

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

        let test_content = utils::common::read_bin_file(std::path::Path::new(&format!("jamtestvectors/statistics/{}/{}", *TEST_TYPE, filename))).unwrap();
        let test_body: Vec<TestBody> = vec![
                                        TestBody::InputStatistics,
                                        TestBody::StateStatistics,
                                        TestBody::StateStatistics];
        
        let _ = encode_decode_test(&test_content, &test_body);

        let mut reader = BytesReader::new(&test_content);
        let input = InputStatistics::decode(&mut reader).expect("Error decoding InputStatistics");
        let pre_state = StateStatistics::decode(&mut reader).expect("Error decoding Statitstics PreState");
        let expected_state = StateStatistics::decode(&mut reader).expect("Error decoding Statitstics PostState");

        let mut statistics = Statistics::default();
        statistics.curr = pre_state.curr_stats;
        statistics.prev = pre_state.prev_stats;
        state_handler::statistics::set(statistics.clone());
        state_handler::time::set(pre_state.tau);
        state_handler::validators::set(pre_state.curr_validators, ValidatorSet::Current);

        let mut result_statistics = get_global_state().lock().unwrap().statistics.clone();
        statistics::process(&mut result_statistics, &input.slot, &input.author_index, &input.extrinsic, &[]);

        let result_time = state_handler::time::get();
        let result_validators = state_handler::validators::get(ValidatorSet::Current);
        
        assert_eq!(expected_state.curr_stats, result_statistics.curr);
        assert_eq!(expected_state.prev_stats, result_statistics.prev);
        assert_eq!(expected_state.tau, result_time);
        assert_eq!(expected_state.curr_validators, result_validators);
    }

    #[test]
    fn run_statistics_tests() {
        
        dotenv::dotenv().ok();
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
        log::info!("Statitstics tests in {} mode", *TEST_TYPE);

        let test_files = vec![
            // Empty extrinsic with no epoch change.
            // Only author blocks counter is incremented.
            "stats_with_empty_extrinsic-1.bin",
            // Misc extrinsic information with no epoch change.
            // See "Extrinsic Semantic Validity" section.
            "stats_with_epoch_change-1.bin",
            // Misc extrinsic information with no epoch change.
            // See "Extrinsic Semantic Validity" section.
            "stats_with_some_extrinsic-1.bin",
        ];
        for file in test_files {
            log::info!("Running test: {}", file);
            run_test(file);
        }
    }
}