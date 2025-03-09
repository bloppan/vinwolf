use once_cell::sync::Lazy;
use sp_core::blake2_256;
use std::sync::Mutex;
use crate::types::{
    AccumulatedHistory, AuthPools, AuthQueues, AvailabilityAssignments, Block, BlockHistory, DisputesRecords, EntropyPool, GlobalState, 
    OpaqueHash, Privileges, ProcessError, ReadyQueue, ReportedWorkPackages, Safrole, SerializedState, ServiceAccounts, ServiceInfo, StateKey, 
    Statistics, TimeSlot, ValidatorSet, ValidatorsData
};
use crate::constants::{
    ACCUMULATION_HISTORY, AUTH_POOLS, AUTH_QUEUE, AVAILABILITY, CURR_VALIDATORS, DISPUTES, ENTROPY, NEXT_VALIDATORS, PREV_VALIDATORS, PRIVILEGES, 
    READY_QUEUE, RECENT_HISTORY, SAFROLE, STATISTICS, TIME
};
use crate::utils::codec::{Encode, EncodeLen};
use crate::utils::codec::jam::global_state::{construct_preimage_key, construct_lookup_key, construct_storage_key, StateKeyTrait};

use reporting_assurance::{process_assurances, process_guarantees};
use statistics::process_statistics;
use safrole::process_safrole;
use authorization::process_authorizations;
use recent_history::process_recent_history;

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
pub mod privileges;

static GLOBAL_STATE: Lazy<Mutex<GlobalState>> = Lazy::new(|| {
    Mutex::new(GlobalState::default())
});

static STATE_ROOT: Lazy<Mutex<OpaqueHash>> = Lazy::new(|| {
    Mutex::new(OpaqueHash::default())
});


pub fn state_transition_function(block: &Block) -> Result<(), ProcessError> {
    
    let mut new_state = get_global_state().lock().unwrap().clone();
    
    process_safrole(
        &mut new_state.safrole,
        &mut new_state.entropy,
        &mut new_state.curr_validators,
        &mut new_state.prev_validators,
        &mut new_state.time,
        &block.header,
        &block.extrinsic.tickets)?;

    // Process report and assurance
    process_assurances(
        &mut new_state.availability,
        &block.extrinsic.assurances,
        &block.header.unsigned.slot,
        &block.header.unsigned.parent,
    )?;

    let _ = process_guarantees(
        &mut new_state.availability, 
        &block.extrinsic.guarantees,
        &block.header.unsigned.slot
    )?; 

    let reported: ReportedWorkPackages = ReportedWorkPackages::default();

    // Process recent history
    process_recent_history(
        &mut new_state.recent_history, 
        &blake2_256(&block.header.encode()), 
        &block.header.unsigned.parent_state_root, 
        &[0u8; 32], 
        &reported);
        
    // Process Authorization
    process_authorizations(
        &mut new_state.auth_pools, 
        &block.header.unsigned.slot, 
        &block.extrinsic.guarantees);

    process_statistics(
        &mut new_state.statistics, 
        &block.header.unsigned.slot, 
        &block.header.unsigned.author_index, 
        &block.extrinsic
    );
    
    set_global_state(new_state);    

    Ok(())
}

impl GlobalState {

    pub fn serialize(&self) -> SerializedState {

        let mut state = SerializedState::default();

        state.map.insert(StateKey::U8(AUTH_POOLS).construct(), self.auth_pools.encode());
        state.map.insert(StateKey::U8(AUTH_QUEUE).construct(), self.auth_queues.encode());
        state.map.insert(StateKey::U8(RECENT_HISTORY).construct(), self.recent_history.encode());
        state.map.insert(StateKey::U8(SAFROLE).construct(), self.safrole.encode());
        state.map.insert(StateKey::U8(DISPUTES).construct(), self.disputes.encode());
        state.map.insert(StateKey::U8(ENTROPY).construct(), self.entropy.encode());
        state.map.insert(StateKey::U8(NEXT_VALIDATORS).construct(), self.next_validators.encode());
        state.map.insert(StateKey::U8(CURR_VALIDATORS).construct(), self.curr_validators.encode());
        state.map.insert(StateKey::U8(PREV_VALIDATORS).construct(), self.prev_validators.encode());
        state.map.insert(StateKey::U8(AVAILABILITY).construct(), self.availability.encode());
        state.map.insert(StateKey::U8(TIME).construct(), self.time.encode());
        state.map.insert(StateKey::U8(PRIVILEGES).construct(), self.privileges.encode());
        state.map.insert(StateKey::U8(STATISTICS).construct(), self.statistics.encode());
        state.map.insert(StateKey::U8(ACCUMULATION_HISTORY).construct(), self.accumulation_history.encode());
        state.map.insert(StateKey::U8(READY_QUEUE).construct(), self.ready_queue.encode());

        for service in self.service_accounts.service_accounts.iter() {
            let key = StateKey::Service(255, *service.0).construct();       
            let service_info = ServiceInfo {
                balance: service.1.balance,
                code_hash: service.1.code_hash,
                min_item_gas: service.1.gas,
                min_memo_gas: service.1.min_gas,
                bytes: service.1.bytes,
                items: service.1.items,
            };

            state.map.insert(key, service_info.encode());

            for preimage in service.1.preimages.iter() {
                let key = StateKey::Account(*service.0, construct_preimage_key(preimage.0).to_vec()).construct();
                state.map.insert(key, preimage.1.encode());
            }
            
            for lookup in service.1.lookup.iter() {
                let key = StateKey::Account(*service.0, construct_lookup_key(&lookup.0.0, lookup.0.1).to_vec()).construct();
                state.map.insert(key, lookup.1.as_slice().encode_len());
            }

            for item in service.1.storage.iter() {
                let key = StateKey::Account(*service.0, construct_storage_key(item.0).to_vec()).construct();
                state.map.insert(key, item.1.encode());
            }
        }

        return state;
    }
}

// Global state
pub fn get_global_state() -> &'static Mutex<GlobalState> {
    &GLOBAL_STATE
}
pub fn set_global_state(new_state: GlobalState) {
    *GLOBAL_STATE.lock().unwrap() = new_state;
}
// State root
pub fn set_state_root(new_root: OpaqueHash) {
    *STATE_ROOT.lock().unwrap() = new_root;
}
pub fn get_state_root() -> OpaqueHash {
    let state = STATE_ROOT.lock().unwrap();
    state.clone()
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
// Service Accounts
pub fn set_service_accounts(new_service_accounts: ServiceAccounts) {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.service_accounts = new_service_accounts;
}
pub fn get_service_accounts() -> ServiceAccounts {
    let state = GLOBAL_STATE.lock().unwrap();
    state.service_accounts.clone()
}
// Accumulation History
pub fn set_accumulation_history(new_accumulation_history: AccumulatedHistory) {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.accumulation_history = new_accumulation_history;
}
pub fn get_accumulation_history() -> AccumulatedHistory {
    let state = GLOBAL_STATE.lock().unwrap();
    state.accumulation_history.clone()
}
// Ready Queue
pub fn set_ready_queue(new_ready_queue: ReadyQueue) {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.ready_queue = new_ready_queue;
}
pub fn get_ready_queue() -> ReadyQueue {
    let state = GLOBAL_STATE.lock().unwrap();
    state.ready_queue.clone()
}
// Privileges
pub fn set_privileges(new_privileges: Privileges) {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.privileges = new_privileges;
}
pub fn get_privileges() -> Privileges {
    let state = GLOBAL_STATE.lock().unwrap();
    state.privileges.clone()
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