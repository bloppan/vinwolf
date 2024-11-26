use once_cell::sync::Lazy;
use crate::{read_test_file};
use crate::codec::{TestBody, encode_decode_test};

use vinwolf::constants::{VALIDATORS_COUNT, EPOCH_LENGTH, CORES_COUNT};
use vinwolf::codec::disputes_extrinsic::{DisputesExtrinsic, DisputesState, OutputData};


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

    let test_content = read_test_file(&format!("data/disputes/{}/{}", *TEST_TYPE, filename));
    let test_body: Vec<TestBody> = vec![
                                        TestBody::DisputesExtrinsic,
                                        TestBody::DisputesState,
                                        TestBody::OutputData,
                                        TestBody::DisputesState];
        
        let _ = encode_decode_test(&test_content, &test_body);
}

#[cfg(test)]
mod test {

    use super::*;

   /* #[test]
    fn progress_with_no_veredicts_1() {
        run_dispute_test("progress_with_no_verdicts-1.bin");
    }*/

    #[test]
    fn run_disputes_tests() {
        
        println!("Dispute tests in {} mode", *TEST_TYPE);
        
        let test_files = vec![
            "progress_with_no_verdicts-1.bin",
            "progress_with_verdicts-1.bin",
            "progress_with_verdicts-2.bin",
            "progress_with_verdicts-3.bin",
            "progress_with_verdicts-4.bin",
            "progress_with_verdicts-5.bin",
            "progress_with_verdicts-6.bin",
            "progress_with_culprits-1.bin",
            "progress_with_culprits-2.bin",
            "progress_with_culprits-3.bin",
            "progress_with_culprits-4.bin",
            "progress_with_culprits-5.bin",
            "progress_with_culprits-6.bin",
            "progress_with_culprits-7.bin",
            "progress_with_faults-1.bin",
            "progress_with_faults-2.bin",
            "progress_with_faults-3.bin",
            "progress_with_faults-4.bin",
            "progress_with_faults-5.bin",
            "progress_with_faults-6.bin",
            "progress_with_faults-7.bin",
            "progress_invalidates_avail_assignments-1.bin",
            "progress_with_bad_signatures-1.bin",
            "progress_with_bad_signatures-2.bin",
            "progress_with_verdict_signatures_from_previous_set-1.bin",
            "progress_with_verdict_signatures_from_previous_set-2.bin",
        ];
        for file in test_files {
            println!("Running test: {}", file);
            run_test(file);
        }
    }
}