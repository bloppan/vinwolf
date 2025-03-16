use crate::integration::w3f::read_test_file;
use crate::integration::w3f::codec::{TestBody, encode_decode_test};

use vinwolf::utils::codec::{Decode, BytesReader};
use vinwolf::types::BlockHistory;
use vinwolf::blockchain::state::{set_recent_history, get_recent_history};
use vinwolf::blockchain::state::recent_history::process_recent_history;
use codec::InputHistory;

pub mod codec;

fn run_test(filename: &str) {
  
    let test_content = read_test_file(&format!("tests/test_vectors/w3f/jamtestvectors/history/data/{}", filename));
    let test_body: Vec<TestBody> = vec![TestBody::InputHistory, TestBody::BlockHistory, TestBody::BlockHistory];

    let _ = encode_decode_test(&test_content, &test_body);
    
    let mut reader = BytesReader::new(&test_content);
    let input = InputHistory::decode(&mut reader).expect("Error decoding InputHistory");
    let expected_pre_state = BlockHistory::decode(&mut reader).expect("Error decoding pre BlockHistory");
    let expected_post_state = BlockHistory::decode(&mut reader).expect("Error decoding post BlockHistory");
    
    set_recent_history(expected_pre_state.clone());

    let mut recent_history_state = get_recent_history();
    assert_eq!(expected_pre_state, recent_history_state);

    process_recent_history(
                    &mut recent_history_state,
                    &input.header_hash, 
                    &input.parent_state_root, 
                    &input.accumulate_root, 
                    &input.work_packages);

    assert_eq!(expected_post_state, recent_history_state);

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_recent_history_tests() {
        
        println!("Recent history tests");

        let test_files = vec![
            // Empty history queue.
            "progress_blocks_history-1.bin",
            // Not empty nor full history queue.
            "progress_blocks_history-2.bin",
            // Fill the history queue.
            "progress_blocks_history-3.bin",
            // Shift the history queue.
            "progress_blocks_history-4.bin",
        ];
        for file in test_files {
            println!("Running test: {}", file);
            run_test(file);
        }
    }
}   