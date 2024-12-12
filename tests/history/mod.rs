use crate::{read_test_file};
use crate::codec::{TestBody, encode_decode_test};

use vinwolf::codec::{Decode, BytesReader};
use vinwolf::codec::history::{Input as InputHistory, State as StateHistory};
use vinwolf::history::{update_recent_history, set_history_state, get_history_state};

fn run_recent_history_test(filename: &str) {
  
    let test_content = read_test_file(&format!("tests/jamtestvectors/history/data/{}", filename));
    let test_body: Vec<TestBody> = vec![TestBody::InputHistory, TestBody::StateHistory, TestBody::StateHistory];

    let _ = encode_decode_test(&test_content, &test_body);
    
    let mut reader = BytesReader::new(&test_content);
    let input = InputHistory::decode(&mut reader).expect("Error decoding InputHistory");
    let expected_pre_state = StateHistory::decode(&mut reader).expect("Error decoding pre StateHistory");
    let expected_post_state = StateHistory::decode(&mut reader).expect("Error decoding post StateHistory");
    
    set_history_state(&expected_pre_state);
    let pre_state_result = get_history_state();
    assert_eq!(expected_pre_state, pre_state_result);

    update_recent_history(
                    input.header_hash, 
                    input.parent_state_root, 
                    input.accumulate_root, 
                    input.work_packages);

    let result_post_state = get_history_state();
    assert_eq!(expected_post_state, result_post_state);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_blocks_history_1() {
        run_recent_history_test("progress_blocks_history-1.bin");
    }

    #[test]
    fn progress_blocks_history_2() {
        run_recent_history_test("progress_blocks_history-2.bin");
    }

    #[test]
    fn progress_blocks_history_3() {
        run_recent_history_test("progress_blocks_history-3.bin");
    }

    #[test]
    fn progress_blocks_history_4() {
        run_recent_history_test("progress_blocks_history-4.bin");
    }
}   