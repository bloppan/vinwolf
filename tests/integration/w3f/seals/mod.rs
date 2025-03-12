use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use hex;
use std::convert::TryInto;

extern crate vinwolf;

use vinwolf::constants::{EPOCH_LENGTH, VALIDATORS_COUNT};  
use vinwolf::types::{
    Header, BandersnatchVrfSignature, Entropy, EntropyPool, Safrole, BandersnatchEpoch, TicketsOrKeys, TimeSlot, BandersnatchPublic, 
    TicketsMark, ValidatorsData, ValidatorSet
};
use vinwolf::utils::codec::{Decode, BytesReader};
use vinwolf::blockchain::block::extrinsic::tickets::verify_seal;
use vinwolf::blockchain::state::{set_validators, set_entropy};
use vinwolf::blockchain::state::safrole::{create_ring_set, create_root_epoch};

use ark_ec_vrfs::suites::bandersnatch::edwards as bandersnatch_ark_ec_vrfs;
use bandersnatch_ark_ec_vrfs::Public;

#[derive(Deserialize, Debug, PartialEq)]
struct Testcase {
    bandersnatch_pub: String,
    bandersnatch_priv: String,
    ticket_id: String,
    attempt: u8,
    c_for_H_s: String,
    m_for_H_s: String,
    H_s: String,
    c_for_H_v: String,
    m_for_H_v: String,
    H_v: String,
    eta3: String,
    T: u8,
    header_bytes: String,
}

struct TestDecoded {
    bandersnatch_pub: [u8; 32],
    bandersnatch_priv: [u8; 32],
    ticket_id: [u8; 32],
    attempt: u8,
    c_for_H_s: Vec<u8>,
    m_for_H_s: Vec<u8>,
    H_s: BandersnatchVrfSignature,
    c_for_H_v: Vec<u8>,
    m_for_H_v: Vec<u8>,
    H_v: BandersnatchVrfSignature,
    eta3: [u8; 32],
    T: u8,
    header_bytes: Vec<u8>,
}

fn decode_test(testcase: Testcase) -> TestDecoded {
    TestDecoded {
        bandersnatch_pub: hex::decode((&testcase.bandersnatch_pub).as_str()).expect("Failed to decode hex").try_into().expect("Failed to convert to array"),
        bandersnatch_priv: hex::decode((&testcase.bandersnatch_priv).as_str()).expect("Failed to decode hex").try_into().expect("Failed to convert to array"),
        attempt: testcase.attempt,
        c_for_H_s: hex::decode((&testcase.c_for_H_s).as_str()).expect("Failed to decode hex"),
        m_for_H_s: hex::decode((&testcase.m_for_H_s).as_str()).expect("Failed to decode hex"),
        H_s: hex::decode((&testcase.H_s).as_str()).expect("Failed to decode hex").try_into().expect("Failed to convert to BandersnatchVrfSignature"),
        c_for_H_v: hex::decode((&testcase.c_for_H_v).as_str()).expect("Failed to decode hex"),
        m_for_H_v: hex::decode((&testcase.m_for_H_v).as_str()).expect("Failed to decode hex"),
        H_v: hex::decode((&testcase.H_v).as_str()).expect("Failed to decode hex").try_into().expect("Failed to convert to BandersnatchVrfSignature"),
        eta3: hex::decode((&testcase.eta3).as_str()).expect("Failed to decode hex").try_into().expect("Failed to convert to Entropy"),
        ticket_id: {
            if testcase.T == 0 {
                [0u8; 32]
            } else {
                hex::decode(&testcase.ticket_id[2..]).expect("Failed to decode hex").try_into().expect("Failed to convert to array")
            }
        },
        T: testcase.T,
        header_bytes: hex::decode((&testcase.header_bytes).as_str()).expect("Failed to decode hex"),
    }
}

#[cfg(test)]
mod tests {

    use super::*;
   
    fn run_seal_test(filename: &str) {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests/test_vectors/jamtestvectors/seals/");
        path.push(filename);
        let mut file = File::open(&path).expect("Failed to open JSON file");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Failed to read JSON file");
        let testcase: Testcase = serde_json::from_str(&contents).expect("Failed to deserialize JSON");
     
        let test_decoded = decode_test(testcase);


        let mut reader = BytesReader::new(&test_decoded.header_bytes);
        let header = Header::decode(&mut reader).expect("Failed to decode Header");
        //println!("{:?}", header);
        let block_author = header.unsigned.author_index as usize;
        println!("block_author = {}", block_author);
        let mut entropy_pool: EntropyPool = EntropyPool::default();
        entropy_pool.buf[3] = Entropy { entropy: test_decoded.eta3 };
        set_entropy(entropy_pool.clone());

        let mut public_keys: Box<[BandersnatchPublic; VALIDATORS_COUNT]> = Box::new([BandersnatchPublic::default(); VALIDATORS_COUNT]);
        public_keys[header.unsigned.author_index as usize] = test_decoded.bandersnatch_pub;
        
        let ring_set: Vec<Public> = create_ring_set(public_keys.as_ref());
        let ring_root = create_root_epoch(ring_set.clone());

        let mut current_validators = ValidatorsData::default();
        current_validators.0[block_author].bandersnatch = test_decoded.bandersnatch_pub;
        set_validators(current_validators.clone(), ValidatorSet::Current);

        let i = header.unsigned.slot % EPOCH_LENGTH as TimeSlot;
        let mut safrole_state = Safrole::default();
        safrole_state.epoch_root = ring_root;
        match test_decoded.T {
            0 => {
                let mut epoch_keys = BandersnatchEpoch::default();
                epoch_keys.0[i as usize] = test_decoded.bandersnatch_pub;
                safrole_state.seal = TicketsOrKeys::Keys(epoch_keys);
            },
            1 => {
                let mut tickets = TicketsMark::default();
                tickets.tickets_mark[i as usize].id = test_decoded.ticket_id;
                tickets.tickets_mark[i as usize].attempt = test_decoded.attempt;
                safrole_state.seal = TicketsOrKeys::Tickets(tickets);
            },
            _ => panic!("Invalid T value"),
        }

        verify_seal(&safrole_state, &entropy_pool, &current_validators, ring_set, &header).expect("Failed to verify seal");

        println!("Test done\n");
    
    }

    #[test]
    fn test_programs() {
        let test_files = vec![
            "0-0.json",
            "0-1.json",
            "0-2.json",
            "0-4.json",
            "0-5.json",
            "1-0.json",
            "1-1.json",
            "1-2.json",
            "1-3.json",
            "1-4.json",
            "1-5.json",
        ];
        for file in test_files {
            println!("Running test for file: {}", file);
            run_seal_test(file);
        }
    }
}
