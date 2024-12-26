use once_cell::sync::Lazy;
use std::sync::Mutex;

use crate::types::TimeSlot;

static TIME_STATE: Lazy<Mutex<Tau>> = Lazy::new(|| Mutex::new(Tau{ tau: 0 }));

#[derive(Clone, PartialEq, PartialOrd)]
pub struct Tau {
    pub tau: TimeSlot,
}

pub fn set_time_state(post_state: &TimeSlot) {
    let mut state = TIME_STATE.lock().unwrap();
    *state = Tau { tau: *post_state };
}

pub fn get_time_state() -> TimeSlot {
    let state = TIME_STATE.lock().unwrap(); 
    return state.tau.clone();
}

