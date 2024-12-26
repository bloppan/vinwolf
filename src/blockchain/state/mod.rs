use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::collections::VecDeque;
use std::mem::size_of;
use std::array::from_fn;

use crate::constants::{CORES_COUNT, RECENT_HISTORY_SIZE};
use crate::types::{
    AuthorizerHash, AuthPool, AuthPools, AuthQueue, AuthQueues, AvailabilityAssignments, Block, BlockHistory, EntropyBuffer, Hash
};
use crate::blockchain::state::reporting_assurance::{process_assurances, process_guarantees};
use crate::blockchain::block::extrinsic::assurances::AssurancesErrorCode;
use crate::utils::codec::ReadError;
use crate::utils::codec::work_report::ReportErrorCode;

pub mod accumulation;
pub mod authorization;
pub mod disputes;
pub mod entropy;
pub mod recent_history;
pub mod reporting_assurance;
pub mod safrole;
pub mod services;
pub mod time;
pub mod validator_statistics;
pub mod validators;


#[derive(Clone)]
pub struct GlobalState {
    pub availability: AvailabilityAssignments,
    pub entropy: EntropyBuffer,
    pub recent_history: BlockHistory,
    pub auth_pools: AuthPools,
    pub auth_queues: AuthQueues,
    /*prev_validators: Box<[ValidatorData; VALIDATORS_COUNT]>,
    curr_validators: Box<[ValidatorData; VALIDATORS_COUNT]>,
    next_validators: Box<[ValidatorData; VALIDATORS_COUNT]>,*/
}

static GLOBAL_STATE: Lazy<Mutex<GlobalState>> = Lazy::new(|| {
    Mutex::new(GlobalState {
        availability: AvailabilityAssignments {
            assignments: Box::new(from_fn(|_| None)),
        },
        entropy: Box::new([[0u8; size_of::<Hash>()]; 4]),
        recent_history: BlockHistory {
            beta: VecDeque::with_capacity(RECENT_HISTORY_SIZE) 
        },
        auth_pools: AuthPools { auth_pools: Box::new(from_fn(|_| AuthPool { auth_pool: Vec::new() })) },
        auth_queues: AuthQueues{ auth_queues: Box::new(from_fn(|_| AuthQueue { auth_queue: Box::new(from_fn(|_| [0; size_of::<AuthorizerHash>()])) }))},
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

pub fn set_reporting_assurance_state(new_availability: &AvailabilityAssignments) {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.availability = new_availability.clone();
}

pub fn get_reporting_assurance_state() -> AvailabilityAssignments {
    let state = GLOBAL_STATE.lock().unwrap();
    state.availability.clone()
}

#[derive(Debug, PartialEq)]
pub enum ProcessError {
    ReadError(ReadError),
    ReportError(ReportErrorCode),
    AssurancesError(AssurancesErrorCode),
}


pub fn state_transition_function(block: &Block) -> Result<(), ProcessError> {
    
    let current_state = get_global_state(); 
    let mut new_state = current_state.clone();
    
    // Process report and assurance
    let _ = process_assurances(&mut new_state.availability, &block.extrinsic.assurances, &block.header.slot, &block.header.parent)?;
    let _ = process_guarantees(&mut new_state.availability, &block.extrinsic.guarantees,&block.header.slot)?; 
    
    
    
    set_global_state(&new_state);
    Ok(())
}