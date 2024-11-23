use std::path::PathBuf;
use std::fs::File;
use std::io::Read;

use vinwolf::codec::{Encode, Decode, BytesReader, ReadError};
use vinwolf::codec::history::{Input, State};

use crate::codec::find_first_difference;

use vinwolf::history::{update_recent_history, set_history_state, get_history_state};

//const TEST_TYPE: &str = "tiny";

fn run_history_bin_file(filename: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join(format!("data/history/data/{}", filename));
    
    let mut file = File::open(&path).expect("Failed to open file");
    let mut test_content = Vec::new();
    let _success = file.read_to_end(&mut test_content);
    let mut history_test = BytesReader::new(&test_content);
    
    let input = Input::decode(&mut history_test).expect("Error decoding input");
    let e_input = Input::encode(&input);
    let pos_input = history_test.get_position();
    if let Some(diff_pos) = find_first_difference(&test_content[..pos_input], &e_input, "Input") {
        panic!("Difference found in 'input' at byte position {}", diff_pos);
    }
    assert_eq!(test_content[..pos_input], e_input);
    if pos_input > test_content.len() {
        panic!("input: Out of test bounds | pos = {}", pos_input);
    }
    
    //println!("\n\n input  = {:0X?}", input);

    let mut pre_state = State::decode(&mut history_test).expect("Error decoding pre_state");
    let e_pre_state = State::encode(&pre_state);
    let pos_pre_state = history_test.get_position();
    if let Some(diff_pos) = find_first_difference(&test_content[pos_input..pos_pre_state], &e_pre_state, "PreState") {
        panic!("Difference found in 'pre_state' at byte position {}", pos_input + diff_pos);
    }
    assert_eq!(test_content[pos_input..pos_pre_state], e_pre_state);
    if pos_pre_state > test_content.len() {
        panic!("pre_state: Out of test_bounds | pos = {}", pos_pre_state);
    }
    
    let post_state = State::decode(&mut history_test).expect("Error decoding post_state");
    let e_post_state = State::encode(&post_state);
    let pos_post_state = history_test.get_position();
    
    //println!("\n\npost_state = {:0X?}", post_state);
    
    if let Some(diff_pos) = find_first_difference(&test_content[pos_pre_state..pos_post_state], &e_post_state, "PostState") {
        panic!("Difference found in 'post_state' at byte position {}", pos_pre_state + diff_pos);
    }
    if pos_post_state > test_content.len() {
        panic!("post_state: Out of test_bounds | pos = {}", pos_post_state);
    }
    assert_eq!(test_content[pos_pre_state..pos_post_state], e_post_state);
    //println!("pos_post_state = {}, test length = {}", pos_post_state, test_content.len());
/*
    println!("input = {:0X?}", input);
    println!("pre_state = {:0X?}", pre_state);
    println!("output = {:?}", output);
    println!("post_state = {:?}", post_state);
*/
    

    let mut result_encoded: Vec<u8> = Vec::new();
    result_encoded.extend(e_input);
    result_encoded.extend(e_pre_state);
    result_encoded.extend(e_post_state);

    assert_eq!(test_content, result_encoded);

    set_history_state(&pre_state);
    update_recent_history(input.header_hash, input.parent_state_root, input.accumulate_root, input.work_packages);
    let state_result = get_history_state();

    assert_eq!(post_state, state_result);
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_blocks_history_1() {
        run_history_bin_file("progress_blocks_history-1.bin");
    }

    #[test]
    fn test_progress_blocks_history_2() {
        run_history_bin_file("progress_blocks_history-2.bin");
    }

    #[test]
    fn test_progress_blocks_history_3() {
        run_history_bin_file("progress_blocks_history-3.bin");
    }
/*
    #[test]
    fn test_progress_blocks_history_4() {
        run_history_bin_file("progress_blocks_history-4.bin");
    }*/
}   