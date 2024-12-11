use once_cell::sync::Lazy;
use std::sync::Mutex;

use crate::codec::safrole::ValidatorsData;

static PREV_VALIDATORS_STATE: Lazy<Mutex<ValidatorsData>> = Lazy::new(|| Mutex::new(ValidatorsData{validators: vec![]}));
static CURR_VALIDATORS_STATE: Lazy<Mutex<ValidatorsData>> = Lazy::new(|| Mutex::new(ValidatorsData{validators: vec![]}));
static NEXT_VALIDATORS_STATE: Lazy<Mutex<ValidatorsData>> = Lazy::new(|| Mutex::new(ValidatorsData{validators: vec![]}));

pub enum ValidatorSet {
    Previous,
    Current,
    Next,
}

pub fn set_validators_state(post_state: &ValidatorsData, validator_set: ValidatorSet) {

    match validator_set {

        ValidatorSet::Previous => {
            let mut state = PREV_VALIDATORS_STATE.lock().unwrap();
            *state = post_state.clone();
        },
        ValidatorSet::Current => {
            let mut state = CURR_VALIDATORS_STATE.lock().unwrap();
            *state = post_state.clone();
        },
        ValidatorSet::Next => {
            let mut state = NEXT_VALIDATORS_STATE.lock().unwrap();
            *state = post_state.clone();
        },
    }    
}

pub fn get_validators_state(validator_set: ValidatorSet) -> ValidatorsData {
    
    match validator_set {

        ValidatorSet::Previous => {
            let state = PREV_VALIDATORS_STATE.lock().unwrap(); 
            return state.clone();
        },
        ValidatorSet::Current => {
            let state = CURR_VALIDATORS_STATE.lock().unwrap(); 
            return state.clone();
        },
        ValidatorSet::Next => {
            let state = NEXT_VALIDATORS_STATE.lock().unwrap(); 
            return state.clone();
        },
    }
}
