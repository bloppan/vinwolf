use once_cell::sync::Lazy;
use crate::integration::{read_test_file, FromProcessError};
use crate::integration::codec::{TestBody, encode_decode_test};

use vinwolf::types::{Safrole, OutputSafrole, OutputDataSafrole};
use vinwolf::constants::{VALIDATORS_COUNT, EPOCH_LENGTH, TICKET_SUBMISSION_ENDS};
use vinwolf::blockchain::state::validators::ValidatorSet;
use vinwolf::blockchain::state::{
    get_global_state, set_time, get_time, set_entropy, get_entropy, set_validators, get_validators, set_safrole, get_safrole,
    ProcessError
};
use vinwolf::blockchain::state::safrole::process_safrole;
use vinwolf::utils::codec::{Decode, BytesReader};

use crate::integration::safrole::codec::{InputSafrole, SafroleState};

pub mod codec;

static TEST_TYPE: Lazy<&'static str> = Lazy::new(|| {
    if VALIDATORS_COUNT == 6 && EPOCH_LENGTH == 12 && TICKET_SUBMISSION_ENDS == 10 {
        "tiny"
    } else if VALIDATORS_COUNT == 1023 && EPOCH_LENGTH == 600 && TICKET_SUBMISSION_ENDS == 500 {
        "full"
    } else {
        panic!("Invalid configuration for tiny nor full tests");
    }
});

#[cfg(test)]
mod tests {

    use super::*;

    impl FromProcessError for OutputSafrole {
        fn from_process_error(error: ProcessError) -> Self {
            match error {
                ProcessError::SafroleError(code) => OutputSafrole::Err(code),
                _ => panic!("Unexpected error type in conversion"),
            }
        }
    }

    fn run_safrole_test(filename: &str) {

        let test_content = read_test_file(&format!("tests/jamtestvectors/safrole/{}/{}", *TEST_TYPE, filename));
        let test_body: Vec<TestBody> = vec![
                                        TestBody::InputSafrole, 
                                        TestBody::SafroleState, 
                                        TestBody::OutputSafrole, 
                                        TestBody::SafroleState];
        
        let _ = encode_decode_test(&test_content, &test_body);

        let mut reader = BytesReader::new(&test_content);
        let input = InputSafrole::decode(&mut reader).expect("Error decoding InputSafrole");
        let pre_state = SafroleState::decode(&mut reader).expect("Error decoding pre_state Safrole");
        let expected_output = OutputSafrole::decode(&mut reader).expect("Error decoding OutputSafrole");       
        let expected_state = SafroleState::decode(&mut reader).expect("Error decoding expected_state Safrole");

        let pre_state_safrole = Safrole {
            pending_validators: pre_state.gamma_k,
            ticket_accumulator: pre_state.gamma_a,
            seal: pre_state.gamma_s,
            epoch_root: pre_state.gamma_z,
        };

        set_time(pre_state.tau);
        set_entropy(pre_state.eta);
        set_validators(pre_state.lambda, ValidatorSet::Previous);
        set_validators(pre_state.kappa, ValidatorSet::Current);
        set_validators(pre_state.iota, ValidatorSet::Next);
        set_safrole(pre_state_safrole);

        let mut state = get_global_state();

        let output_result = process_safrole(&mut state.safrole
                                                                        , &mut state.entropy
                                                                        , &mut state.curr_validators
                                                                        , &mut state.prev_validators
                                                                        , &mut state.time
                                                                        , &input.slot
                                                                        , &input.entropy
                                                                        , &input.tickets_extrinsic
                                                                        , &input.post_offenders);
        
        match output_result {
            Ok(_) => { 
                set_time(state.time.clone());
                set_entropy(state.entropy.clone());
                set_validators(state.prev_validators.clone(), ValidatorSet::Previous);
                set_validators(state.curr_validators.clone(), ValidatorSet::Current);
                set_validators(state.next_validators.clone(), ValidatorSet::Next);
                set_safrole(state.safrole.clone());           
            },
            Err(_) => { },
        }

        let result_time = get_time();
        let result_entropy = get_entropy();
        let result_curr_validators = get_validators(ValidatorSet::Current);
        let result_prev_validators = get_validators(ValidatorSet::Previous);
        let result_next_validators = get_validators(ValidatorSet::Next);
        let result_safrole = get_safrole();

        assert!(expected_state.tau == result_time);
        assert!(expected_state.eta == result_entropy);
        assert!(expected_state.lambda == result_prev_validators);
        assert!(expected_state.kappa == result_curr_validators);
        assert!(expected_state.iota == result_next_validators);
        
        let expected_safrole_state = Safrole {
            pending_validators: expected_state.gamma_k,
            ticket_accumulator: expected_state.gamma_a,
            seal: expected_state.gamma_s,
            epoch_root: expected_state.gamma_z,
        };

        /*assert!(expected_safrole_state == result_safrole);
        println!("expected safrole: {:?}", expected_safrole_state);
        println!("result safrole: {:?}", result_safrole);*/

        match output_result {
            Ok(OutputDataSafrole { epoch_mark, tickets_mark }) => {
                assert_eq!(expected_output, OutputSafrole::Ok(OutputDataSafrole {epoch_mark, tickets_mark}));
            }
            Err(error) => {
                assert_eq!(expected_output, OutputSafrole::from_process_error(error));
            }
        }
    }

    #[test]
    fn enact_epoch_change_with_no_tickets_1() {
        run_safrole_test("enact-epoch-change-with-no-tickets-1.bin");
    }

    #[test]
    fn enact_epoch_change_with_no_tickets_2() {
        run_safrole_test("enact-epoch-change-with-no-tickets-2.bin");
    }

    #[test]
    fn enact_epoch_change_with_no_tickets_3() {
        run_safrole_test("enact-epoch-change-with-no-tickets-3.bin");
    }

    #[test]
    fn enact_epoch_change_with_no_tickets_4() {
        run_safrole_test("enact-epoch-change-with-no-tickets-4.bin");
    }

    /*#[test]
    fn publish_tickets_no_mark_1() {
        run_safrole_test("publish-tickets-no-mark-1.bin");
    }

    #[test]
    fn publish_tickets_no_mark_2() {
        run_safrole_test("publish-tickets-no-mark-2.bin");
    }

    #[test]
    fn publish_tickets_no_mark_3() {
        run_safrole_test("publish-tickets-no-mark-3.bin");
    }

    #[test]
    fn publish_tickets_no_mark_4() {
        run_safrole_test("publish-tickets-no-mark-4.bin");
    }

    #[test]
    fn publish_tickets_no_mark_5() {
        run_safrole_test("publish-tickets-no-mark-5.bin");
    }

    #[test]
    fn publish_tickets_no_mark_6() {
        run_safrole_test("publish-tickets-no-mark-6.bin");
    }

    #[test]
    fn publish_tickets_no_mark_7() {
        run_safrole_test("publish-tickets-no-mark-7.bin");
    }

    #[test]
    fn publish_tickets_no_mark_8() {
        run_safrole_test("publish-tickets-no-mark-8.bin");
    }

    #[test]
    fn publish_tickets_no_mark_9() {
        run_safrole_test("publish-tickets-no-mark-9.bin");
    }

    #[test]
    fn publish_tickets_with_mark_1() {
        run_safrole_test("publish-tickets-with-mark-1.bin");
    }

    #[test]
    fn publish_tickets_with_mark_2() {
        run_safrole_test("publish-tickets-with-mark-2.bin");
    }

    #[test]
    fn publish_tickets_with_mark_3() {
        run_safrole_test("publish-tickets-with-mark-3.bin");
    }

    #[test]
    fn publish_tickets_with_mark_4() {
        run_safrole_test("publish-tickets-with-mark-4.bin");
    }

    #[test]
    fn publish_tickets_with_mark_5() {
        run_safrole_test("publish-tickets-with-mark-5.bin");
    }

    #[test]
    fn skip_epoch_tail_1() {
        run_safrole_test("skip-epoch-tail-1.bin");
    }

    #[test]
    fn skip_epochs_1() {
        run_safrole_test("skip-epochs-1.bin");
    }

    #[test]
    fn enact_epoch_change_with_padding_1() {
        run_safrole_test("enact-epoch-change-with-padding-1.bin");
    }*/

}