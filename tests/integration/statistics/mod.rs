use once_cell::sync::Lazy;
use crate::integration::read_test_file;
use crate::integration::codec::{TestBody, encode_decode_test};

pub mod schema;
use schema::{InputStatistics, StateStatistics};

use vinwolf::constants::{CORES_COUNT, VALIDATORS_COUNT};
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

    fn run_test(filename: &str) {

        let test_content = read_test_file(&format!("tests/jamtestvectors/statistics/{}/{}", *TEST_TYPE, filename));
        let test_body: Vec<TestBody> = vec![
                                        TestBody::InputStatistics,
                                        TestBody::StateStatistics,
                                        TestBody::StateStatistics];
        
        let _ = encode_decode_test(&test_content, &test_body);

        let mut reader = BytesReader::new(&test_content);
        let input = InputStatistics::decode(&mut reader).expect("Error decoding post InputAuthorizations");
        let pre_state = StateStatistics::decode(&mut reader).expect("Error decoding post Authorizations PreState");
        let expected_state = StateStatistics::decode(&mut reader).expect("Error decoding post Authorizations PostState");

    }

    #[test]
    fn run_statistics_tests() {
        
        println!("Authorizations tests in {} mode", *TEST_TYPE);

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
            println!("Running test: {}", file);
            run_test(file);
        }
    }
}