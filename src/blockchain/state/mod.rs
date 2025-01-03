use once_cell::sync::Lazy;
use std::sync::Mutex;
use crate::types::{
    AssurancesErrorCode, AuthPools, AuthQueues, AvailabilityAssignments, Block, BlockHistory, DisputesErrorCode, DisputesRecords, EntropyPool, Safrole, SafroleErrorCode, Statistics, TimeSlot, ValidatorsData
};
use validators::ValidatorSet;
use reporting_assurance::{process_assurances, process_guarantees};
use statistics::process_statistics;
use recent_history::process_recent_history;
use crate::utils::codec::ReadError;
use crate::utils::codec::jam::work_report::ReportErrorCode;

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

static GLOBAL_STATE: Lazy<Mutex<GlobalState>> = Lazy::new(|| {
    Mutex::new(GlobalState::default())
});

#[derive(Clone)]
pub struct GlobalState {
    pub time: TimeSlot,
    pub availability: AvailabilityAssignments,
    pub entropy: EntropyPool,
    pub recent_history: BlockHistory,
    pub auth_pools: AuthPools,
    pub auth_queues: AuthQueues,
    pub statistics: Statistics,
    pub prev_validators: ValidatorsData,
    pub curr_validators: ValidatorsData,
    pub next_validators: ValidatorsData,
    pub disputes: DisputesRecords,
    pub safrole: Safrole,
}

impl Default for GlobalState {
    fn default() -> Self {
        GlobalState {
            time: TimeSlot::default(),
            availability: AvailabilityAssignments::default(),
            entropy: EntropyPool::default(),
            recent_history: BlockHistory::default(),
            auth_pools: AuthPools::default(),
            auth_queues: AuthQueues::default(),
            statistics: Statistics::default(),
            prev_validators: ValidatorsData::default(),
            curr_validators: ValidatorsData::default(),
            next_validators: ValidatorsData::default(),
            disputes: DisputesRecords::default(),
            safrole: Safrole::default(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ProcessError {
    ReadError(ReadError),
    SafroleError(SafroleErrorCode),
    DisputesError(DisputesErrorCode),
    ReportError(ReportErrorCode),
    AssurancesError(AssurancesErrorCode),
}

pub fn state_transition_function(block: &Block) -> Result<(), ProcessError> {
    
    let mut new_state = get_global_state().lock().unwrap().clone();
    
    // Process report and assurance
    process_assurances(
        &mut new_state.availability,
        &block.extrinsic.assurances,
        &block.header.slot,
        &block.header.parent,
    )?;
    let _ = process_guarantees(
        &mut new_state.availability, 
        &block.extrinsic.guarantees,
        &block.header.slot
    )?; 
    // Process recent history
    /*process_recent_history(
        &mut new_state.recent_history, 
        &block.header.parent, 
        &block.header.parent_state_root, 
        &block.header.accumulate_root, 
        block.extrinsic.work_packages.clone());*/
    // Process Authorization
    //process_authorizations(&mut new_state.auth_pools, &block.header.slot, code_authorizers);
    process_statistics(
        &mut new_state.statistics, 
        &block.header.slot, 
        &block.header.author_index, 
        &block.extrinsic
    );
    

    set_global_state(new_state);
    
    Ok(())
}

pub fn get_global_state() -> &'static Mutex<GlobalState> {
    &GLOBAL_STATE
}
pub fn set_global_state(new_state: GlobalState) {
    *GLOBAL_STATE.lock().unwrap() = new_state;
}
// Time
pub fn set_time(new_time: TimeSlot) {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.time = new_time;
}
pub fn get_time() -> TimeSlot {
    let state = GLOBAL_STATE.lock().unwrap();
    state.time.clone()
}
// Entropy
pub fn set_entropy(new_entropy: EntropyPool) {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.entropy = new_entropy;
}
pub fn get_entropy() -> EntropyPool {
    let state = GLOBAL_STATE.lock().unwrap();
    state.entropy.clone()
}
// Authorization Pools
pub fn set_authpools(new_authpool: AuthPools) {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.auth_pools = new_authpool;
}
pub fn get_authpools() -> AuthPools {
    let state = GLOBAL_STATE.lock().unwrap();
    state.auth_pools.clone()
}
// Authorizations Queues
pub fn set_authqueues(new_authqueue: AuthQueues) {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.auth_queues = new_authqueue;
}
pub fn get_authqueues() -> AuthQueues {
    let state = GLOBAL_STATE.lock().unwrap();
    state.auth_queues.clone()
}
// Disputes
pub fn set_disputes(new_disputes: DisputesRecords) {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.disputes = new_disputes;
}
pub fn get_disputes() -> DisputesRecords {
    let state = GLOBAL_STATE.lock().unwrap();
    state.disputes.clone()
}
// Reporting and assurance
pub fn set_reporting_assurance(new_availability: AvailabilityAssignments) {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.availability = new_availability;
}
pub fn get_reporting_assurance() -> AvailabilityAssignments {
    let state = GLOBAL_STATE.lock().unwrap();
    state.availability.clone()
}
// Statistics
pub fn set_statistics(new_statistics: Statistics) {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.statistics = new_statistics;
}
pub fn get_statistics() -> Statistics {
    let state = GLOBAL_STATE.lock().unwrap();
    state.statistics.clone()
}
// Recent History
pub fn set_recent_history(new_recent_history: BlockHistory) {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.recent_history = new_recent_history;
}
pub fn get_recent_history() -> BlockHistory {
    let state = GLOBAL_STATE.lock().unwrap();
    state.recent_history.clone()
}
// Safrole
pub fn set_safrole(new_safrole: Safrole) {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.safrole = new_safrole;
}
pub fn get_safrole() -> Safrole {
    let state = GLOBAL_STATE.lock().unwrap();
    state.safrole.clone()
}
// Validators
pub fn set_validators(new_validators: ValidatorsData, validator_set: ValidatorSet) {

    let mut state = GLOBAL_STATE.lock().unwrap();

    match validator_set {
        ValidatorSet::Previous => {
            state.prev_validators = new_validators;
        },
        ValidatorSet::Current => {
            state.curr_validators = new_validators;
        },
        ValidatorSet::Next => {
            state.next_validators = new_validators;
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