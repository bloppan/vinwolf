use once_cell::sync::Lazy;
use crate::integration::{read_test_file, FromProcessError};
use crate::integration::codec::{TestBody, encode_decode_test};

use vinwolf::types::{Safrole, OutputSafrole, OutputDataSafrole, DisputesRecords};
use vinwolf::constants::{VALIDATORS_COUNT, EPOCH_LENGTH, TICKET_SUBMISSION_ENDS, TICKET_ENTRIES_PER_VALIDATOR};
use vinwolf::blockchain::state::validators::ValidatorSet;
use vinwolf::blockchain::state::{
    get_global_state, set_time, get_time, set_entropy, get_entropy, set_validators, get_validators, set_safrole, get_safrole,
    set_disputes, get_disputes, ProcessError
};
use vinwolf::blockchain::state::safrole::process_safrole;
use vinwolf::utils::codec::{Decode, BytesReader};

use crate::integration::safrole::codec::{InputSafrole, SafroleState};

pub mod codec;

static TEST_TYPE: Lazy<&'static str> = Lazy::new(|| {
    if VALIDATORS_COUNT == 6 && EPOCH_LENGTH == 12 && TICKET_SUBMISSION_ENDS == 10 && TICKET_ENTRIES_PER_VALIDATOR == 3{
        "tiny"
    } else if VALIDATORS_COUNT == 1023 && EPOCH_LENGTH == 600 && TICKET_SUBMISSION_ENDS == 500 && TICKET_ENTRIES_PER_VALIDATOR == 2 {
        "full"
    } else {
        panic!("Invalid configuration for tiny nor full tests");
    }
});

#[cfg(test)]
mod tests {

    use vinwolf::blockchain::state::set_disputes;

    use super::*;

    impl FromProcessError for OutputSafrole {
        fn from_process_error(error: ProcessError) -> Self {
            match error {
                ProcessError::SafroleError(code) => OutputSafrole::Err(code),
                _ => panic!("Unexpected error type in conversion"),
            }
        }
    }

    fn run_test(filename: &str) {

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
        let disputes = DisputesRecords {
            good: vec![],
            bad: vec![],
            wonky: vec![],
            offenders: pre_state.post_offenders,
        };

        set_time(pre_state.tau);
        set_entropy(pre_state.eta);
        set_validators(pre_state.lambda, ValidatorSet::Previous);
        set_validators(pre_state.kappa, ValidatorSet::Current);
        set_validators(pre_state.iota, ValidatorSet::Next);
        set_safrole(pre_state_safrole);
        set_disputes(disputes);

        let mut state = get_global_state();

        let output_result = process_safrole(&mut state.safrole
                                                                        , &mut state.entropy
                                                                        , &mut state.curr_validators
                                                                        , &mut state.prev_validators
                                                                        , &mut state.time
                                                                        , &input.slot
                                                                        , &input.entropy
                                                                        , &input.tickets_extrinsic);
        
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

        let expected_safrole_state = Safrole {
            pending_validators: expected_state.gamma_k,
            ticket_accumulator: expected_state.gamma_a,
            seal: expected_state.gamma_s,
            epoch_root: expected_state.gamma_z,
        };

        let result_time = get_time();
        let result_entropy = get_entropy();
        let result_curr_validators = get_validators(ValidatorSet::Current);
        let result_prev_validators = get_validators(ValidatorSet::Previous);
        let result_next_validators = get_validators(ValidatorSet::Next);
        let result_safrole = get_safrole();
        let result_disputes = get_disputes();

        assert_eq!(expected_state.tau, result_time);
        assert_eq!(expected_state.eta, result_entropy);
        assert_eq!(expected_state.lambda, result_prev_validators);
        assert_eq!(expected_state.kappa, result_curr_validators);
        assert_eq!(expected_state.iota, result_next_validators);       
        assert_eq!(expected_safrole_state, result_safrole);
        assert_eq!(expected_state.post_offenders, result_disputes.offenders);

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
    fn run_safrole_tests() {
        
        println!("Safrole tests in {} mode", *TEST_TYPE);
        let test_files = vec![
            // Progress by one slot.
            // Randomness accumulator is updated.
            "enact-epoch-change-with-no-tickets-1.bin", // OK
            // Progress from slot X to slot X.
            // Timeslot must be strictly monotonic.
            "enact-epoch-change-with-no-tickets-2.bin", // FAIL 
            // Progress from a slot at the begining of the epoch to a slot in the epoch's tail.
            // Tickets mark is not generated (no enough tickets).
            "enact-epoch-change-with-no-tickets-3.bin", // OK 
            // Progress from epoch's tail to next epoch.
            // Authorities and entropies are rotated. Epoch mark is generated.
            "enact-epoch-change-with-no-tickets-4.bin", // FAIL
            // Submit an extrinsic with a bad ticket attempt number.
            "publish-tickets-no-mark-1.bin", // FAIL
            // Submit good tickets extrinsic from some authorities.
            "publish-tickets-no-mark-2.bin", // OK
            // Submit one ticket already recorded in the state.
            "publish-tickets-no-mark-3.bin", // FAIL
            // Submit tickets in bad order.
            "publish-tickets-no-mark-4.bin", // FAIL 
            // Submit tickets with bad ring proof.
            "publish-tickets-no-mark-5.bin", // FAIL 
            // Submit some tickets.
            "publish-tickets-no-mark-6.bin", // OK 
            // Submit tickets when epoch's lottery is over.
            "publish-tickets-no-mark-7.bin", // FAIL
            // Progress into epoch tail, closing the epoch's lottery.
            // No enough tickets, thus no tickets mark is generated.
            "publish-tickets-no-mark-8.bin", // OK
            // Progress into next epoch with no enough tickets.
            // Accumulated tickets are discarded. Epoch mark generated. Fallback method enacted.
            "publish-tickets-no-mark-9.bin", // OK
            // Publish some tickets with an almost full tickets accumulator.
            // Tickets accumulator is not full yet. No ticket is dropped from accumulator.
            "publish-tickets-with-mark-1.bin", // OK
            // Publish some tickets filling the accumulator.
            // Two old tickets are removed from the accumulator.
            "publish-tickets-with-mark-2.bin", // OK
            // Publish some tickets with a full accumulator.
            // Some old ticket are removed to make space for new ones.
            "publish-tickets-with-mark-3.bin", // OK
            // With a full accumulator, conclude the lottery.
            // Tickets mark is generated.
            "publish-tickets-with-mark-4.bin", // OK
            // With a published tickets mark, progress into next epoch.
            // Epoch mark is generated. Tickets are enacted.
            "publish-tickets-with-mark-5.bin", // OK
            // Progress to next epoch by skipping epochs tail with a full tickets accumulator.
            // Tickets mark has no chance to be generated. Accumulated tickets discarded. Fallback method enacted.
            "skip-epoch-tail-1.bin", // OK
            // Progress skipping epochs with a full tickets accumulator.
            // Tickets mark is not generated. Accumulated tickets discarded. Fallback method enacted.
            "skip-epochs-1.bin", // OK
            // On epoch change we recompute the ring commitment.
            // One of the keys to be used is invalidated (zeroed out) because it belongs to the (posterior) offenders list.
            // One of the keys is just invalid (i.e. it can't be decoded into a valid Bandersnatch point).
            // Both the invalid keys are replaced with the padding point during ring commitment computation.
            "enact-epoch-change-with-padding-1.bin", // OK
            ];
        
        for file in test_files {
            println!("Running test: {}", file);
            run_test(file);
        }
    }
    
}