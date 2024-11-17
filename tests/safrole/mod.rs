use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use vinwolf::codec::safrole::{SafroleState, Output, Input};
use vinwolf::codec::{Encode, Decode, BytesReader};
use vinwolf::safrole::update_state;

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_TYPE: &str = "tiny";

    fn find_first_difference(data1: &[u8], data2: &[u8], _part: &str) -> Option<usize> {
        data1.iter()
            .zip(data2.iter())
            .position(|(byte1, byte2)| byte1 != byte2)
            .map(|pos| {
                //println!("Difference at {} byte position: {}", part, pos);
                println!("First 32 bytes of data1: {:0X?}", &data1[pos..pos + 64.min(data1.len())]);
                println!("First 32 bytes of data2: {:0X?}", &data2[pos..pos + 64.min(data2.len())]);
                pos
            })
    }

    fn run_safrole_bin_file(filename: &str) {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join(format!("data/safrole/{}/{}", TEST_TYPE, filename));
        
        let mut file = File::open(&path).expect("Failed to open file");
        let mut test_content = Vec::new();
        let _success = file.read_to_end(&mut test_content);
        let mut safrole_test = BytesReader::new(&test_content);
        
        let input = Input::decode(&mut safrole_test).expect("Error decoding input");
        let e_input = Input::encode(&input);
        let pos_input = safrole_test.get_position();
        if let Some(diff_pos) = find_first_difference(&test_content[..pos_input], &e_input, "Input") {
            panic!("Difference found in 'input' at byte position {}", diff_pos);
        }
        assert_eq!(test_content[..pos_input], e_input);
        if pos_input > test_content.len() {
            panic!("input: Out of test bounds | pos = {}", pos_input);
        }
        
        //println!("\n\n input  = {:0X?}", input);

        let mut pre_state = SafroleState::decode(&mut safrole_test).expect("Error decoding pre_state");
        let e_pre_state = SafroleState::encode(&pre_state);
        let pos_pre_state = safrole_test.get_position();
        if let Some(diff_pos) = find_first_difference(&test_content[pos_input..pos_pre_state], &e_pre_state, "PreState") {
            panic!("Difference found in 'pre_state' at byte position {}", pos_input + diff_pos);
        }
        assert_eq!(test_content[pos_input..pos_pre_state], e_pre_state);
        if pos_pre_state > test_content.len() {
            panic!("pre_state: Out of test_bounds | pos = {}", pos_pre_state);
        }
        
        //println!("\n\n pre_state  = {:0X?}", pre_state);

        let output = Output::decode(&mut safrole_test).expect("Error decoding output");
        let e_output = Output::encode(&output);
        let pos_output = safrole_test.get_position();
        
        //println!("pos_output = {pos_output}");
        
        if let Some(diff_pos) = find_first_difference(&test_content[pos_pre_state..pos_output], &e_output, "Output") {
            panic!("Difference found in 'output' at byte position {}", pos_pre_state + diff_pos);
        }
        assert_eq!(test_content[pos_input..pos_pre_state], e_pre_state);
        if pos_output > test_content.len() {
            panic!("output: Out of test_bounds | pos = {}", pos_output);
        }
        
        //println!("\n\n output  = {:0X?}", output);
        
        let post_state = SafroleState::decode(&mut safrole_test).expect("Error decoding post_state");
        let e_post_state = SafroleState::encode(&post_state);
        let pos_post_state = safrole_test.get_position();
        
        //println!("\n\npost_state = {:0X?}", post_state);
        
        if let Some(diff_pos) = find_first_difference(&test_content[pos_output..pos_post_state], &e_post_state, "PostState") {
            panic!("Difference found in 'post_state' at byte position {}", pos_output + diff_pos);
        }
        if pos_post_state > test_content.len() {
            panic!("post_state: Out of test_bounds | pos = {}", pos_post_state);
        }
        //println!("pos_post_state = {}, test length = {}", pos_post_state, test_content.len());
/*
        println!("input = {:0X?}", input);
        println!("pre_state = {:0X?}", pre_state);
        println!("output = {:?}", output);
        println!("post_state = {:?}", post_state);
*/
        
        let e_post_state = SafroleState::encode(&post_state);

        let mut result_encoded: Vec<u8> = Vec::new();
        result_encoded.extend(e_input);
        result_encoded.extend(e_pre_state);
        result_encoded.extend(e_output);
        result_encoded.extend(e_post_state);

        assert_eq!(test_content, result_encoded);

        let res_output = update_state(input, &mut pre_state);

        assert_eq!(post_state.tau, pre_state.tau);
        assert_eq!(post_state.eta, pre_state.eta);
        assert_eq!(post_state.lambda, pre_state.lambda);
        assert_eq!(post_state.kappa, pre_state.kappa);
        assert_eq!(post_state.gamma_k, pre_state.gamma_k);
        assert_eq!(post_state.iota, pre_state.iota);
        assert_eq!(post_state.gamma_a, pre_state.gamma_a);
        assert_eq!(post_state.gamma_s, pre_state.gamma_s);
        assert_eq!(post_state.gamma_z, pre_state.gamma_z);

        assert_eq!(post_state, pre_state);
        assert_eq!(output, res_output);
    }

    #[test]
    fn test_enact_epoch_change_with_no_tickets_1() {
        run_safrole_bin_file("enact-epoch-change-with-no-tickets-1.bin");
    }

    #[test]
    fn test_enact_epoch_change_with_no_tickets_2() {
        run_safrole_bin_file("enact-epoch-change-with-no-tickets-2.bin");
    }

    #[test]
    fn test_enact_epoch_change_with_no_tickets_3() {
        run_safrole_bin_file("enact-epoch-change-with-no-tickets-3.bin");
    }

    #[test]
    fn test_enact_epoch_change_with_no_tickets_4() {
        run_safrole_bin_file("enact-epoch-change-with-no-tickets-4.bin");
    }

    #[test]
    fn test_publish_tickets_no_mark_1() {
        run_safrole_bin_file("publish-tickets-no-mark-1.bin");
    }

    #[test]
    fn test_publish_tickets_no_mark_2() {
        run_safrole_bin_file("publish-tickets-no-mark-2.bin");
    }

    #[test]
    fn test_publish_tickets_no_mark_3() {
        run_safrole_bin_file("publish-tickets-no-mark-3.bin");
    }

    #[test]
    fn test_publish_tickets_no_mark_4() {
        run_safrole_bin_file("publish-tickets-no-mark-4.bin");
    }

    #[test]
    fn test_publish_tickets_no_mark_5() {
        run_safrole_bin_file("publish-tickets-no-mark-5.bin");
    }

    #[test]
    fn test_publish_tickets_no_mark_6() {
        run_safrole_bin_file("publish-tickets-no-mark-6.bin");
    }

    #[test]
    fn test_publish_tickets_no_mark_7() {
        run_safrole_bin_file("publish-tickets-no-mark-7.bin");
    }

    #[test]
    fn test_publish_tickets_no_mark_8() {
        run_safrole_bin_file("publish-tickets-no-mark-8.bin");
    }

    #[test]
    fn test_publish_tickets_no_mark_9() {
        run_safrole_bin_file("publish-tickets-no-mark-9.bin");
    }

    #[test]
    fn test_publish_tickets_with_mark_1() {
        run_safrole_bin_file("publish-tickets-with-mark-1.bin");
    }

    #[test]
    fn test_publish_tickets_with_mark_2() {
        run_safrole_bin_file("publish-tickets-with-mark-2.bin");
    }

    #[test]
    fn test_publish_tickets_with_mark_3() {
        run_safrole_bin_file("publish-tickets-with-mark-3.bin");
    }

    #[test]
    fn test_publish_tickets_with_mark_4() {
        run_safrole_bin_file("publish-tickets-with-mark-4.bin");
    }

    #[test]
    fn test_publish_tickets_with_mark_5() {
        run_safrole_bin_file("publish-tickets-with-mark-5.bin");
    }

    #[test]
    fn test_skip_epoch_tail_1() {
        run_safrole_bin_file("skip-epoch-tail-1.bin");
    }

    #[test]
    fn test_skip_epochs_1() {
        run_safrole_bin_file("skip-epochs-1.bin");
    }

    #[test]
    fn test_enact_epoch_change_with_padding_1() {
        run_safrole_bin_file("enact-epoch-change-with-padding-1.bin");
    }

}