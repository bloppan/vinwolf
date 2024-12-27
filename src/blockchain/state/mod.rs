use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::collections::VecDeque;
use std::mem::size_of;
use std::array::from_fn;

use crate::constants::{ENTROPY_POOL_SIZE, MAX_ITEMS_AUTHORIZATION_POOL, RECENT_HISTORY_SIZE};
use crate::types::{
    AuthPool, AuthPools, AuthQueue, AuthQueues, AuthorizerHash, AvailabilityAssignments, Block, BlockHistory, EntropyBuffer, Hash, 
    Statistics, TimeSlot, ValidatorsData
};
use crate::blockchain::state::validators::ValidatorSet;
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
pub mod statistics;
pub mod validators;


#[derive(Clone)]
pub struct GlobalState {
    pub time: TimeSlot,
    pub availability: AvailabilityAssignments,
    pub entropy: EntropyBuffer,
    pub recent_history: BlockHistory,
    pub auth_pools: AuthPools,
    pub auth_queues: AuthQueues,
    pub statistics: Statistics,
    pub prev_validators: ValidatorsData,
    pub curr_validators: ValidatorsData,
    pub next_validators: ValidatorsData,
}

static GLOBAL_STATE: Lazy<Mutex<GlobalState>> = Lazy::new(|| {
    Mutex::new(GlobalState {
        time: TimeSlot::default(),
        availability: AvailabilityAssignments {
            assignments: Box::new(from_fn(|_| None)),
        },
        entropy: Box::new([[0u8; size_of::<Hash>()]; ENTROPY_POOL_SIZE]),
        recent_history: BlockHistory {
            beta: VecDeque::with_capacity(RECENT_HISTORY_SIZE) 
        },
        auth_pools: AuthPools { auth_pools: Box::new(from_fn(|_| AuthPool { auth_pool: VecDeque::with_capacity(MAX_ITEMS_AUTHORIZATION_POOL) })) },
        auth_queues: AuthQueues{ auth_queues: Box::new(from_fn(|_| AuthQueue { auth_queue: Box::new(from_fn(|_| [0; size_of::<AuthorizerHash>()])) }))},
        statistics: Statistics::default(),
        prev_validators: ValidatorsData::default(),
        curr_validators: ValidatorsData::default(),
        next_validators: ValidatorsData::default(),
    })
});

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

pub fn get_global_state() -> GlobalState {
    let state = GLOBAL_STATE.lock().unwrap();
    state.clone()
}
pub fn set_global_state(new_state: &GlobalState) {
    let mut state = GLOBAL_STATE.lock().unwrap();
    *state = new_state.clone();
}
// Time
pub fn set_time(new_time: &TimeSlot) {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.time = new_time.clone();
}
pub fn get_time() -> TimeSlot {
    let state = GLOBAL_STATE.lock().unwrap();
    state.time.clone()
}
// Authorization Pools
pub fn set_authpools(new_authpool: &AuthPools) {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.auth_pools = new_authpool.clone();
}
pub fn get_authpools() -> AuthPools {
    let state = GLOBAL_STATE.lock().unwrap();
    state.auth_pools.clone()
}
// Authorizations Queues
pub fn set_authqueues(new_authqueue: &AuthQueues) {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.auth_queues = new_authqueue.clone();
}
pub fn get_authqueues() -> AuthQueues {
    let state = GLOBAL_STATE.lock().unwrap();
    state.auth_queues.clone()
}
// Reporting and assurance
pub fn set_reporting_assurance(new_availability: &AvailabilityAssignments) {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.availability = new_availability.clone();
}
pub fn get_reporting_assurance() -> AvailabilityAssignments {
    let state = GLOBAL_STATE.lock().unwrap();
    state.availability.clone()
}
// Statistics
pub fn set_statistics(new_statistics: &Statistics) {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.statistics = new_statistics.clone();
}
pub fn get_statistics() -> Statistics {
    let state = GLOBAL_STATE.lock().unwrap();
    state.statistics.clone()
}
// Validators
/*pubenum ValidatorSet {
    Previous,
    Current,
    Next,
}*/

pub fn set_validators(new_validators: &ValidatorsData, validator_set: ValidatorSet) {

    let mut state = GLOBAL_STATE.lock().unwrap();

    match validator_set {
        ValidatorSet::Previous => {
            state.prev_validators = new_validators.clone();
        },
        ValidatorSet::Current => {
            state.curr_validators = new_validators.clone();
        },
        ValidatorSet::Next => {
            state.next_validators = new_validators.clone();
        },
    }    
}
pub fn get_validators(validator_set: ValidatorSet) -> ValidatorsData {
    
    match validator_set {
        ValidatorSet::Previous => {
            return GLOBAL_STATE.lock().unwrap().prev_validators.clone();
        },
        ValidatorSet::Current => {
            return GLOBAL_STATE.lock().unwrap().curr_validators.clone();
        },
        ValidatorSet::Next => {
            return GLOBAL_STATE.lock().unwrap().next_validators.clone();
        },
    }
}