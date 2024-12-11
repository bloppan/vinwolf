use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::collections::VecDeque;

use crate::constants::RECENT_HISTORY_SIZE;
use crate::codec::history::State as BlockHistory;

static RECENT_HISTORY_STATE: Lazy<Mutex<BlockHistory>> = Lazy::new(|| Mutex::new(BlockHistory{beta: VecDeque::with_capacity(RECENT_HISTORY_SIZE)}));

pub fn set_history_state(post_state: &BlockHistory) {
    let mut state = RECENT_HISTORY_STATE.lock().unwrap();
    *state = post_state.clone();
}

pub fn get_history_state() -> BlockHistory {
    let state = RECENT_HISTORY_STATE.lock().unwrap(); 
    return state.clone();
}