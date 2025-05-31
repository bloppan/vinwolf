use once_cell::sync::Lazy;
use crate::integration::w3f::{read_test_file, FromProcessError};
use crate::integration::w3f::codec::{TestBody, encode_decode_test};

use vinwolf::constants::{CORES_COUNT, EPOCH_LENGTH, ROTATION_PERIOD, VALIDATORS_COUNT};
use vinwolf::types::{RawState, Block, ValidatorSet, ProcessError, Statistics, Extrinsic, ServiceAccounts, Account};
use vinwolf::blockchain::state::{
    get_global_state, set_reporting_assurance, get_reporting_assurance, set_auth_pools, get_auth_pools, 
    set_entropy, get_entropy, set_validators, get_validators, set_recent_history, get_recent_history,
    set_disputes, get_disputes, set_statistics, get_statistics, set_service_accounts, get_service_accounts
};
use vinwolf::blockchain::state::reporting_assurance::process_guarantees;
use vinwolf::blockchain::state::statistics::process;
use vinwolf::utils::codec::{Decode, BytesReader};



pub mod codec;


#[cfg(test)]
mod tests {
    
    /*impl FromProcessError for OutputWorkReport {
        fn from_process_error(error: ProcessError) -> Self {
            match error {
                ProcessError::ReportError(code) => OutputWorkReport::Err(code),
                _ => panic!("Unexpected error type in conversion"),
            }
        }
    }*/

    use super::*;

    fn run_test(filename: &str) {

        

    }

    #[test]
    fn run_traces_tests() {

        let test_body: Vec<TestBody> = vec![TestBody::RawState,
                                            TestBody::Block,
                                            TestBody::RawState];

        let test_content = read_test_file(&format!("tests/test_vectors/w3f/jamtestvectors/traces/fallback/00000000.bin"));
        let _ = encode_decode_test(&test_content, &test_body);
        
        let mut reader = BytesReader::new(&test_content);
        let pre_state = RawState::decode(&mut reader).expect("Error decoding post WorkReport PreState");
        let block = Block::decode(&mut reader).expect("Error decoding post OutputWorkReport");
        let post_state = RawState::decode(&mut reader).expect("Error decoding post WorkReport PostState");
        //println!("pre_state: {:x?}", pre_state);
        //println!("block: {:x?}", block);
        //println!("post_state: {:x?}", post_state);
        
        let mut slot = 1;
        
        loop {
            println!("reading file: {}", slot);
            let test_content = read_test_file(&format!("tests/test_vectors/w3f/jamtestvectors/traces/fallback/{:08}.bin", slot));
            let _ = encode_decode_test(&test_content, &test_body);
            
            let mut reader = BytesReader::new(&test_content);
            let pre_state = RawState::decode(&mut reader).expect("Error decoding post WorkReport PreState");
            let block = Block::decode(&mut reader).expect("Error decoding post OutputWorkReport");
            let post_state = RawState::decode(&mut reader).expect("Error decoding post WorkReport PostState");
            //println!("pre_state: {:x?}", pre_state);
            //println!("block: {:x?}", block);
            //println!("post_state: {:x?}", post_state);

            slot += 1;
        }



    }
}