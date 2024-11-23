use once_cell::sync::Lazy;
use std::sync::Mutex;
use crate::types::{Hash};
use crate::codec::{Encode, EncodeSize};
use crate::codec::history::{Input, State, BlockInfo, ReportedWorkPackages};
use crate::trie::{merkle_b, append};

use sp_core::keccak_256;


static STATE_RECENT_HISTORY: Lazy<Mutex<State>> = Lazy::new(|| Mutex::new(State{beta: vec![]}));

pub fn set_history_state(state: &State) {
    let mut pre_state = STATE_RECENT_HISTORY.lock().unwrap();
    *pre_state = state.clone();
}

pub fn get_history_state() -> State {
    let lock = STATE_RECENT_HISTORY.lock().unwrap(); // Adquiere el lock
    return lock.clone();
}

pub fn update_recent_history(
    header_hash: Hash, 
    parent_state_root: Hash, 
    accumulate_root: Hash, 
    work_packages: ReportedWorkPackages
) {
    let mut pre_state = STATE_RECENT_HISTORY.lock().unwrap(); 

    let b = append(&[], accumulate_root, keccak_256);

    pre_state.beta.push(BlockInfo{
                            header_hash: header_hash,
                            mmr: mmr.peaks: Some(b),
                            state_root: [0u8; 32],
                            reported: work_packages});
}

/*
pub fn progress_blocks(input: &Input, state: &State) {

    let mut combined = Vec::new();
    input.parent_state_root.encode_size(4).encode_to(&mut combined);
    input.header_hash.encode_to(&mut combined);
    println!("combined 1 = {:0x?}", combined);

    let r = merkle_b(&combined, keccak_256);

    println!("r = {:0x?}", r);

    let value: [u8; 32] = [
        0x87, 0x20, 0xb9, 0x7d, 0xdd, 0x6a, 0xcc, 0x0f,
        0x6e, 0xb6, 0x6e, 0x09, 0x55, 0x24, 0x03, 0x86,
        0x75, 0xa4, 0xe4, 0x06, 0x7a, 0xdc, 0x10, 0xec,
        0x39, 0x93, 0x9e, 0xae, 0xfc, 0x47, 0xd8, 0x42
    ];

    let value2: [u8; 32] = [
    0x75, 0x07, 0x51, 0x5a, 0x48, 0x43, 0x9d, 0xc5,
    0x8b, 0xc3, 0x18, 0xc4, 0x8a, 0x12, 0x0b, 0x65,
    0x61, 0x36, 0x69, 0x9f, 0x42, 0xbf, 0xd2, 0xbd,
    0x45, 0x47, 0x3b, 0xec, 0xba, 0x53, 0x46, 0x2d
    ];

    let fin = [value, value2].concat();

    //let b = append(&[Some(value)], value2, keccak_256);
    let b = append(&[], value, keccak_256);

    println!("res = {:0x?}", b);
} */