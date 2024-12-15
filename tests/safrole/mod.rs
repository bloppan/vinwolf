use once_cell::sync::Lazy;
use crate::read_test_file;
use crate::codec::{TestBody, encode_decode_test};

use vinwolf::constants::{VALIDATORS_COUNT, EPOCH_LENGTH, TICKET_SUBMISSION_ENDS};
use vinwolf::blockchain::state::safrole::codec::{SafroleState, Output as OutputSafrole, Input as InputSafrole};
use vinwolf::blockchain::state::safrole::update_state;
use vinwolf::utils::codec::{Decode, BytesReader};

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
        let mut state = SafroleState::decode(&mut reader).expect("Error decoding pre_state SafroleState");
        let expected_output = OutputSafrole::decode(&mut reader).expect("Error decoding OutputSafrole");       
        let expected_state = SafroleState::decode(&mut reader).expect("Error decoding expected_state SafroleState");

        let output_result = update_state(input, &mut state);

        assert_eq!(expected_state, state);
        assert_eq!(expected_output, output_result);
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

    #[test]
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
    }

}