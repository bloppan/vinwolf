#[cfg(test)]
mod tests {

    use crate::test_types::{InputAuthorizations, StateAuthorizations};
    use crate::codec::tests::{TestBody, encode_decode_test};
    use state_handler::{get_global_state};
    use codec::{BytesReader, Decode};
    use jam_types::{Guarantee, WorkReport};
    use once_cell::sync::Lazy;
    use constants::node::CORES_COUNT;

    static TEST_TYPE: Lazy<&'static str> = Lazy::new(|| {
        if CORES_COUNT == 2 {
            "tiny"
        } else if CORES_COUNT == 341 {
            "full"
        } else {
            panic!("Invalid configuration for tiny nor full tests");
        }
    });

    fn run_test(filename: &str) {

        let test_content = utils::common::read_bin_file(std::path::Path::new(&format!("jamtestvectors/authorizations/{}/{}", *TEST_TYPE, filename))).unwrap();
        let test_body: Vec<TestBody> = vec![
                                        TestBody::InputAuthorizations,
                                        TestBody::StateAuthorizations,
                                        TestBody::StateAuthorizations];
        
        let _ = encode_decode_test(&test_content, &test_body);

        let mut reader = BytesReader::new(&test_content);
        let input = InputAuthorizations::decode(&mut reader).expect("Error decoding post InputAuthorizations");
        let pre_state = StateAuthorizations::decode(&mut reader).expect("Error decoding post Authorizations PreState");
        let expected_state = StateAuthorizations::decode(&mut reader).expect("Error decoding post Authorizations PostState");

        state_handler::auth_pools::set(pre_state.auth_pools);
        state_handler::auth_queues::set(pre_state.auth_queues);
        
        let mut guarantees_extrinsic = Vec::new();
        for auth in input.auths.authorizers.iter() {
            let mut work_report = WorkReport::default();
            work_report.authorizer_hash = auth.auth_hash;
            work_report.core_index = auth.core;
            let mut report_guarantee = Guarantee::default();
            report_guarantee.slot = input.slot;
            report_guarantee.report = work_report;
            guarantees_extrinsic.push(report_guarantee);
        }      

        let mut auth_pool_state = get_global_state().lock().unwrap().auth_pools.clone();
        authorization::process(&mut auth_pool_state, &input.slot, &guarantees_extrinsic);
        
        let result_auth_queues = state_handler::auth_queues::get();

        assert_eq!(expected_state.auth_pools, auth_pool_state);
        assert_eq!(expected_state.auth_queues, result_auth_queues);

    }

    #[test]
    fn run_authorizations_tests() {
        
        dotenv::dotenv().ok();
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
        log::info!("Authorizations tests in {} mode", *TEST_TYPE);

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
            println!("");
            log::info!("Running test: {}", file);
            run_test(file);
        }
    }
}
