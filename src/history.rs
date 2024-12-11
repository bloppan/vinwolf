use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::collections::VecDeque;
use sp_core::keccak_256;

use crate::types::Hash;
use crate::constants::RECENT_HISTORY_SIZE;
use crate::codec::history::{State, BlockInfo, ReportedWorkPackages, Mmr};
use crate::trie::append;


static STATE_RECENT_HISTORY: Lazy<Mutex<State>> = Lazy::new(|| Mutex::new(State{beta: VecDeque::with_capacity(RECENT_HISTORY_SIZE)}));

pub fn set_history_state(post_state: &State) {
    let mut state = STATE_RECENT_HISTORY.lock().unwrap();
    *state = post_state.clone();
}

pub fn get_history_state() -> State {
    let state = STATE_RECENT_HISTORY.lock().unwrap(); 
    return state.clone();
}

pub fn update_recent_history(
    header_hash: Hash, 
    parent_state_root: Hash, 
    accumulate_root: Hash, 
    work_packages: ReportedWorkPackages
) {
    let mut pre_state = STATE_RECENT_HISTORY.lock().unwrap(); 
    let history_len = pre_state.beta.len();

    if history_len == 0 {
        pre_state.beta.push_back(BlockInfo {
            header_hash,
            mmr: append(&Mmr { peaks: Vec::new() }, accumulate_root, keccak_256),
            state_root: [0u8; 32],
            reported: work_packages,
        });
        return;
    }

    let last_mmr = Mmr {
        peaks: pre_state.beta[history_len - 1].mmr.peaks.clone(),
    };
    pre_state.beta[history_len - 1].state_root = parent_state_root;

    pre_state.beta.push_back(BlockInfo {
        header_hash,
        mmr: append(&last_mmr, accumulate_root, keccak_256),
        state_root: [0u8; 32],
        reported: work_packages,
    });

    if history_len >= 8 {
        pre_state.beta.pop_front();
    }
}

