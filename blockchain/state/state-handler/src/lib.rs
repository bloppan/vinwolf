use {once_cell::sync::Lazy, std::sync::Mutex};

use jam_types::{
    AccumulatedHistory, AuthPools, AuthQueues, AvailabilityAssignments, RecentBlocks, DisputesRecords, EntropyPool, GlobalState, CoreIndex,
    OpaqueHash, Privileges, ReadyQueue, Safrole, ServiceAccounts, Statistics, TimeSlot, ValidatorSet, ValidatorsData, AvailabilityAssignment,
    Offenders, WorkReportHash, ProcessError, DisputesErrorCode, Entropy
};
use codec::Encode;

static GLOBAL_STATE: Lazy<Mutex<GlobalState>> = Lazy::new(|| {
    Mutex::new(GlobalState::default())
});

static STATE_ROOT: Lazy<Mutex<OpaqueHash>> = Lazy::new(|| {
    Mutex::new(OpaqueHash::default())
});

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
pub fn get_state_root() -> &'static Mutex<OpaqueHash> {
    &STATE_ROOT
}

static CURRENT_SLOT: Lazy<Mutex<TimeSlot>> = Lazy::new(|| {
    Mutex::new(TimeSlot::default())
});

pub mod time {

    use super::*;

    pub fn set(new_time: TimeSlot) {
        let mut state = GLOBAL_STATE.lock().unwrap();
        state.time = new_time;
    }
    pub fn get() -> TimeSlot {
        let state = GLOBAL_STATE.lock().unwrap();
        state.time.clone()
    }
    pub fn set_current(slot: &TimeSlot) {
        let mut current_slot = CURRENT_SLOT.lock().unwrap();
        *current_slot = *slot;
    }
    pub fn get_current() -> TimeSlot {
        let current_slot = CURRENT_SLOT.lock().unwrap();
        current_slot.clone()
    }
}

static RECENT_ENTROPY: Lazy<Mutex<OpaqueHash>> = Lazy::new(|| {
    Mutex::new(OpaqueHash::default())
});

pub mod entropy {

    use super::*;

    pub fn set(new_entropy: EntropyPool) {
        let mut state = GLOBAL_STATE.lock().unwrap();
        state.entropy = new_entropy;
    }
    pub fn get() -> EntropyPool {
        let state = GLOBAL_STATE.lock().unwrap();
        state.entropy.clone()
    }
    pub fn get_recent() -> Entropy {
        let recent_entropy = RECENT_ENTROPY.lock().unwrap();
        let entropy = recent_entropy.clone();
        Entropy { entropy }
    }
    pub fn set_recent(entropy: OpaqueHash) {
        let mut recent_entropy = RECENT_ENTROPY.lock().unwrap();
        *recent_entropy = entropy;
    }
}

static CURR_BLOCK_HISTORY: Lazy<Mutex<RecentBlocks>> = Lazy::new(|| {
    Mutex::new(RecentBlocks::default())
});

pub mod recent_history {
    
    use super::*;

    pub fn set(new_recent_history: RecentBlocks) {
        let mut state = GLOBAL_STATE.lock().unwrap();
        state.recent_history = new_recent_history;
    }
    pub fn get() -> RecentBlocks {
        let state = GLOBAL_STATE.lock().unwrap();
        state.recent_history.clone()
    }
    pub fn get_current() -> &'static Mutex<RecentBlocks> {
        &CURR_BLOCK_HISTORY
    }
    pub fn set_current(new_state: RecentBlocks) {
        *CURR_BLOCK_HISTORY.lock().unwrap() = new_state;
    }
}

pub mod reports {

    use super::*;

    pub fn set(new_availability: AvailabilityAssignments) {
        let mut state = GLOBAL_STATE.lock().unwrap();
        state.availability = new_availability;
    }
    pub fn get() -> AvailabilityAssignments {
        let state = GLOBAL_STATE.lock().unwrap();
        state.availability.clone()
    }
    pub fn add_assignment(assignment: &AvailabilityAssignment, state: &mut AvailabilityAssignments) {
        state.list[assignment.report.core_index as usize] = Some(assignment.clone());
    }
    pub fn remove_assignment(core_index: &CoreIndex, state: &mut AvailabilityAssignments) {
        state.list[*core_index as usize] = None;
    }
    pub fn update_first_step(availability_state: &mut AvailabilityAssignments, disputes_state: &DisputesRecords) {
        // We clear any work-reports which we judged as uncertain or invalid from their core:
        for assignment in availability_state.list.iter_mut() {
            if let Some(availability_assignment) = assignment {
                // Calculate target hash
                let target_hash = sp_core::blake2_256(&availability_assignment.report.encode());
                // Check if the hash is contained in bad or wonky sets
                if disputes_state.bad.contains(&target_hash)
                    || disputes_state.wonky.contains(&target_hash)
                {
                    *assignment = None; // Set to None
                }
            }
        }
    }
}

pub mod disputes {

    use super::*;

    pub fn set(new_disputes: DisputesRecords) {
        let mut state = GLOBAL_STATE.lock().unwrap();
        state.disputes = new_disputes;
    }
    pub fn get() -> DisputesRecords {
        let state = GLOBAL_STATE.lock().unwrap();
        state.disputes.clone()
    }
    pub fn update(
        disputes_state: &mut DisputesRecords, 
        new_wr_reported: &DisputesRecords, 
        culprits_keys: &[WorkReportHash],
        faults_keys: &[WorkReportHash]) 
    -> Result<Offenders, ProcessError> {

        let new_offenders = Vec::from([culprits_keys, faults_keys].concat());

        // In the disputes extrinsic can not be offenders already reported
        let all_offenders = Vec::from([disputes_state.offenders.clone(), new_offenders.clone()].concat());
        if utils::common::has_duplicates(&all_offenders) {
            return Err(ProcessError::DisputesError(DisputesErrorCode::OffenderAlreadyReported));
        }   

        // If the state was initialized, then we save the auxiliar records in the state
        disputes_state.good.extend_from_slice(&new_wr_reported.good);
        disputes_state.bad.extend_from_slice(&new_wr_reported.bad);
        disputes_state.wonky.extend_from_slice(&new_wr_reported.wonky);
        let mut offenders = new_offenders.clone();
        offenders.sort();
        disputes_state.offenders.extend_from_slice(&offenders);

        Ok(new_offenders)
    }
}


pub mod auth_pools {
    
    use super::*;

    pub fn set(new_authpool: AuthPools) {
        let mut state = GLOBAL_STATE.lock().unwrap();
        state.auth_pools = new_authpool;
    }
    pub fn get() -> AuthPools {
        let state = GLOBAL_STATE.lock().unwrap();
        state.auth_pools.clone()
    }
}

pub mod auth_queues {

    use super::*;

    pub fn set(new_authqueue: AuthQueues) {
        let mut state = GLOBAL_STATE.lock().unwrap();
        state.auth_queues = new_authqueue;
    }
    pub fn get() -> AuthQueues {
        let state = GLOBAL_STATE.lock().unwrap();
        state.auth_queues.clone()
    }
}

pub mod acc_outputs {

    use jam_types::RecentAccOutputs;

    use super::*;

    pub fn set(new_acc_outputs: RecentAccOutputs) {
        let mut state = GLOBAL_STATE.lock().unwrap();
        state.recent_acc_outputs = new_acc_outputs;
    }
    pub fn get() -> RecentAccOutputs {
        let state = GLOBAL_STATE.lock().unwrap();
        state.recent_acc_outputs.clone()
    }
}

pub mod statistics {

    use super::*;

    pub fn set(new_statistics: Statistics) {
        let mut state = GLOBAL_STATE.lock().unwrap();
        state.statistics = new_statistics;
    }
    pub fn get() -> Statistics {
        let state = GLOBAL_STATE.lock().unwrap();
        state.statistics.clone()
    }
}

pub mod safrole {
    
    use super::*;

    pub fn set(new_safrole: Safrole) {
        let mut state = GLOBAL_STATE.lock().unwrap();
        state.safrole = new_safrole;
    }
    pub fn get() -> Safrole {
        let state = GLOBAL_STATE.lock().unwrap();
        state.safrole.clone()
    }
}

pub mod service_accounts {

    use super::*;

    pub fn set(new_service_accounts: ServiceAccounts) {
        let mut state = GLOBAL_STATE.lock().unwrap();
        state.service_accounts = new_service_accounts;
    }
    pub fn get() -> ServiceAccounts {
        let state = GLOBAL_STATE.lock().unwrap();
        state.service_accounts.clone()
    }
}

pub mod acc_history {

    use super::*;

    pub fn set(new_accumulation_history: AccumulatedHistory) {
        let mut state = GLOBAL_STATE.lock().unwrap();
        state.accumulation_history = new_accumulation_history;
    }
    pub fn get() -> AccumulatedHistory {
        let state = GLOBAL_STATE.lock().unwrap();
        state.accumulation_history.clone()
    }
}

pub mod ready_queue {

    use super::*;

    pub fn set(new_ready_queue: ReadyQueue) {
        let mut state = GLOBAL_STATE.lock().unwrap();
        state.ready_queue = new_ready_queue;
    }
    pub fn get() -> ReadyQueue {
        let state = GLOBAL_STATE.lock().unwrap();
        state.ready_queue.clone()
    }
}

pub mod privileges {

    use super::*;

    pub fn set(new_privileges: Privileges) {
        let mut state = GLOBAL_STATE.lock().unwrap();
        state.privileges = new_privileges;
    }
    pub fn get() -> Privileges {
        let state = GLOBAL_STATE.lock().unwrap();
        state.privileges.clone()
    }
}

pub mod validators {

    use super::*;

    pub fn set(new_validators: ValidatorsData, validator_set: ValidatorSet) {

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
    pub fn get(validator_set: ValidatorSet) -> ValidatorsData {
        
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
}