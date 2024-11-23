use once_cell::sync::Lazy;
use std::sync::Mutex;
use crate::types::{Hash};
use crate::constants::{RECENT_HISTORY_SIZE};
use crate::codec::{Encode, EncodeSize};
use crate::codec::history::{Input, State, BlockInfo, ReportedWorkPackages, Mmr, MmrPeak};
use crate::trie::{merkle_b, append};

use sp_core::keccak_256;


static STATE_RECENT_HISTORY: Lazy<Mutex<State>> = Lazy::new(|| Mutex::new(State{beta: vec![]}));

pub fn set_history_state(state: &State) {
    let mut pre_state = STATE_RECENT_HISTORY.lock().unwrap();
    *pre_state = state.clone();
}

pub fn get_history_state() -> State {
    let lock = STATE_RECENT_HISTORY.lock().unwrap(); 
    return lock.clone();
}

pub fn update_recent_history(
    header_hash: Hash, 
    parent_state_root: Hash, 
    accumulate_root: Hash, 
    work_packages: ReportedWorkPackages
) {
    let mut pre_state = STATE_RECENT_HISTORY.lock().unwrap(); 
    let history_len = pre_state.beta.len();
    println!("history_len = {history_len}");

    if history_len == 0 {
        pre_state.beta.push(BlockInfo{
                                header_hash: header_hash,
                                mmr: append(&Mmr {peaks: Vec::new()}, accumulate_root, keccak_256),
                                state_root: [0u8; 32],
                                reported: work_packages});
    } else if history_len >= 1 && history_len <= 8 {
        let mmr_result: Mmr = Mmr{peaks: pre_state.beta[history_len - 1].mmr.peaks.clone()};
        pre_state.beta[history_len - 1].state_root = parent_state_root;

        pre_state.beta.push(BlockInfo{
                                header_hash: header_hash,
                                mmr: append(&mmr_result, accumulate_root, keccak_256),
                                state_root: [0u8; 32],
                                reported: work_packages});
    }
    
}
