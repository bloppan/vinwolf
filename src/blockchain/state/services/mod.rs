use once_cell::sync::Lazy;
use std::sync::Mutex;

use crate::types::Services;

static SERVICES_STATE: Lazy<Mutex<Services>> = Lazy::new(|| Mutex::new(Services{0: Vec::new()}));

pub fn set_services_state(post_state: &Services) {
    let mut state = SERVICES_STATE.lock().unwrap();
    *state = post_state.clone();
}

pub fn get_services_state() -> Services {
    let state = SERVICES_STATE.lock().unwrap(); 
    return state.clone();
}
