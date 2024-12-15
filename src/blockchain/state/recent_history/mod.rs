use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::collections::VecDeque;
use sp_core::keccak_256;

use crate::types::Hash;
use crate::constants::RECENT_HISTORY_SIZE;
use crate::blockchain::state::recent_history::codec::{State as BlockHistory, BlockInfo, ReportedWorkPackages, Mmr};
use crate::utils::trie::append;

pub mod codec;

static RECENT_HISTORY_STATE: Lazy<Mutex<BlockHistory>> = Lazy::new(|| Mutex::new(BlockHistory{beta: VecDeque::with_capacity(RECENT_HISTORY_SIZE)}));

pub fn set_history_state(post_state: &BlockHistory) {
    let mut state = RECENT_HISTORY_STATE.lock().unwrap();
    *state = post_state.clone();
}

pub fn get_history_state() -> BlockHistory {
    let state = RECENT_HISTORY_STATE.lock().unwrap(); 
    return state.clone();
}

pub fn update_recent_history(
    header_hash: Hash, 
    parent_state_root: Hash, 
    accumulate_root: Hash, 
    work_packages: ReportedWorkPackages
) {
    let mut pre_state = RECENT_HISTORY_STATE.lock().unwrap(); 
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

