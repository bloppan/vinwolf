use once_cell::sync::Lazy;
use std::sync::Mutex;

use crate::types::EntropyBuffer;

static ENTROPY_STATE: Lazy<Mutex<EntropyBuffer>> = Lazy::new(|| Mutex::new(Box::new([[0u8; 32]; 4])));

pub fn set_entropy_state(post_state: &EntropyBuffer) {
    let mut state = ENTROPY_STATE.lock().unwrap();
    *state = post_state.clone();
}

pub fn get_entropy_state() -> EntropyBuffer {
    let state = ENTROPY_STATE.lock().unwrap(); 
    return state.clone();
}

