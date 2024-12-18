use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::array::from_fn;

use crate::types::{ValidatorsData, ValidatorData, BandersnatchPublic, Ed25519Public, BlsPublic, Metadata};

static PREV_VALIDATORS_STATE: Lazy<Mutex<ValidatorsData>> = Lazy::new(|| Mutex::new(ValidatorsData{validators: Box::new(from_fn(|_| ValidatorData {
    bandersnatch: [0u8; std::mem::size_of::<BandersnatchPublic>()],
    ed25519: [0u8; std::mem::size_of::<Ed25519Public>()],
    bls: [0u8; std::mem::size_of::<BlsPublic>()],
    metadata: [0u8; std::mem::size_of::<Metadata>()],
    }))}));

static CURR_VALIDATORS_STATE: Lazy<Mutex<ValidatorsData>> = Lazy::new(|| Mutex::new(ValidatorsData{validators: Box::new(from_fn(|_| ValidatorData {
    bandersnatch: [0u8; std::mem::size_of::<BandersnatchPublic>()],
    ed25519: [0u8; std::mem::size_of::<Ed25519Public>()],
    bls: [0u8; std::mem::size_of::<BlsPublic>()],
    metadata: [0u8; std::mem::size_of::<Metadata>()],
    }))}));

static NEXT_VALIDATORS_STATE: Lazy<Mutex<ValidatorsData>> = Lazy::new(|| Mutex::new(ValidatorsData{validators: Box::new(from_fn(|_| ValidatorData {
    bandersnatch: [0u8; std::mem::size_of::<BandersnatchPublic>()],
    ed25519: [0u8; std::mem::size_of::<Ed25519Public>()],
    bls: [0u8; std::mem::size_of::<BlsPublic>()],
    metadata: [0u8; std::mem::size_of::<Metadata>()],
    }))}));


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
