use once_cell::sync::Lazy;
use crate::integration::read_test_file;
use crate::integration::codec::{TestBody, encode_decode_test};

pub mod codec;
use codec::{InputAuthorizations, StateAuthorizations};

use vinwolf::constants::CORES_COUNT;
use vinwolf::blockchain::state::{get_global_state, set_authpools, set_authqueues, get_authqueues};
use vinwolf::blockchain::state::authorization::process_authorizations;
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

        let mut reader = BytesReader::new(&test_content);
        let input = InputAuthorizations::decode(&mut reader).expect("Error decoding post InputAuthorizations");
        let pre_state = StateAuthorizations::decode(&mut reader).expect("Error decoding post Authorizations PreState");
        let expected_state = StateAuthorizations::decode(&mut reader).expect("Error decoding post Authorizations PostState");

        set_authpools(&pre_state.auth_pools);
        set_authqueues(&pre_state.auth_queues);
        
        let code_authorizers = input.auths.clone();

        let mut auth_pool_state = get_global_state().auth_pools.clone();
        process_authorizations(&mut auth_pool_state, &input.slot, &code_authorizers);
        
        let result_auth_queues = get_authqueues();

        assert_eq!(expected_state.auth_pools, auth_pool_state);
        assert_eq!(expected_state.auth_queues, result_auth_queues);

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