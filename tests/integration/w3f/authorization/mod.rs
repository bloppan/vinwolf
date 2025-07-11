use once_cell::sync::Lazy;
use crate::integration::w3f::read_test_file;
use crate::integration::w3f::codec::{TestBody, encode_decode_test};
use dotenv::dotenv;

pub mod codec;
use codec::{InputAuthorizations, StateAuthorizations};

use vinwolf::constants::CORES_COUNT;
use vinwolf::blockchain::state::{get_global_state, set_auth_pools, set_auth_queues, get_auth_queues};
use vinwolf::blockchain::state::authorization::process;
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

    use vinwolf::types::{GuaranteesExtrinsic, ReportGuarantee, WorkReport};

    use super::*;

    fn run_test(filename: &str) {

        let test_content = read_test_file(&format!("tests/test_vectors/w3f/jamtestvectors/authorizations/{}/{}", *TEST_TYPE, filename));
        let test_body: Vec<TestBody> = vec![
                                        TestBody::InputAuthorizations,
                                        TestBody::StateAuthorizations,
                                        TestBody::StateAuthorizations];
        
        let _ = encode_decode_test(&test_content, &test_body);

        let mut reader = BytesReader::new(&test_content);
        let input = InputAuthorizations::decode(&mut reader).expect("Error decoding post InputAuthorizations");
        let pre_state = StateAuthorizations::decode(&mut reader).expect("Error decoding post Authorizations PreState");
        let expected_state = StateAuthorizations::decode(&mut reader).expect("Error decoding post Authorizations PostState");

        set_auth_pools(pre_state.auth_pools);
        set_auth_queues(pre_state.auth_queues);
        
        let mut guarantees_extrinsic = GuaranteesExtrinsic::default();
        for auth in input.auths.authorizers.iter() {
            let mut work_report = WorkReport::default();
            work_report.authorizer_hash = auth.auth_hash;
            work_report.core_index = auth.core;
            let mut report_guarantee = ReportGuarantee::default();
            report_guarantee.slot = input.slot;
            report_guarantee.report = work_report;
            guarantees_extrinsic.report_guarantee.push(report_guarantee);
        }      

        let mut auth_pool_state = get_global_state().lock().unwrap().auth_pools.clone();
        process(&mut auth_pool_state, &input.slot, &guarantees_extrinsic);
        
        let result_auth_queues = get_auth_queues();

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
            log::info!("");
            log::info!("Running test: {}", file);
            run_test(file);
        }
    }
}