use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::collections::VecDeque;

use crate::constants::{CORES_COUNT, MAX_ITEMS_AUTHORIZATION_POOL, MAX_ITEMS_AUTHORIZATION_QUEUE, RECENT_HISTORY_SIZE, VALIDATORS_COUNT};
use crate::types::{EntropyBuffer, Hash};
use crate::codec::work_report::{AuthPool, ErrorCode as ReportErrorCode};
use crate::codec::disputes_extrinsic::AvailabilityAssignments;
use crate::codec::history::State as BlockHistory;
use crate::codec::safrole::ValidatorData;

use crate::codec::block::Block;
use crate::blockchain::state::reporting_assurance::process_report_assurance;


pub mod authorization;
pub mod entropy;
pub mod time;
pub mod disputes;
pub mod recent_history;
pub mod reporting_assurance;
pub mod services;
pub mod validators;

#[derive(Clone)]
pub struct GlobalState {
    availability: AvailabilityAssignments,
    entropy: EntropyBuffer,
    recent_history: BlockHistory,
    auth_pools: Box<[AuthPool; CORES_COUNT]>,
    /*prev_validators: Box<[ValidatorData; VALIDATORS_COUNT]>,
    curr_validators: Box<[ValidatorData; VALIDATORS_COUNT]>,
    next_validators: Box<[ValidatorData; VALIDATORS_COUNT]>,*/
}

static GLOBAL_STATE: Lazy<Mutex<GlobalState>> = Lazy::new(|| {
    Mutex::new(GlobalState {
        availability: AvailabilityAssignments {
            assignments: Box::new(std::array::from_fn(|_| None)),
        },
        entropy: Box::new([[0u8; std::mem::size_of::<Hash>()]; 4]),
        recent_history: BlockHistory {
            beta: VecDeque::with_capacity(RECENT_HISTORY_SIZE) 
        },
        auth_pools: Box::new(std::array::from_fn(|_| AuthPool { auth_pool: Vec::new() })),
        //prev_validators: Box::new(ValidatorData{validators: vec![]}),
    })
});

pub fn get_global_state() -> GlobalState {
    let state = GLOBAL_STATE.lock().unwrap();
    state.clone()
}

pub fn set_global_state(new_state: &GlobalState) {
    let mut state = GLOBAL_STATE.lock().unwrap();
    *state = new_state.clone();
}

enum GlobalError {
    ReportError(ReportErrorCode),
}

pub fn state_transition_function(block: &Block) -> Result<(), GlobalError> {
    
    let current_state = get_global_state(); 
    let mut new_state = current_state.clone();
    
    // Process report and assurance
    if let Err(Error) = process_report_assurance(
        &mut new_state.availability,
        &block.extrinsic.guarantees,
        &block.header.slot,
    ) {
        return Err(GlobalError::ReportError(Error));
    }
    
    set_global_state(&new_state);
    Ok(())
}